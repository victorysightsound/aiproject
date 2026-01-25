// Session commands - start, end, list

use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::cli::{SessionCommands, SessionSubcommand};
use crate::database::open_database;
use crate::paths::get_tracking_db_path;
use crate::session::{create_session, end_session, get_active_session, get_recent_sessions};

pub fn run(cmd: SessionCommands) -> Result<()> {
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    match cmd.command {
        SessionSubcommand::Start => cmd_start(&conn),
        SessionSubcommand::End { summary } => cmd_end(&conn, &summary),
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
fn cmd_end(conn: &rusqlite::Connection, summary: &str) -> Result<()> {
    // Get active session
    let session = match get_active_session(conn)? {
        Some(s) => s,
        None => bail!("No active session to end"),
    };

    // End the session
    end_session(conn, session.session_id, summary)?;

    println!(
        "{} Session #{} ended. Summary: {}",
        "✓".green(),
        session.session_id,
        summary
    );

    // TODO: Create backup if auto_backup enabled in config

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
