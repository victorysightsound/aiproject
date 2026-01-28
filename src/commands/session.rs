// Session commands - start, end, list

use std::process::Command;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::Confirm;

use crate::cli::{SessionCommands, SessionSubcommand};
use crate::config::ProjectConfig;
use crate::database::open_database;
use crate::paths::{get_project_root, get_tracking_db_path};
use crate::session::{create_session, end_session, get_active_session, get_recent_sessions};

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

    // End the session
    end_session(conn, session.session_id, summary)?;

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

    // TODO: Create backup if auto_backup enabled in config

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

/// Handle auto-commit on session end
fn handle_auto_commit(summary: &str) -> Result<()> {
    // Load config
    let config = ProjectConfig::load()?;

    // Check if auto-commit is enabled
    if !config.auto_commit {
        return Ok(());
    }

    // Check if we're in a git repo
    let project_root = get_project_root()?;
    if !project_root.join(".git").exists() {
        return Ok(());
    }

    // Check if there are any changes to commit
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&project_root)
        .output()
        .with_context(|| "Failed to run git status")?;

    let has_changes = !status_output.stdout.is_empty();

    if !has_changes {
        println!("  {} No changes to commit", "ℹ".blue());
        return Ok(());
    }

    // Determine if we should commit
    let should_commit = match config.auto_commit_mode.as_str() {
        "auto" => true,
        "prompt" | _ => {
            // Check if we're in a TTY (interactive terminal)
            if atty::is(atty::Stream::Stdin) {
                Confirm::new()
                    .with_prompt("Commit changes with session summary?")
                    .default(true)
                    .interact()
                    .unwrap_or(false)
            } else {
                // Non-interactive, skip
                println!("  {} Skipping commit (non-interactive)", "ℹ".blue());
                false
            }
        }
    };

    if !should_commit {
        return Ok(());
    }

    // Stage all changes
    let add_result = Command::new("git")
        .args(["add", "-A"])
        .current_dir(&project_root)
        .output()
        .with_context(|| "Failed to run git add")?;

    if !add_result.status.success() {
        bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&add_result.stderr)
        );
    }

    // Create commit with session summary
    let commit_message = format!("[proj] {}", summary);
    let commit_result = Command::new("git")
        .args(["commit", "-m", &commit_message])
        .current_dir(&project_root)
        .output()
        .with_context(|| "Failed to run git commit")?;

    if !commit_result.status.success() {
        let stderr = String::from_utf8_lossy(&commit_result.stderr);
        // Check if it's just "nothing to commit"
        if stderr.contains("nothing to commit") {
            println!("  {} No changes to commit", "ℹ".blue());
            return Ok(());
        }
        bail!("git commit failed: {}", stderr);
    }

    println!("  {} Committed: {}", "✓".green(), commit_message);

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
