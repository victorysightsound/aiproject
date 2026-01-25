// Compress command - compress old sessions into summaries for token efficiency

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use dialoguer::Confirm;
use rusqlite::Connection;

use crate::database::open_database;
use crate::paths::get_tracking_db_path;

/// Session data for compression
struct SessionInfo {
    session_id: i64,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    summary: Option<String>,
    agent: Option<String>,
}

pub fn run(auto: bool) -> Result<()> {
    // Open database
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    // Default settings
    let threshold = 10; // max sessions to compress at once
    let min_age_days = 7;

    // Get sessions eligible for compression
    let sessions = get_sessions_for_compression(&conn, threshold, min_age_days)?;

    if sessions.is_empty() {
        println!("No sessions eligible for compression.");
        println!(
            "  (Looking for completed sessions older than {} days)",
            min_age_days
        );
        return Ok(());
    }

    println!(
        "Found {} session(s) eligible for compression:\n",
        sessions.len()
    );
    for s in &sessions {
        let summary_preview = match &s.summary {
            Some(sum) if sum.len() > 50 => format!("{}...", &sum[..50]),
            Some(sum) => sum.clone(),
            None => "No summary".to_string(),
        };
        let ended_str = s
            .ended_at
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "?".to_string());
        println!("  #{} ({}): {}", s.session_id, ended_str, summary_preview);
    }

    // Confirm unless auto mode
    if !auto {
        let confirm = Confirm::new()
            .with_prompt(format!(
                "Compress these {} sessions into a single summary?",
                sessions.len()
            ))
            .default(true)
            .interact()?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Build combined content for compression
    let mut combined_summaries = Vec::new();
    let mut original_tokens = 0;

    for s in &sessions {
        let agent = s.agent.as_deref().unwrap_or("unknown");
        let started = s.started_at.format("%Y-%m-%d %H:%M").to_string();
        let ended = s
            .ended_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "?".to_string());
        let summary = s.summary.as_deref().unwrap_or("No summary");

        let session_text = format!(
            "Session #{} ({}, {} - {}): {}",
            s.session_id, agent, started, ended, summary
        );
        original_tokens += estimate_tokens(&session_text);
        combined_summaries.push(session_text);

        // Get decisions from this session
        let decisions = get_session_decisions(&conn, s.session_id)?;
        for (topic, decision) in decisions {
            let decision_text = format!("  Decision [{}]: {}", topic, decision);
            original_tokens += estimate_tokens(&decision_text);
            combined_summaries.push(decision_text);
        }
    }

    // Generate compressed summary
    let session_ids: Vec<i64> = sessions.iter().map(|s| s.session_id).collect();
    let first_started = sessions
        .first()
        .map(|s| s.started_at.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    let last_ended = sessions
        .last()
        .and_then(|s| s.ended_at)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let date_range = format!("{} to {}", first_started, last_ended);

    let mut compressed = format!(
        "[Compressed: Sessions #{}-#{}, {}]\n",
        session_ids.first().unwrap_or(&0),
        session_ids.last().unwrap_or(&0),
        date_range
    );
    compressed.push_str(&format!("Covered {} work sessions. ", sessions.len()));

    // Extract key summaries
    let summaries: Vec<&str> = sessions
        .iter()
        .filter_map(|s| s.summary.as_deref())
        .collect();

    if !summaries.is_empty() {
        compressed.push_str("Key accomplishments: ");
        let preview: Vec<&str> = summaries.iter().take(3).copied().collect();
        compressed.push_str(&preview.join("; "));
        if summaries.len() > 3 {
            compressed.push_str(&format!(" (+{} more)", summaries.len() - 3));
        }
    }

    let compressed_tokens = estimate_tokens(&compressed);
    let savings = original_tokens.saturating_sub(compressed_tokens);
    let savings_pct = if original_tokens > 0 {
        (savings * 100) / original_tokens
    } else {
        0
    };

    println!("\nCompression result:");
    println!("  Original: ~{} tokens", original_tokens);
    println!("  Compressed: ~{} tokens", compressed_tokens);
    println!("  Savings: ~{} tokens ({}%)", savings, savings_pct);

    let preview = if compressed.len() > 200 {
        format!("{}...", &compressed[..200])
    } else {
        compressed.clone()
    };
    println!("\nCompressed summary:");
    println!("  {}", preview);

    // Save compressed summary
    save_compressed_sessions(
        &conn,
        &session_ids,
        &compressed,
        original_tokens,
        compressed_tokens,
    )?;
    println!("\nSaved compression. Original sessions preserved but marked as compressed.");

    Ok(())
}

/// Get sessions eligible for compression (completed, old enough, not already compressed)
fn get_sessions_for_compression(
    conn: &Connection,
    threshold: usize,
    min_age_days: i64,
) -> Result<Vec<SessionInfo>> {
    let cutoff = Utc::now() - Duration::days(min_age_days);
    let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

    // Get sessions not already compressed
    // Note: SQLite json_each requires JSON extension; we use a simpler approach
    let mut stmt = conn.prepare(
        "SELECT session_id, started_at, ended_at, summary, agent
         FROM sessions
         WHERE status = 'completed'
           AND ended_at < ?
           AND session_id NOT IN (
               SELECT CAST(value AS INTEGER)
               FROM compressed_sessions, json_each(compressed_sessions.session_ids)
           )
         ORDER BY ended_at
         LIMIT ?",
    )?;

    let sessions = stmt.query_map(rusqlite::params![cutoff_str, threshold as i64], |row| {
        Ok(SessionInfo {
            session_id: row.get(0)?,
            started_at: parse_datetime(row.get::<_, String>(1)?),
            ended_at: row.get::<_, Option<String>>(2)?.map(parse_datetime),
            summary: row.get(3)?,
            agent: row.get(4)?,
        })
    })?;

    sessions
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.into())
}

/// Get decisions for a specific session
fn get_session_decisions(conn: &Connection, session_id: i64) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT topic, decision FROM decisions WHERE session_id = ?")?;

    let decisions = stmt.query_map([session_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    decisions
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.into())
}

/// Save compressed sessions to the database
fn save_compressed_sessions(
    conn: &Connection,
    session_ids: &[i64],
    summary: &str,
    original_tokens: usize,
    compressed_tokens: usize,
) -> Result<()> {
    let sessions_json = serde_json::to_string(session_ids)?;

    // Get date range from sessions
    let placeholders: String = session_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "SELECT MIN(started_at) as start, MAX(ended_at) as end
         FROM sessions WHERE session_id IN ({})",
        placeholders
    );

    let mut stmt = conn.prepare(&query)?;
    let params: Vec<&dyn rusqlite::ToSql> = session_ids
        .iter()
        .map(|id| id as &dyn rusqlite::ToSql)
        .collect();

    let (date_start, date_end): (Option<String>, Option<String>) =
        stmt.query_row(params.as_slice(), |row| Ok((row.get(0)?, row.get(1)?)))?;

    conn.execute(
        "INSERT INTO compressed_sessions
         (session_ids, date_range_start, date_range_end, compressed_summary,
          original_token_estimate, compressed_token_estimate)
         VALUES (?, ?, ?, ?, ?, ?)",
        rusqlite::params![
            sessions_json,
            date_start,
            date_end,
            summary,
            original_tokens as i64,
            compressed_tokens as i64
        ],
    )?;

    Ok(())
}

/// Rough token estimate (4 chars per token average)
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Parse datetime string from SQLite
fn parse_datetime(s: String) -> DateTime<Utc> {
    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|_| Utc::now())
}
