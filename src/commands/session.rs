// Session commands - start, end, list

use std::process::Command;

use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::cli::{SessionCommands, SessionSubcommand};
use crate::config::ProjectConfig;
use crate::database::open_database;
use crate::git;
use crate::paths::get_tracking_db_path;
use crate::session::{
    create_session, end_session_with_structured, get_active_session, get_recent_sessions,
};

pub fn run(cmd: SessionCommands) -> Result<()> {
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    match cmd.command {
        SessionSubcommand::Start => cmd_start(&conn),
        SessionSubcommand::End { summary, force } => cmd_end(&conn, &summary, force),
        SessionSubcommand::List => cmd_list(&conn),
    }
}

/// Start a new session
fn cmd_start(conn: &rusqlite::Connection) -> Result<()> {
    // Check if there's already an active session
    if let Some(active) = get_active_session(conn)? {
        println!(
            "Session #{} is already active (started {})",
            active.session_id,
            active.started_at.format("%Y-%m-%d %H:%M")
        );
        return Ok(());
    }

    // Create a new session
    let session = create_session(conn)?;
    println!("{} Session #{} started", "✓".green(), session.session_id);
    Ok(())
}

/// End the current session with a summary
fn cmd_end(conn: &rusqlite::Connection, summary: &str, force: bool) -> Result<()> {
    // Get active session
    let session = match get_active_session(conn)? {
        Some(s) => s,
        None => bail!("No active session to end"),
    };

    // Check if any activity was logged
    let has_activity = check_session_has_activity(conn, session.session_id)?;

    // Display session activity
    display_session_activity(conn, session.session_id)?;

    // Display session review hints
    display_session_hints(
        conn,
        session.session_id,
        &session.started_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    )?;

    // If no activity and not forced, show options and exit
    if !has_activity && !force {
        println!();
        println!("{}", "─".repeat(50));
        println!();
        println!("{} No activity was logged this session.", "⚠".yellow());
        println!();
        println!("Before ending, you can capture what happened:");
        println!();
        println!(
            "  {} {} - Run proj log/task commands yourself",
            "1.".bold(),
            "Add manually".cyan()
        );
        println!(
            "  {} {} - AI analyzes conversation and logs items",
            "2.".bold(),
            "AI review".cyan()
        );
        println!(
            "  {} {} - Run: proj session end --force \"{}\"",
            "3.".bold(),
            "End anyway".cyan(),
            summary
        );
        println!();
        println!("{}", "Session not ended. Choose an option above.".dimmed());
        return Ok(());
    }

    // Build structured summary
    let structured = build_structured_summary(conn, session.session_id, summary)?;

    // End the session with structured summary
    end_session_with_structured(conn, session.session_id, summary, &structured)?;

    println!(
        "\n{} Session #{} ended. Summary: {}",
        "✓".green(),
        session.session_id,
        summary
    );

    // Handle auto-commit if enabled
    if let Err(e) = handle_auto_commit(summary) {
        // Don't fail the session end, just warn
        println!("  {} Auto-commit skipped: {}", "⚠".yellow(), e);
    }

    Ok(())
}

/// Check if a session has any logged activity
fn check_session_has_activity(conn: &rusqlite::Connection, session_id: i64) -> Result<bool> {
    // Check decisions
    let decision_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM decisions WHERE session_id = ?",
        [session_id],
        |row| row.get(0),
    )?;

    if decision_count > 0 {
        return Ok(true);
    }

    // Check tasks
    let task_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE session_id = ?",
        [session_id],
        |row| row.get(0),
    )?;

    if task_count > 0 {
        return Ok(true);
    }

    // Check blockers
    let blocker_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM blockers WHERE session_id = ?",
        [session_id],
        |row| row.get(0),
    )?;

    if blocker_count > 0 {
        return Ok(true);
    }

    // Check notes
    let note_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM context_notes WHERE session_id = ?",
        [session_id],
        |row| row.get(0),
    )?;

    if note_count > 0 {
        return Ok(true);
    }

    // Check questions
    let question_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM questions WHERE session_id = ?",
        [session_id],
        |row| row.get(0),
    )?;

    Ok(question_count > 0)
}

/// Display activity logged during a session
fn display_session_activity(conn: &rusqlite::Connection, session_id: i64) -> Result<()> {
    println!("{}", "Session Activity:".bold());
    println!("{}", "─".repeat(50));

    let mut has_activity = false;

    // Get decisions
    let mut stmt = conn.prepare(
        "SELECT topic, decision FROM decisions WHERE session_id = ? ORDER BY created_at",
    )?;
    let decisions: Vec<(String, String)> = stmt
        .query_map([session_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    if !decisions.is_empty() {
        has_activity = true;
        println!("\n{} Decisions ({})", "◆".cyan(), decisions.len());
        for (topic, decision) in &decisions {
            println!("  • {}: {}", topic.bold(), decision);
        }
    }

    // Get tasks added
    let mut stmt = conn.prepare(
        "SELECT description, priority, status FROM tasks WHERE session_id = ? ORDER BY created_at",
    )?;
    let tasks: Vec<(String, String, String)> = stmt
        .query_map([session_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if !tasks.is_empty() {
        has_activity = true;
        println!("\n{} Tasks Added ({})", "◆".cyan(), tasks.len());
        for (desc, priority, status) in &tasks {
            let priority_indicator = match priority.as_str() {
                "urgent" => "[!!!]".red(),
                "high" => "[!]".yellow(),
                _ => "".white(),
            };
            let status_indicator = match status.as_str() {
                "completed" => "✓".green(),
                "in_progress" => "◐".yellow(),
                _ => "○".white(),
            };
            println!("  {} {} {}", status_indicator, desc, priority_indicator);
        }
    }

    // Get blockers
    let mut stmt = conn.prepare(
        "SELECT description, status FROM blockers WHERE session_id = ? ORDER BY created_at",
    )?;
    let blockers: Vec<(String, String)> = stmt
        .query_map([session_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    if !blockers.is_empty() {
        has_activity = true;
        println!("\n{} Blockers ({})", "◆".cyan(), blockers.len());
        for (desc, status) in &blockers {
            let indicator = if status == "resolved" {
                "✓".green()
            } else {
                "✗".red()
            };
            println!("  {} {}", indicator, desc);
        }
    }

    // Get notes
    let mut stmt = conn.prepare(
        "SELECT category, title FROM context_notes WHERE session_id = ? ORDER BY created_at",
    )?;
    let notes: Vec<(String, String)> = stmt
        .query_map([session_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    if !notes.is_empty() {
        has_activity = true;
        println!("\n{} Notes ({})", "◆".cyan(), notes.len());
        for (category, title) in &notes {
            println!("  • [{}] {}", category, title);
        }
    }

    // Get questions
    let mut stmt = conn.prepare(
        "SELECT question, status FROM questions WHERE session_id = ? ORDER BY created_at",
    )?;
    let questions: Vec<(String, String)> = stmt
        .query_map([session_id], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(|r| r.ok())
        .collect();

    if !questions.is_empty() {
        has_activity = true;
        println!("\n{} Questions ({})", "◆".cyan(), questions.len());
        for (question, status) in &questions {
            let indicator = if status == "answered" {
                "✓".green()
            } else {
                "?".yellow()
            };
            println!("  {} {}", indicator, question);
        }
    }

    if !has_activity {
        println!(
            "\n  {} No decisions, tasks, or blockers logged this session.",
            "ℹ".blue()
        );
        println!(
            "  Tip: Use 'proj log decision', 'proj task add', 'proj log blocker' during sessions."
        );
    }

    println!("{}", "─".repeat(50));

    Ok(())
}

/// Display session review hints - check for potentially missed logging
fn display_session_hints(
    conn: &rusqlite::Connection,
    session_id: i64,
    session_started_at: &str,
) -> Result<()> {
    // Count logged items
    let decision_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM decisions WHERE session_id = ?",
        [session_id],
        |row| row.get(0),
    )?;
    let task_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE session_id = ?",
        [session_id],
        |row| row.get(0),
    )?;
    let blocker_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM blockers WHERE session_id = ?",
        [session_id],
        |row| row.get(0),
    )?;

    // Count git commits since session start
    let commit_count = git::get_commit_count_since(conn, session_started_at).unwrap_or(0);

    println!();
    println!("{}", "Session Review:".bold());
    println!(
        "  Logged: {} decisions, {} tasks, {} blockers",
        decision_count, task_count, blocker_count
    );
    if commit_count > 0 {
        println!("  Git: {} commits since session start", commit_count);
    }

    // Generate hints
    let mut hints = Vec::new();

    if commit_count > 0 && decision_count == 0 {
        hints.push(format!(
            "{} commits were made but no decisions logged. Consider logging key decisions.",
            commit_count
        ));
    }

    if commit_count > 3 && task_count == 0 {
        hints.push(
            "Several commits suggest meaningful work. Consider adding tasks for follow-up items."
                .to_string(),
        );
    }

    // Check session duration
    if let Ok(started) =
        chrono::NaiveDateTime::parse_from_str(session_started_at, "%Y-%m-%d %H:%M:%S")
    {
        let now = chrono::Utc::now().naive_utc();
        let duration = now - started;
        if duration.num_hours() >= 1 && decision_count == 0 && task_count == 0 && blocker_count == 0
        {
            hints.push(
                "Session lasted 1+ hours with no activity logged. Consider reviewing what was accomplished.".to_string()
            );
        }
    }

    if !hints.is_empty() {
        println!();
        println!("  {}:", "Hints".yellow());
        for hint in &hints {
            println!("    {}", hint);
        }
    }

    Ok(())
}

/// Build a structured JSON summary of session activity
fn build_structured_summary(
    conn: &rusqlite::Connection,
    session_id: i64,
    summary: &str,
) -> Result<String> {
    // Gather decisions
    let mut stmt = conn.prepare(
        "SELECT topic, decision FROM decisions WHERE session_id = ? ORDER BY created_at",
    )?;
    let decisions: Vec<String> = stmt
        .query_map([session_id], |row| {
            Ok(format!(
                "{}: {}",
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Gather tasks created
    let mut stmt = conn.prepare(
        "SELECT description FROM tasks WHERE session_id = ? AND status != 'completed' ORDER BY created_at",
    )?;
    let tasks_created: Vec<String> = stmt
        .query_map([session_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    // Gather tasks completed (any task marked completed during this session period)
    let mut stmt = conn.prepare(
        "SELECT description FROM tasks WHERE session_id = ? AND status = 'completed' ORDER BY created_at",
    )?;
    let tasks_completed: Vec<String> = stmt
        .query_map([session_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    // Gather blockers
    let mut stmt =
        conn.prepare("SELECT description FROM blockers WHERE session_id = ? ORDER BY created_at")?;
    let blockers: Vec<String> = stmt
        .query_map([session_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    // Gather notes
    let mut stmt = conn.prepare(
        "SELECT category, title FROM context_notes WHERE session_id = ? ORDER BY created_at",
    )?;
    let notes: Vec<String> = stmt
        .query_map([session_id], |row| {
            Ok(format!(
                "{}: {}",
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Get session start time for git commit query
    let started_at: String = conn.query_row(
        "SELECT started_at FROM sessions WHERE session_id = ?",
        [session_id],
        |row| row.get(0),
    )?;

    // Gather git commits since session start
    let git_commits_data = git::get_commits_since(conn, &started_at)?;
    let git_commits: Vec<String> = git_commits_data
        .iter()
        .map(|c| format!("{}: {}", c.short_hash, c.message))
        .collect();

    // Get files from git diff since session start
    let files_touched = get_files_touched_since(&started_at);

    // Build JSON
    let structured = serde_json::json!({
        "summary": summary,
        "decisions": decisions,
        "tasks_created": tasks_created,
        "tasks_completed": tasks_completed,
        "blockers": blockers,
        "notes": notes,
        "git_commits": git_commits,
        "files_touched": files_touched,
    });

    Ok(structured.to_string())
}

/// Get list of files changed since a given datetime via git
fn get_files_touched_since(since: &str) -> Vec<String> {
    let project_root = match crate::paths::get_project_root() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    if !project_root.join(".git").exists() {
        return Vec::new();
    }

    // Use git diff to find files changed
    let output = Command::new("git")
        .args(["diff", "--name-only", &format!("--since={}", since), "HEAD"])
        .current_dir(&project_root)
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect(),
        _ => {
            // Fallback: try git log --name-only
            let output = Command::new("git")
                .args([
                    "log",
                    "--name-only",
                    "--pretty=format:",
                    &format!("--since={}", since),
                ])
                .current_dir(&project_root)
                .output();

            match output {
                Ok(o) if o.status.success() => {
                    let mut files: Vec<String> = String::from_utf8_lossy(&o.stdout)
                        .lines()
                        .filter(|l| !l.is_empty())
                        .map(|l| l.to_string())
                        .collect();
                    files.sort();
                    files.dedup();
                    files
                }
                _ => Vec::new(),
            }
        }
    }
}

/// Handle auto-commit on session end
fn handle_auto_commit(summary: &str) -> Result<()> {
    let config = ProjectConfig::load()?;

    if !config.auto_commit {
        return Ok(());
    }

    let commit_message = format!("[proj] {}", summary);
    crate::commit::auto_commit(&commit_message, &config)?;

    Ok(())
}

/// List recent sessions
fn cmd_list(conn: &rusqlite::Connection) -> Result<()> {
    let sessions = get_recent_sessions(conn, 10)?;

    if sessions.is_empty() {
        println!("No sessions found.");
        return Ok(());
    }

    println!("{}", "Recent Sessions:".bold());
    println!("{}", "-".repeat(60));

    for session in sessions {
        let status_indicator = match session.status.as_str() {
            "active" => "(active)".green(),
            "completed" => "(completed)".white(),
            "abandoned" => "(abandoned)".yellow(),
            _ => session.status.clone().white(),
        };

        let date_str = if session.status == "active" {
            format!("started {}", session.started_at.format("%Y-%m-%d %H:%M"))
        } else {
            session
                .ended_at
                .map(|e| format!("ended {}", e.format("%Y-%m-%d %H:%M")))
                .unwrap_or_else(|| session.started_at.format("%Y-%m-%d %H:%M").to_string())
        };

        println!(
            "#{:<4} {} {}",
            session.session_id, date_str, status_indicator
        );

        if let Some(summary) = &session.summary {
            println!("      {}", summary);
        }
    }

    Ok(())
}
