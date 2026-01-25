// Delta command - show only what changed since last status check

use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use sha2::{Digest, Sha256};

use crate::database::open_database;
use crate::paths::get_tracking_db_path;
use crate::session::get_or_create_session;

/// Counts of various item types for snapshot comparison
type ItemCounts = HashMap<String, i64>;

pub fn run() -> Result<()> {
    // Open database
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    // Get or create session
    let session = get_or_create_session(&conn)?;

    // Check if this is a brand new session (no prior activity)
    let is_new_session =
        !session.full_context_shown && get_session_activity_count(&conn, session.session_id)? == 0;

    if is_new_session {
        println!("New session started. Run 'proj status' for full context.");
        return Ok(());
    }

    // Get current counts
    let current_counts = get_current_counts(&conn)?;

    // Get last snapshot
    let (last_hash, last_counts) = get_last_snapshot(&conn, session.session_id, "delta")?;

    // Compute current hash
    let current_hash = compute_content_hash(&current_counts);

    // Compare hashes
    if let Some(ref lh) = last_hash {
        if lh == &current_hash {
            println!("No changes since last check.");
            return Ok(());
        }
    }

    // Calculate and display deltas
    println!(
        "Changes since last check (Session #{}):\n",
        session.session_id
    );

    let mut changes = Vec::new();
    for (key, current) in &current_counts {
        let last = last_counts.get(key).copied().unwrap_or(0);
        if *current != last {
            let diff = *current - last;
            let label = key.replace('_', " ");
            if diff > 0 {
                changes.push(format!("  + {} new {}", diff, label));
            } else {
                changes.push(format!("  - {} {} resolved/removed", diff.abs(), label));
            }
        }
    }

    if changes.is_empty() {
        println!("  (counts unchanged, but content may have changed)");
    } else {
        for c in &changes {
            println!("{}", c);
        }
    }

    // Show recent activity
    let activity = get_recent_activity(&conn, session.session_id, 5)?;
    if !activity.is_empty() {
        println!("\nRecent activity this session:");
        for a in &activity {
            let summary = truncate(&a.summary, 50);
            println!(
                "  {} [{}] {}",
                a.timestamp.format("%H:%M:%S"),
                a.action_type,
                summary
            );
        }
    }

    // Save new snapshot
    save_snapshot(
        &conn,
        session.session_id,
        "delta",
        &current_hash,
        &current_counts,
    )?;

    Ok(())
}

/// Get current counts of all tracked items
fn get_current_counts(conn: &Connection) -> Result<ItemCounts> {
    let mut counts = HashMap::new();

    // Active tasks
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE status NOT IN ('completed', 'cancelled')",
        [],
        |row| row.get(0),
    )?;
    counts.insert("active_tasks".to_string(), count);

    // Completed tasks
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM tasks WHERE status = 'completed'",
        [],
        |row| row.get(0),
    )?;
    counts.insert("completed_tasks".to_string(), count);

    // Active blockers
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM blockers WHERE status = 'active'",
        [],
        |row| row.get(0),
    )?;
    counts.insert("blockers".to_string(), count);

    // Active decisions
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM decisions WHERE status = 'active'",
        [],
        |row| row.get(0),
    )?;
    counts.insert("decisions".to_string(), count);

    // Open questions
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM questions WHERE status = 'open'",
        [],
        |row| row.get(0),
    )?;
    counts.insert("questions".to_string(), count);

    // Active notes
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM context_notes WHERE status = 'active'",
        [],
        |row| row.get(0),
    )?;
    counts.insert("notes".to_string(), count);

    Ok(counts)
}

/// Get the last context snapshot for comparison
fn get_last_snapshot(
    conn: &Connection,
    session_id: i64,
    snapshot_type: &str,
) -> Result<(Option<String>, ItemCounts)> {
    let mut stmt = conn.prepare(
        "SELECT content_hash, item_counts FROM context_snapshots
         WHERE session_id = ? AND snapshot_type = ?
         ORDER BY created_at DESC LIMIT 1",
    )?;

    let result = stmt.query_row(rusqlite::params![session_id, snapshot_type], |row| {
        let hash: String = row.get(0)?;
        let counts_json: Option<String> = row.get(1)?;
        Ok((hash, counts_json))
    });

    match result {
        Ok((hash, counts_json)) => {
            let counts: ItemCounts = match counts_json {
                Some(json) => serde_json::from_str(&json).unwrap_or_default(),
                None => HashMap::new(),
            };
            Ok((Some(hash), counts))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok((None, HashMap::new())),
        Err(e) => Err(e.into()),
    }
}

/// Save a context snapshot for delta tracking
fn save_snapshot(
    conn: &Connection,
    session_id: i64,
    snapshot_type: &str,
    content_hash: &str,
    item_counts: &ItemCounts,
) -> Result<()> {
    let counts_json = serde_json::to_string(item_counts)?;
    conn.execute(
        "INSERT INTO context_snapshots (session_id, snapshot_type, content_hash, item_counts) VALUES (?, ?, ?, ?)",
        rusqlite::params![session_id, snapshot_type, content_hash, counts_json],
    )?;
    Ok(())
}

/// Compute a content hash from item counts
fn compute_content_hash(counts: &ItemCounts) -> String {
    // Sort keys for deterministic hashing
    let mut keys: Vec<_> = counts.keys().collect();
    keys.sort();

    let mut hasher = Sha256::new();
    for key in keys {
        hasher.update(key.as_bytes());
        hasher.update(b":");
        hasher.update(counts[key].to_string().as_bytes());
        hasher.update(b";");
    }

    format!("{:x}", hasher.finalize())
}

/// Get the count of activity in a session
fn get_session_activity_count(conn: &Connection, session_id: i64) -> Result<i64> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM activity_log WHERE session_id = ?",
        [session_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Activity log entry
struct ActivityEntry {
    timestamp: DateTime<Utc>,
    action_type: String,
    summary: String,
}

/// Get recent activity for a session
fn get_recent_activity(
    conn: &Connection,
    session_id: i64,
    limit: usize,
) -> Result<Vec<ActivityEntry>> {
    let mut stmt = conn.prepare(
        "SELECT timestamp, action_type, summary FROM activity_log
         WHERE session_id = ?
         ORDER BY timestamp DESC LIMIT ?",
    )?;

    let entries = stmt.query_map(rusqlite::params![session_id, limit as i64], |row| {
        let timestamp_str: String = row.get(0)?;
        let timestamp = parse_datetime(timestamp_str);
        Ok(ActivityEntry {
            timestamp,
            action_type: row.get(1)?,
            summary: row.get(2)?,
        })
    })?;

    entries.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

/// Parse datetime string from SQLite
fn parse_datetime(s: String) -> DateTime<Utc> {
    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|_| Utc::now())
}

/// Truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
