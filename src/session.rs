// Session management - get_or_create_session and related functions

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rusqlite::Connection;

use crate::models::Session;

/// Stale session threshold in hours
const STALE_SESSION_HOURS: i64 = 8;

/// Result of get_or_create_session that indicates if a stale session was closed
pub struct SessionResult {
    pub session: Session,
    pub auto_closed_session: Option<Session>,
}

/// Gets the currently active session, or creates a new one if none exists.
/// If an active session is stale (8+ hours old), it will be auto-closed.
pub fn get_or_create_session(conn: &Connection) -> Result<Session> {
    let result = get_or_create_session_with_info(conn)?;
    Ok(result.session)
}

/// Gets the currently active session with info about whether a stale session was closed
pub fn get_or_create_session_with_info(conn: &Connection) -> Result<SessionResult> {
    // Try to get active session first
    if let Some(session) = get_active_session(conn)? {
        // Check if session is stale (8+ hours since started)
        let now = Utc::now();
        let session_age = now - session.started_at;

        if session_age > Duration::hours(STALE_SESSION_HOURS) {
            // Auto-close the stale session
            auto_close_session(conn, session.session_id)?;
            let closed_session = session;

            // Create a new session
            let new_session = create_session(conn)?;

            return Ok(SessionResult {
                session: new_session,
                auto_closed_session: Some(closed_session),
            });
        }

        return Ok(SessionResult {
            session,
            auto_closed_session: None,
        });
    }

    // No active session, create a new one
    let session = create_session(conn)?;
    Ok(SessionResult {
        session,
        auto_closed_session: None,
    })
}

/// Auto-closes a stale session with a marker summary
fn auto_close_session(conn: &Connection, session_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET ended_at = datetime('now'), status = 'completed', summary = '(auto-closed)' WHERE session_id = ?1",
        [session_id],
    )?;
    Ok(())
}

/// Gets the currently active session if one exists
pub fn get_active_session(conn: &Connection) -> Result<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, started_at, ended_at, agent, summary, files_touched, status, full_context_shown, structured_summary
         FROM sessions
         WHERE status = 'active'
         ORDER BY started_at DESC
         LIMIT 1"
    )?;

    let session = stmt.query_row([], |row| {
        Ok(Session {
            session_id: row.get(0)?,
            started_at: parse_datetime(row.get::<_, String>(1)?),
            ended_at: row.get::<_, Option<String>>(2)?.map(parse_datetime),
            agent: row.get(3)?,
            summary: row.get(4)?,
            files_touched: row.get(5)?,
            status: row.get(6)?,
            full_context_shown: row.get::<_, i32>(7)? != 0,
            structured_summary: row.get(8)?,
        })
    });

    match session {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Creates a new session
pub fn create_session(conn: &Connection) -> Result<Session> {
    conn.execute(
        "INSERT INTO sessions (status, full_context_shown) VALUES ('active', 0)",
        [],
    )?;

    let session_id = conn.last_insert_rowid();

    // Fetch the created session
    let mut stmt = conn.prepare(
        "SELECT session_id, started_at, ended_at, agent, summary, files_touched, status, full_context_shown, structured_summary
         FROM sessions
         WHERE session_id = ?1"
    )?;

    let session = stmt.query_row([session_id], |row| {
        Ok(Session {
            session_id: row.get(0)?,
            started_at: parse_datetime(row.get::<_, String>(1)?),
            ended_at: row.get::<_, Option<String>>(2)?.map(parse_datetime),
            agent: row.get(3)?,
            summary: row.get(4)?,
            files_touched: row.get(5)?,
            status: row.get(6)?,
            full_context_shown: row.get::<_, i32>(7)? != 0,
            structured_summary: row.get(8)?,
        })
    })?;

    Ok(session)
}

/// Ends a session with a summary
#[allow(dead_code)]
pub fn end_session(conn: &Connection, session_id: i64, summary: &str) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET ended_at = datetime('now'), status = 'completed', summary = ?1 WHERE session_id = ?2",
        rusqlite::params![summary, session_id],
    )?;
    Ok(())
}

/// Ends a session with both plain and structured summaries
pub fn end_session_with_structured(conn: &Connection, session_id: i64, summary: &str, structured_summary: &str) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET ended_at = datetime('now'), status = 'completed', summary = ?1, structured_summary = ?2 WHERE session_id = ?3",
        rusqlite::params![summary, structured_summary, session_id],
    )?;
    Ok(())
}

/// Marks a session as having shown full context
pub fn mark_full_context_shown(conn: &Connection, session_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET full_context_shown = 1 WHERE session_id = ?1",
        [session_id],
    )?;
    Ok(())
}

/// Gets the last N completed sessions
pub fn get_recent_sessions(conn: &Connection, limit: usize) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, started_at, ended_at, agent, summary, files_touched, status, full_context_shown, structured_summary
         FROM sessions
         ORDER BY started_at DESC
         LIMIT ?1"
    )?;

    let sessions = stmt.query_map([limit as i64], |row| {
        Ok(Session {
            session_id: row.get(0)?,
            started_at: parse_datetime(row.get::<_, String>(1)?),
            ended_at: row.get::<_, Option<String>>(2)?.map(parse_datetime),
            agent: row.get(3)?,
            summary: row.get(4)?,
            files_touched: row.get(5)?,
            status: row.get(6)?,
            full_context_shown: row.get::<_, i32>(7)? != 0,
            structured_summary: row.get(8)?,
        })
    })?;

    sessions
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.into())
}

/// Gets the last completed session
pub fn get_last_completed_session(conn: &Connection) -> Result<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, started_at, ended_at, agent, summary, files_touched, status, full_context_shown, structured_summary
         FROM sessions
         WHERE status = 'completed'
         ORDER BY ended_at DESC
         LIMIT 1"
    )?;

    let session = stmt.query_row([], |row| {
        Ok(Session {
            session_id: row.get(0)?,
            started_at: parse_datetime(row.get::<_, String>(1)?),
            ended_at: row.get::<_, Option<String>>(2)?.map(parse_datetime),
            agent: row.get(3)?,
            summary: row.get(4)?,
            files_touched: row.get(5)?,
            status: row.get(6)?,
            full_context_shown: row.get::<_, i32>(7)? != 0,
            structured_summary: row.get(8)?,
        })
    });

    match session {
        Ok(s) => Ok(Some(s)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Parse datetime string from SQLite
fn parse_datetime(s: String) -> DateTime<Utc> {
    // SQLite stores as "YYYY-MM-DD HH:MM:SS"
    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|_| Utc::now())
}
