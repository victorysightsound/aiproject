// Context command - search decisions and notes

use anyhow::{Context, Result};
use colored::Colorize;
use rusqlite::Connection;

use crate::database::open_database;
use crate::git;
use crate::paths::get_tracking_db_path;

pub fn run(topic: &str, ranked: bool, recent: bool) -> Result<()> {
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    if recent {
        show_recent(&conn)
    } else if ranked {
        search_ranked(&conn, topic)
    } else {
        search_basic(&conn, topic)
    }
}

/// Basic search - search decisions, notes, and FTS
fn search_basic(conn: &Connection, topic: &str) -> Result<()> {
    println!("{}", format!("Searching for: {}", topic).bold());
    println!("{}", "=".repeat(60));

    let mut found = false;

    // Search decisions
    let decisions = search_decisions(conn, topic)?;
    if !decisions.is_empty() {
        println!();
        println!("{}", "Decisions".underline());
        for (id, topic_found, decision, rationale, created_at) in &decisions {
            println!("  #{} {} ({})", id, topic_found.bold(), created_at);
            println!("     Decision: {}", decision);
            if let Some(r) = rationale {
                println!("     Rationale: {}", r.dimmed());
            }
        }
        found = true;
    }

    // Search context notes
    let notes = search_notes(conn, topic)?;
    if !notes.is_empty() {
        println!();
        println!("{}", "Context Notes".underline());
        for (id, category, title, content, created_at) in &notes {
            println!("  #{} [{}] {} ({})", id, category, title.bold(), created_at);
            println!("     {}", truncate(content, 80));
        }
        found = true;
    }

    // Search git commits
    let git_results = git::search_git_commits(conn, topic)?;
    if !git_results.is_empty() {
        println!();
        println!("{}", "Git Commits".underline());
        for (_id, short_hash, message, committed_at) in &git_results {
            println!("  {} {} ({})", short_hash.dimmed(), message, committed_at);
        }
        found = true;
    }

    // Search FTS index
    let fts_results = search_fts(conn, topic)?;
    if !fts_results.is_empty() {
        println!();
        println!("{}", "Full-Text Search Results".underline());
        for (table, record_id, content) in &fts_results {
            println!("  [{}:{}] {}", table, record_id, truncate(content, 70));
        }
        found = true;
    }

    if !found {
        println!();
        println!("No results found for '{}'", topic);
    }

    Ok(())
}

/// Ranked search - search with relevance scoring
fn search_ranked(conn: &Connection, topic: &str) -> Result<()> {
    println!("{}", format!("Ranked search for: {}", topic).bold());
    println!("{}", "=".repeat(60));

    let mut results: Vec<SearchResult> = Vec::new();

    // Get all matches with scores
    let decisions = search_decisions(conn, topic)?;
    for (id, topic_found, decision, rationale, created_at) in decisions {
        let score = calculate_score(&topic_found, topic, &created_at);
        results.push(SearchResult {
            result_type: "decision".to_string(),
            id,
            title: topic_found.clone(),
            content: decision,
            extra: rationale,
            score,
        });
    }

    let notes = search_notes(conn, topic)?;
    for (id, category, title, content, created_at) in notes {
        let score = calculate_score(&title, topic, &created_at);
        results.push(SearchResult {
            result_type: format!("note:{}", category),
            id,
            title,
            content: content.clone(),
            extra: None,
            score,
        });
    }

    let git_results = git::search_git_commits(conn, topic)?;
    for (id, short_hash, message, committed_at) in git_results {
        let score = calculate_score(&message, topic, &committed_at);
        results.push(SearchResult {
            result_type: "commit".to_string(),
            id,
            title: format!("{}: {}", short_hash, message),
            content: message,
            extra: None,
            score,
        });
    }

    // Sort by score descending
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if results.is_empty() {
        println!();
        println!("No results found for '{}'", topic);
        return Ok(());
    }

    println!();
    for (i, result) in results.iter().enumerate() {
        let rank_indicator = if i < 3 {
            format!("[{}]", "★".repeat(3 - i)).yellow()
        } else {
            format!("[{:.1}]", result.score).dimmed()
        };

        println!(
            "{} {} #{} - {}",
            rank_indicator,
            result.result_type.cyan(),
            result.id,
            result.title.bold()
        );
        println!("   {}", truncate(&result.content, 70));
        if let Some(extra) = &result.extra {
            println!("   {}", extra.dimmed());
        }
        println!();
    }

    Ok(())
}

struct SearchResult {
    result_type: String,
    id: i64,
    title: String,
    content: String,
    extra: Option<String>,
    score: f64,
}

/// Calculate relevance score
fn calculate_score(title: &str, query: &str, created_at: &str) -> f64 {
    let mut score = 0.0;

    let title_lower = title.to_lowercase();
    let query_lower = query.to_lowercase();

    // Exact match bonus
    if title_lower == query_lower {
        score += 10.0;
    }
    // Starts with query
    else if title_lower.starts_with(&query_lower) {
        score += 5.0;
    }
    // Contains query
    else if title_lower.contains(&query_lower) {
        score += 3.0;
    }

    // Word match bonus
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    let title_words: Vec<&str> = title_lower.split_whitespace().collect();
    for qw in &query_words {
        for tw in &title_words {
            if tw.contains(qw) {
                score += 1.0;
            }
        }
    }

    // Recency bonus (newer items score higher)
    if let Ok(date) = chrono::NaiveDateTime::parse_from_str(created_at, "%Y-%m-%d %H:%M:%S") {
        let now = chrono::Utc::now().naive_utc();
        let days_old = (now - date).num_days() as f64;
        // Decay: half life of 30 days
        let recency_bonus = 2.0 * (0.5_f64).powf(days_old / 30.0);
        score += recency_bonus;
    }

    score
}

/// Search decisions table
fn search_decisions(
    conn: &Connection,
    topic: &str,
) -> Result<Vec<(i64, String, String, Option<String>, String)>> {
    let pattern = format!("%{}%", topic);
    let mut stmt = conn.prepare(
        "SELECT decision_id, topic, decision, rationale, created_at
         FROM decisions
         WHERE status = 'active' AND (topic LIKE ?1 OR decision LIKE ?1 OR rationale LIKE ?1)
         ORDER BY created_at DESC
         LIMIT 20",
    )?;

    let results = stmt.query_map([&pattern], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;

    results.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

/// Search context_notes table
fn search_notes(
    conn: &Connection,
    topic: &str,
) -> Result<Vec<(i64, String, String, String, String)>> {
    let pattern = format!("%{}%", topic);
    let mut stmt = conn.prepare(
        "SELECT note_id, category, title, content, created_at
         FROM context_notes
         WHERE status = 'active' AND (title LIKE ?1 OR content LIKE ?1 OR category LIKE ?1)
         ORDER BY created_at DESC
         LIMIT 20",
    )?;

    let results = stmt.query_map([&pattern], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;

    results.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

/// Search FTS index
fn search_fts(conn: &Connection, topic: &str) -> Result<Vec<(String, i64, String)>> {
    // Try FTS match first, fall back gracefully if FTS fails or returns invalid data
    let stmt = conn.prepare(
        "SELECT table_name, record_id, content
         FROM tracking_fts
         WHERE tracking_fts MATCH ?1
         LIMIT 20",
    );

    match stmt {
        Ok(mut s) => {
            let results = s.query_map([topic], |row| {
                // Handle potential NULL values gracefully
                let table_name: Option<String> = row.get(0).ok();
                let record_id: Option<i64> = row.get(1).ok();
                let content: Option<String> = row.get(2).ok();

                match (table_name, record_id, content) {
                    (Some(t), Some(r), Some(c)) => Ok(Some((t, r, c))),
                    _ => Ok(None),
                }
            });

            match results {
                Ok(r) => {
                    let collected: Vec<_> = r.filter_map(|res| res.ok().flatten()).collect();
                    Ok(collected)
                }
                Err(_) => Ok(Vec::new()),
            }
        }
        Err(_) => Ok(Vec::new()),
    }
}

/// Show the last N items chronologically across all tables
fn show_recent(conn: &Connection) -> Result<()> {
    println!("{}", "Recent Activity (last 10 items):".bold());
    println!("{}", "=".repeat(60));

    // Collect items from multiple tables with a unified date
    let mut items: Vec<(String, String, String)> = Vec::new(); // (datetime, type, content)

    // Recent decisions
    let mut stmt = conn.prepare(
        "SELECT created_at, topic, decision FROM decisions WHERE status = 'active' ORDER BY created_at DESC LIMIT 10",
    )?;
    let decisions: Vec<_> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    for (dt, topic, decision) in decisions {
        items.push((
            dt,
            "decision".to_string(),
            format!("{}: {}", topic, decision),
        ));
    }

    // Recent tasks
    let mut stmt = conn.prepare(
        "SELECT created_at, description, status FROM tasks ORDER BY created_at DESC LIMIT 10",
    )?;
    let tasks: Vec<_> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    for (dt, desc, status) in tasks {
        items.push((dt, "task".to_string(), format!("[{}] {}", status, desc)));
    }

    // Recent notes
    let mut stmt = conn.prepare(
        "SELECT created_at, category, title FROM context_notes WHERE status = 'active' ORDER BY created_at DESC LIMIT 10",
    )?;
    let notes: Vec<_> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();
    for (dt, category, title) in notes {
        items.push((dt, "note".to_string(), format!("[{}] {}", category, title)));
    }

    // Recent git commits
    let commits = git::get_recent_commits(conn, 10)?;
    for c in commits {
        items.push((
            c.committed_at,
            "commit".to_string(),
            format!("{}: {}", c.short_hash, c.message),
        ));
    }

    // Sort by date descending, take top 10
    items.sort_by(|a, b| b.0.cmp(&a.0));
    items.truncate(10);

    if items.is_empty() {
        println!();
        println!("No recent activity found.");
        return Ok(());
    }

    println!();
    for (dt, item_type, content) in &items {
        let type_display = match item_type.as_str() {
            "decision" => "decision".cyan(),
            "task" => "task".yellow(),
            "note" => "note".green(),
            "commit" => "commit".magenta(),
            _ => item_type.white(),
        };
        // Show just date portion for compact display
        let date_short = if dt.len() >= 10 { &dt[..10] } else { dt };
        println!("  {} {} {}", date_short.dimmed(), type_display, content);
    }

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
