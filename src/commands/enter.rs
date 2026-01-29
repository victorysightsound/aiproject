// Enter command - silent session start for shell hook integration
//
// Behavior:
// - If active session exists: exit silently (no output)
// - If no active session (or stale session auto-closed): show full context
//
// This enables autonomous tracking via shell hooks without cluttering output.

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use colored::Colorize;

use crate::commands::status;
use crate::commands::update_check;
use crate::database::open_database;
use crate::paths::get_tracking_db_path;
use crate::session::{get_active_session, get_or_create_session_with_info};

/// Stale session threshold in hours (matches session.rs)
const STALE_SESSION_HOURS: i64 = 8;

pub fn run() -> Result<()> {
    // Open the tracking database
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    // First, check if there's an active session that's NOT stale
    // We do this check separately to avoid creating a session unnecessarily
    if let Some(session) = get_active_session(&conn)? {
        let now = Utc::now();
        let session_age = now - session.started_at;

        if session_age <= Duration::hours(STALE_SESSION_HOURS) {
            // Active, non-stale session exists - exit silently
            return Ok(());
        }
    }

    // No active session, or session is stale - this will create a new session
    let session_result = get_or_create_session_with_info(&conn)?;

    // If a stale session was auto-closed, notify the user
    if let Some(closed) = session_result.auto_closed_session {
        println!(
            "{} Previous session #{} was stale (8+ hours). Auto-closed.",
            "⚠".yellow(),
            closed.session_id
        );
        println!(
            "{} Started new session #{}",
            "✓".green(),
            session_result.session.session_id
        );
        println!();
    }

    // Show full context for the new session (reuse status command logic)
    // Pass full=true to ensure full context is shown
    status::run(false, false, true)?;

    // Check for updates (cached, runs at most once per day)
    update_check::check_and_notify();

    Ok(())
}
