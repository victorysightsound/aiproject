// Status command - tiered context output with first-run enforcement

use anyhow::{Context, Result};
use colored::Colorize;
use rusqlite::Connection;

use crate::config::ProjectConfig;
use crate::database::open_database;
use crate::models::{Blocker, Decision, Question, Task};
use crate::paths::{get_config_path, get_tracking_db_path};
use crate::session::{get_active_session, get_or_create_session, get_last_completed_session, mark_full_context_shown};

/// Status tier levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusTier {
    /// Tier 0: Micro context (~10 tokens)
    Micro,
    /// Tier 1: Minimal context (~50 tokens)
    Minimal,
    /// Tier 2: Working context (~200 tokens)
    Working,
    /// Tier 3: Full context (~500+ tokens)
    Full,
}

pub fn run(quiet: bool, verbose: bool, full: bool) -> Result<()> {
    // Determine requested tier from flags
    let requested_tier = if quiet {
        StatusTier::Micro
    } else if full {
        StatusTier::Full
    } else if verbose {
        StatusTier::Working
    } else {
        StatusTier::Minimal
    };

    // Open the tracking database
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    // Get or create session
    let session = get_or_create_session(&conn)?;

    // First-run enforcement: if full_context_shown is false, force Full tier
    let effective_tier = if !session.full_context_shown {
        StatusTier::Full
    } else {
        requested_tier
    };

    // Load project config
    let config = load_config()?;

    // Output based on tier
    match effective_tier {
        StatusTier::Micro => output_tier0(&conn, &config, &session)?,
        StatusTier::Minimal => output_tier1(&conn, &config, &session)?,
        StatusTier::Working => output_tier2(&conn, &config, &session)?,
        StatusTier::Full => {
            output_tier3(&conn, &config, &session)?;
            // Mark that full context has been shown this session
            mark_full_context_shown(&conn, session.session_id)?;
        }
    }

    Ok(())
}

/// Load project configuration
fn load_config() -> Result<ProjectConfig> {
    let config_path = get_config_path()?;
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config at {:?}", config_path))?;
    let config: ProjectConfig = serde_json::from_str(&content)
        .with_context(|| "Failed to parse config.json")?;
    Ok(config)
}

/// Tier 0: Micro context (~10 tokens)
fn output_tier0(conn: &Connection, config: &ProjectConfig, session: &crate::models::Session) -> Result<()> {
    let mut parts = vec![format!("{} [#{}]", config.name, session.session_id)];

    // Add current task if any
    if let Some(task) = get_priority_task(conn)? {
        parts.push(format!("Task: {}", truncate(&task.description, 30)));
    }

    // Add blocker count if any
    let blocker_count = get_active_blocker_count(conn)?;
    if blocker_count > 0 {
        parts.push(format!("{} blocker(s)", blocker_count));
    }

    println!("{}", parts.join(" | "));
    Ok(())
}

/// Tier 1: Minimal context (~50 tokens)
fn output_tier1(conn: &Connection, config: &ProjectConfig, session: &crate::models::Session) -> Result<()> {
    // Header
    println!("{}", "=".repeat(60));
    println!("PROJECT: {}", config.name.bold());
    println!("{}", "=".repeat(60));
    println!();
    println!("[Session #{} active]", session.session_id);
    println!();

    // Last session summary
    if let Some(last) = get_last_completed_session(conn)? {
        if let Some(ended) = &last.ended_at {
            println!("Last session ({}):", ended.format("%Y-%m-%d %H:%M"));
            if let Some(summary) = &last.summary {
                println!("  {}", summary);
            }
            println!();
        }
    }

    // Active blockers
    let blockers = get_active_blockers(conn)?;
    if !blockers.is_empty() {
        println!("Blockers ({}):", blockers.len());
        for b in &blockers {
            println!("  {} {}", "✗".red(), b.description);
        }
        println!();
    }

    // Priority tasks
    let tasks = get_priority_tasks(conn, 3)?;
    if !tasks.is_empty() {
        println!("Priority Tasks:");
        for t in &tasks {
            let status_icon = match t.status.as_str() {
                "in_progress" => "◐",
                "blocked" => "✗",
                _ => "○",
            };
            let priority_marker = if t.priority == "high" || t.priority == "urgent" { " [!]" } else { "" };
            println!("  {} [{}] {}{}", status_icon, t.task_id, t.description, priority_marker);
        }
    }

    Ok(())
}

/// Tier 2: Working context (~200 tokens)
fn output_tier2(conn: &Connection, config: &ProjectConfig, session: &crate::models::Session) -> Result<()> {
    // Start with Tier 1 content
    output_tier1(conn, config, session)?;

    println!();
    println!("{}", "-".repeat(40));

    // Type and description
    if let Some(desc) = &config.description {
        println!("Type: {} | {}", config.project_type, desc);
    } else {
        println!("Type: {}", config.project_type);
    }
    println!();

    // All active tasks (not just priority)
    let tasks = get_active_tasks(conn)?;
    if !tasks.is_empty() {
        println!("All Active Tasks ({}):", tasks.len());
        for t in &tasks {
            let status_icon = match t.status.as_str() {
                "in_progress" => "◐".yellow(),
                "blocked" => "✗".red(),
                "pending" => "○".white(),
                _ => "○".white(),
            };
            println!("  {} [{}] {} ({})", status_icon, t.task_id, t.description, t.priority);
        }
        println!();
    }

    // Recent decisions
    let decisions = get_recent_decisions(conn, 5)?;
    if !decisions.is_empty() {
        println!("Recent Decisions:");
        for d in &decisions {
            println!("  • {}: {}", d.topic.bold(), truncate(&d.decision, 50));
        }
        println!();
    }

    // Open questions
    let questions = get_open_questions(conn)?;
    if !questions.is_empty() {
        println!("Open Questions ({}):", questions.len());
        for q in &questions {
            println!("  ? {}", truncate(&q.question, 60));
        }
    }

    Ok(())
}

/// Tier 3: Full context (~500+ tokens)
fn output_tier3(conn: &Connection, config: &ProjectConfig, session: &crate::models::Session) -> Result<()> {
    println!("{}", "=".repeat(60));
    println!("{}", "FULL PROJECT CONTEXT".bold());
    println!("{}", "=".repeat(60));
    println!();

    // Project info
    println!("Project: {}", config.name.bold());
    println!("Type: {}", config.project_type);
    if let Some(desc) = &config.description {
        println!("Description: {}", desc);
    }
    println!("Schema Version: {}", config.schema_version);
    println!();

    // Current session
    println!("{}", "-".repeat(40));
    println!("CURRENT SESSION #{}", session.session_id);
    println!("Started: {}", session.started_at.format("%Y-%m-%d %H:%M:%S"));
    println!();

    // Last session summary
    if let Some(last) = get_last_completed_session(conn)? {
        println!("{}", "-".repeat(40));
        println!("LAST SESSION (#{}):", last.session_id);
        if let Some(ended) = &last.ended_at {
            println!("Ended: {}", ended.format("%Y-%m-%d %H:%M:%S"));
        }
        if let Some(summary) = &last.summary {
            println!("Summary: {}", summary);
        }
        println!();
    }

    // Active blockers
    println!("{}", "-".repeat(40));
    println!("BLOCKERS:");
    let blockers = get_active_blockers(conn)?;
    if blockers.is_empty() {
        println!("  (none)");
    } else {
        for b in &blockers {
            println!("  {} {} (created {})", "✗".red(), b.description, b.created_at.format("%Y-%m-%d"));
        }
    }
    println!();

    // All active tasks
    println!("{}", "-".repeat(40));
    println!("TASKS:");
    let tasks = get_active_tasks(conn)?;
    if tasks.is_empty() {
        println!("  (none)");
    } else {
        for t in &tasks {
            let status_icon = match t.status.as_str() {
                "in_progress" => "◐".yellow(),
                "blocked" => "✗".red(),
                "pending" => "○".white(),
                _ => "○".white(),
            };
            println!("  {} [{}] {} [{}] {}",
                status_icon, t.task_id, t.description, t.priority,
                if t.blocked_by.is_some() { "(blocked)" } else { "" }
            );
            if let Some(notes) = &t.notes {
                println!("       Notes: {}", notes);
            }
        }
    }
    println!();

    // Recent decisions
    println!("{}", "-".repeat(40));
    println!("RECENT DECISIONS:");
    let decisions = get_recent_decisions(conn, 10)?;
    if decisions.is_empty() {
        println!("  (none)");
    } else {
        for d in &decisions {
            println!("  • {} ({})", d.topic.bold(), d.created_at.format("%Y-%m-%d"));
            println!("    Decision: {}", d.decision);
            if let Some(rationale) = &d.rationale {
                println!("    Rationale: {}", rationale);
            }
        }
    }
    println!();

    // Open questions
    println!("{}", "-".repeat(40));
    println!("OPEN QUESTIONS:");
    let questions = get_open_questions(conn)?;
    if questions.is_empty() {
        println!("  (none)");
    } else {
        for q in &questions {
            println!("  ? {} ({})", q.question, q.created_at.format("%Y-%m-%d"));
            if let Some(ctx) = &q.context {
                println!("    Context: {}", ctx);
            }
        }
    }
    println!();

    // Context notes by category
    println!("{}", "-".repeat(40));
    println!("CONTEXT NOTES:");
    let notes = get_active_context_notes(conn)?;
    if notes.is_empty() {
        println!("  (none)");
    } else {
        let mut current_category = String::new();
        for n in &notes {
            if n.category != current_category {
                current_category = n.category.clone();
                println!();
                println!("  [{}]", current_category.to_uppercase());
            }
            println!("    • {}: {}", n.title, truncate(&n.content, 60));
        }
    }
    println!();

    // Recent sessions list
    println!("{}", "-".repeat(40));
    println!("RECENT SESSIONS:");
    let sessions = crate::session::get_recent_sessions(conn, 5)?;
    for s in &sessions {
        let status_indicator = if s.status == "active" { "(active)" } else { "" };
        println!("  #{} {} - {} {}",
            s.session_id,
            s.started_at.format("%Y-%m-%d %H:%M"),
            s.summary.as_ref().map(|s| truncate(s, 40)).unwrap_or_else(|| "(no summary)".to_string()),
            status_indicator
        );
    }

    Ok(())
}

// Helper functions for database queries

fn get_priority_task(conn: &Connection) -> Result<Option<Task>> {
    let mut stmt = conn.prepare(
        "SELECT task_id, session_id, created_at, completed_at, description, status, priority, blocked_by, parent_task_id, notes
         FROM tasks
         WHERE status IN ('pending', 'in_progress')
         ORDER BY
           CASE priority WHEN 'urgent' THEN 1 WHEN 'high' THEN 2 WHEN 'normal' THEN 3 ELSE 4 END,
           created_at
         LIMIT 1"
    )?;

    let task = stmt.query_row([], |row| {
        Ok(Task {
            task_id: row.get(0)?,
            session_id: row.get(1)?,
            created_at: parse_datetime(row.get::<_, String>(2)?),
            completed_at: row.get::<_, Option<String>>(3)?.map(parse_datetime),
            description: row.get(4)?,
            status: row.get(5)?,
            priority: row.get(6)?,
            blocked_by: row.get(7)?,
            parent_task_id: row.get(8)?,
            notes: row.get(9)?,
        })
    });

    match task {
        Ok(t) => Ok(Some(t)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn get_priority_tasks(conn: &Connection, limit: usize) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT task_id, session_id, created_at, completed_at, description, status, priority, blocked_by, parent_task_id, notes
         FROM tasks
         WHERE status IN ('pending', 'in_progress', 'blocked')
         ORDER BY
           CASE priority WHEN 'urgent' THEN 1 WHEN 'high' THEN 2 WHEN 'normal' THEN 3 ELSE 4 END,
           created_at
         LIMIT ?1"
    )?;

    let tasks = stmt.query_map([limit as i64], |row| {
        Ok(Task {
            task_id: row.get(0)?,
            session_id: row.get(1)?,
            created_at: parse_datetime(row.get::<_, String>(2)?),
            completed_at: row.get::<_, Option<String>>(3)?.map(parse_datetime),
            description: row.get(4)?,
            status: row.get(5)?,
            priority: row.get(6)?,
            blocked_by: row.get(7)?,
            parent_task_id: row.get(8)?,
            notes: row.get(9)?,
        })
    })?;

    tasks.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

fn get_active_tasks(conn: &Connection) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT task_id, session_id, created_at, completed_at, description, status, priority, blocked_by, parent_task_id, notes
         FROM tasks
         WHERE status NOT IN ('completed', 'cancelled')
         ORDER BY
           CASE priority WHEN 'urgent' THEN 1 WHEN 'high' THEN 2 WHEN 'normal' THEN 3 ELSE 4 END,
           created_at"
    )?;

    let tasks = stmt.query_map([], |row| {
        Ok(Task {
            task_id: row.get(0)?,
            session_id: row.get(1)?,
            created_at: parse_datetime(row.get::<_, String>(2)?),
            completed_at: row.get::<_, Option<String>>(3)?.map(parse_datetime),
            description: row.get(4)?,
            status: row.get(5)?,
            priority: row.get(6)?,
            blocked_by: row.get(7)?,
            parent_task_id: row.get(8)?,
            notes: row.get(9)?,
        })
    })?;

    tasks.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

fn get_active_blocker_count(conn: &Connection) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM blockers WHERE status = 'active'",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

fn get_active_blockers(conn: &Connection) -> Result<Vec<Blocker>> {
    let mut stmt = conn.prepare(
        "SELECT blocker_id, session_id, created_at, resolved_at, description, status, resolution, related_task_id
         FROM blockers
         WHERE status = 'active'
         ORDER BY created_at DESC"
    )?;

    let blockers = stmt.query_map([], |row| {
        Ok(Blocker {
            blocker_id: row.get(0)?,
            session_id: row.get(1)?,
            created_at: parse_datetime(row.get::<_, String>(2)?),
            resolved_at: row.get::<_, Option<String>>(3)?.map(parse_datetime),
            description: row.get(4)?,
            status: row.get(5)?,
            resolution: row.get(6)?,
            related_task_id: row.get(7)?,
        })
    })?;

    blockers.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

fn get_recent_decisions(conn: &Connection, limit: usize) -> Result<Vec<Decision>> {
    let mut stmt = conn.prepare(
        "SELECT decision_id, session_id, created_at, topic, decision, rationale, alternatives, status, superseded_by
         FROM decisions
         WHERE status = 'active'
         ORDER BY created_at DESC
         LIMIT ?1"
    )?;

    let decisions = stmt.query_map([limit as i64], |row| {
        Ok(Decision {
            decision_id: row.get(0)?,
            session_id: row.get(1)?,
            created_at: parse_datetime(row.get::<_, String>(2)?),
            topic: row.get(3)?,
            decision: row.get(4)?,
            rationale: row.get(5)?,
            alternatives: row.get(6)?,
            status: row.get(7)?,
            superseded_by: row.get(8)?,
        })
    })?;

    decisions.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

fn get_open_questions(conn: &Connection) -> Result<Vec<Question>> {
    let mut stmt = conn.prepare(
        "SELECT question_id, session_id, created_at, answered_at, question, context, answer, status
         FROM questions
         WHERE status = 'open'
         ORDER BY created_at DESC"
    )?;

    let questions = stmt.query_map([], |row| {
        Ok(Question {
            question_id: row.get(0)?,
            session_id: row.get(1)?,
            created_at: parse_datetime(row.get::<_, String>(2)?),
            answered_at: row.get::<_, Option<String>>(3)?.map(parse_datetime),
            question: row.get(4)?,
            context: row.get(5)?,
            answer: row.get(6)?,
            status: row.get(7)?,
        })
    })?;

    questions.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

fn get_active_context_notes(conn: &Connection) -> Result<Vec<crate::models::ContextNote>> {
    let mut stmt = conn.prepare(
        "SELECT note_id, session_id, created_at, updated_at, category, title, content, status
         FROM context_notes
         WHERE status = 'active'
         ORDER BY category, created_at"
    )?;

    let notes = stmt.query_map([], |row| {
        Ok(crate::models::ContextNote {
            note_id: row.get(0)?,
            session_id: row.get(1)?,
            created_at: parse_datetime(row.get::<_, String>(2)?),
            updated_at: parse_datetime(row.get::<_, String>(3)?),
            category: row.get(4)?,
            title: row.get(5)?,
            content: row.get(6)?,
            status: row.get(7)?,
        })
    })?;

    notes.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

/// Parse datetime string from SQLite
fn parse_datetime(s: String) -> chrono::DateTime<chrono::Utc> {
    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|_| chrono::Utc::now())
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
