// Log commands - decision, note, blocker, question

use anyhow::{Context, Result};
use colored::Colorize;
use rusqlite::Connection;

use crate::cli::{LogCommands, LogSubcommand};
use crate::database::open_database;
use crate::paths::get_tracking_db_path;
use crate::session::get_or_create_session;

pub fn run(cmd: LogCommands) -> Result<()> {
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    // Get or create session for all log operations
    let session = get_or_create_session(&conn)?;

    match cmd.command {
        LogSubcommand::Decision {
            topic,
            decision,
            rationale,
        } => cmd_log_decision(
            &conn,
            session.session_id,
            &topic,
            &decision,
            rationale.as_deref(),
        ),
        LogSubcommand::Note {
            category,
            title,
            content,
        } => cmd_log_note(&conn, session.session_id, &category, &title, &content),
        LogSubcommand::Blocker { description } => {
            cmd_log_blocker(&conn, session.session_id, &description)
        }
        LogSubcommand::Question { question, context } => {
            cmd_log_question(&conn, session.session_id, &question, context.as_deref())
        }
    }
}

/// Log a decision
fn cmd_log_decision(
    conn: &Connection,
    session_id: i64,
    topic: &str,
    decision: &str,
    rationale: Option<&str>,
) -> Result<()> {
    // Insert decision
    conn.execute(
        "INSERT INTO decisions (session_id, topic, decision, rationale, status) VALUES (?1, ?2, ?3, ?4, 'active')",
        rusqlite::params![session_id, topic, decision, rationale],
    )?;

    let decision_id = conn.last_insert_rowid();

    // Insert into activity_log
    let summary = format!("Decision: {} - {}", topic, truncate(decision, 50));
    insert_activity_log(conn, session_id, "decision", decision_id, &summary)?;

    // Update FTS index
    let fts_content = format!("{} {} {}", topic, decision, rationale.unwrap_or(""));
    insert_fts_entry(conn, &fts_content, "decisions", decision_id)?;

    println!(
        "{} Logged decision #{}: {}",
        "✓".green(),
        decision_id,
        topic
    );
    Ok(())
}

/// Log a context note
fn cmd_log_note(
    conn: &Connection,
    session_id: i64,
    category: &str,
    title: &str,
    content: &str,
) -> Result<()> {
    // Validate category
    let valid_categories = ["goal", "constraint", "assumption", "requirement", "note"];
    if !valid_categories.contains(&category) {
        println!(
            "{} Invalid category '{}'. Valid categories: {}",
            "!".yellow(),
            category,
            valid_categories.join(", ")
        );
        return Ok(());
    }

    // Insert note
    conn.execute(
        "INSERT INTO context_notes (session_id, category, title, content, status) VALUES (?1, ?2, ?3, ?4, 'active')",
        rusqlite::params![session_id, category, title, content],
    )?;

    let note_id = conn.last_insert_rowid();

    // Insert into activity_log
    let summary = format!("Note [{}]: {} - {}", category, title, truncate(content, 40));
    insert_activity_log(conn, session_id, "note", note_id, &summary)?;

    // Update FTS index
    let fts_content = format!("{} {} {}", category, title, content);
    insert_fts_entry(conn, &fts_content, "context_notes", note_id)?;

    println!(
        "{} Logged note #{} [{}]: {}",
        "✓".green(),
        note_id,
        category,
        title
    );
    Ok(())
}

/// Log a blocker
fn cmd_log_blocker(conn: &Connection, session_id: i64, description: &str) -> Result<()> {
    // Insert blocker
    conn.execute(
        "INSERT INTO blockers (session_id, description, status) VALUES (?1, ?2, 'active')",
        rusqlite::params![session_id, description],
    )?;

    let blocker_id = conn.last_insert_rowid();

    // Insert into activity_log
    let summary = format!("Blocker: {}", truncate(description, 60));
    insert_activity_log(conn, session_id, "blocker", blocker_id, &summary)?;

    // Update FTS index
    insert_fts_entry(conn, description, "blockers", blocker_id)?;

    println!(
        "{} Logged blocker #{}: {}",
        "✗".red(),
        blocker_id,
        truncate(description, 50)
    );
    Ok(())
}

/// Log a question
fn cmd_log_question(
    conn: &Connection,
    session_id: i64,
    question: &str,
    context: Option<&str>,
) -> Result<()> {
    // Insert question
    conn.execute(
        "INSERT INTO questions (session_id, question, context, status) VALUES (?1, ?2, ?3, 'open')",
        rusqlite::params![session_id, question, context],
    )?;

    let question_id = conn.last_insert_rowid();

    // Insert into activity_log
    let summary = format!("Question: {}", truncate(question, 60));
    insert_activity_log(conn, session_id, "question", question_id, &summary)?;

    // Update FTS index
    let fts_content = format!("{} {}", question, context.unwrap_or(""));
    insert_fts_entry(conn, &fts_content, "questions", question_id)?;

    println!(
        "{} Logged question #{}: {}",
        "?".cyan(),
        question_id,
        truncate(question, 50)
    );
    Ok(())
}

/// Insert an entry into the activity log
fn insert_activity_log(
    conn: &Connection,
    session_id: i64,
    action_type: &str,
    action_id: i64,
    summary: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO activity_log (session_id, action_type, action_id, summary) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![session_id, action_type, action_id, summary],
    )?;
    Ok(())
}

/// Insert an entry into the FTS index
fn insert_fts_entry(
    conn: &Connection,
    content: &str,
    table_name: &str,
    record_id: i64,
) -> Result<()> {
    conn.execute(
        "INSERT INTO tracking_fts (content, table_name, record_id) VALUES (?1, ?2, ?3)",
        rusqlite::params![content, table_name, record_id],
    )?;
    Ok(())
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
