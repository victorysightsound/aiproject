// Cleanup command - clean up stale context items

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use dialoguer::Select;
use rusqlite::Connection;

use crate::database::open_database;
use crate::paths::get_tracking_db_path;

/// Stale blocker info
struct StaleBlocker {
    blocker_id: i64,
    description: String,
    created_at: String,
}

/// Stale question info
struct StaleQuestion {
    question_id: i64,
    question: String,
    created_at: String,
}

/// Stale task info
struct StaleTask {
    task_id: i64,
    description: String,
    status: String,
    created_at: String,
}

/// Stale context note info
struct StaleNote {
    note_id: i64,
    category: String,
    title: String,
    created_at: String,
}

/// Collection of stale items
struct StaleItems {
    blockers: Vec<StaleBlocker>,
    questions: Vec<StaleQuestion>,
    tasks: Vec<StaleTask>,
    context_notes: Vec<StaleNote>,
}

impl StaleItems {
    fn total(&self) -> usize {
        self.blockers.len() + self.questions.len() + self.tasks.len() + self.context_notes.len()
    }
}

pub fn run(auto: bool, days: u32) -> Result<()> {
    // Open database
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    let stale = get_stale_items(&conn, days as i64)?;

    if stale.total() == 0 {
        println!("No stale items found (threshold: {} days).", days);
        return Ok(());
    }

    println!("Found {} stale item(s) older than {} days:\n", stale.total(), days);

    let mut archived_count = 0;

    // Process blockers
    if !stale.blockers.is_empty() {
        println!("BLOCKERS ({}):", stale.blockers.len());
        for b in &stale.blockers {
            println!("  [{}] {}", b.blocker_id, b.description);
            println!("      Created: {}", format_date(&b.created_at));

            if !auto {
                let options = &["Keep active", "Mark resolved", "Archive"];
                let selection = Select::new()
                    .with_prompt("    Action?")
                    .items(options)
                    .default(2) // Archive
                    .interact()?;

                match selection {
                    1 => {
                        archive_item(&conn, "blockers", "blocker_id", b.blocker_id, "resolved")?;
                        archived_count += 1;
                    }
                    2 => {
                        archive_item(&conn, "blockers", "blocker_id", b.blocker_id, "archived")?;
                        archived_count += 1;
                    }
                    _ => {} // Keep active
                }
            } else {
                archive_item(&conn, "blockers", "blocker_id", b.blocker_id, "archived")?;
                archived_count += 1;
            }
        }
    }

    // Process questions
    if !stale.questions.is_empty() {
        println!("\nQUESTIONS ({}):", stale.questions.len());
        for q in &stale.questions {
            println!("  [{}] {}", q.question_id, q.question);
            println!("      Created: {}", format_date(&q.created_at));

            if !auto {
                let options = &["Keep open", "Mark answered", "Mark deferred"];
                let selection = Select::new()
                    .with_prompt("    Action?")
                    .items(options)
                    .default(2) // Deferred
                    .interact()?;

                match selection {
                    1 => {
                        archive_item(&conn, "questions", "question_id", q.question_id, "answered")?;
                        archived_count += 1;
                    }
                    2 => {
                        archive_item(&conn, "questions", "question_id", q.question_id, "deferred")?;
                        archived_count += 1;
                    }
                    _ => {} // Keep open
                }
            } else {
                archive_item(&conn, "questions", "question_id", q.question_id, "deferred")?;
                archived_count += 1;
            }
        }
    }

    // Process tasks
    if !stale.tasks.is_empty() {
        println!("\nTASKS ({}):", stale.tasks.len());
        for t in &stale.tasks {
            println!("  [{}] {} ({})", t.task_id, t.description, t.status);
            println!("      Created: {}", format_date(&t.created_at));

            if !auto {
                let options = &["Keep as-is", "Mark completed", "Cancel"];
                let selection = Select::new()
                    .with_prompt("    Action?")
                    .items(options)
                    .default(0) // Keep as-is
                    .interact()?;

                match selection {
                    1 => {
                        conn.execute(
                            "UPDATE tasks SET status = 'completed', completed_at = datetime('now') WHERE task_id = ?",
                            [t.task_id],
                        )?;
                        archived_count += 1;
                    }
                    2 => {
                        archive_item(&conn, "tasks", "task_id", t.task_id, "cancelled")?;
                        archived_count += 1;
                    }
                    _ => {} // Keep as-is
                }
            } else {
                archive_item(&conn, "tasks", "task_id", t.task_id, "cancelled")?;
                archived_count += 1;
            }
        }
    }

    // Process context notes (only in interactive mode)
    if !stale.context_notes.is_empty() && !auto {
        println!("\nCONTEXT NOTES ({}):", stale.context_notes.len());
        for n in &stale.context_notes {
            println!("  [{}] [{}] {}", n.note_id, n.category, n.title);
            println!("      Created: {}", format_date(&n.created_at));

            let options = &["Keep active", "Mark outdated", "Archive"];
            let selection = Select::new()
                .with_prompt("    Action?")
                .items(options)
                .default(0) // Keep active
                .interact()?;

            match selection {
                1 => {
                    archive_item(&conn, "context_notes", "note_id", n.note_id, "outdated")?;
                    archived_count += 1;
                }
                2 => {
                    archive_item(&conn, "context_notes", "note_id", n.note_id, "archived")?;
                    archived_count += 1;
                }
                _ => {} // Keep active
            }
        }
    }

    println!("\nCleanup complete. {} item(s) archived/resolved.", archived_count);

    Ok(())
}

/// Get items that haven't been updated in threshold days
fn get_stale_items(conn: &Connection, days_threshold: i64) -> Result<StaleItems> {
    let cutoff = Utc::now() - Duration::days(days_threshold);
    let cutoff_str = cutoff.format("%Y-%m-%d %H:%M:%S").to_string();

    // Stale active blockers
    let blockers = {
        let mut stmt = conn.prepare(
            "SELECT blocker_id, description, created_at FROM blockers
             WHERE status = 'active' AND created_at < ?"
        )?;
        let rows = stmt.query_map([&cutoff_str], |row| {
            Ok(StaleBlocker {
                blocker_id: row.get(0)?,
                description: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    // Stale open questions
    let questions = {
        let mut stmt = conn.prepare(
            "SELECT question_id, question, created_at FROM questions
             WHERE status = 'open' AND created_at < ?"
        )?;
        let rows = stmt.query_map([&cutoff_str], |row| {
            Ok(StaleQuestion {
                question_id: row.get(0)?,
                question: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    // Stale pending/blocked tasks
    let tasks = {
        let mut stmt = conn.prepare(
            "SELECT task_id, description, status, created_at FROM tasks
             WHERE status IN ('pending', 'blocked') AND created_at < ?"
        )?;
        let rows = stmt.query_map([&cutoff_str], |row| {
            Ok(StaleTask {
                task_id: row.get(0)?,
                description: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    // Stale context notes
    let context_notes = {
        let mut stmt = conn.prepare(
            "SELECT note_id, category, title, created_at FROM context_notes
             WHERE status = 'active' AND created_at < ?"
        )?;
        let rows = stmt.query_map([&cutoff_str], |row| {
            Ok(StaleNote {
                note_id: row.get(0)?,
                category: row.get(1)?,
                title: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    Ok(StaleItems {
        blockers,
        questions,
        tasks,
        context_notes,
    })
}

/// Archive a single item by updating its status
fn archive_item(
    conn: &Connection,
    table: &str,
    id_field: &str,
    item_id: i64,
    archive_status: &str,
) -> Result<()> {
    let query = format!("UPDATE {} SET status = ? WHERE {} = ?", table, id_field);
    conn.execute(&query, rusqlite::params![archive_status, item_id])?;
    Ok(())
}

/// Format a date string for display (just the date portion)
fn format_date(datetime_str: &str) -> &str {
    // SQLite datetime is "YYYY-MM-DD HH:MM:SS", take just the date
    if datetime_str.len() >= 10 {
        &datetime_str[..10]
    } else {
        datetime_str
    }
}
