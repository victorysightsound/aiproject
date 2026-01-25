// Task commands - add, update, list

use anyhow::{bail, Context, Result};
use colored::Colorize;
use rusqlite::Connection;

use crate::cli::{TaskCommands, TaskSubcommand};
use crate::database::open_database;
use crate::models::Task;
use crate::paths::get_tracking_db_path;
use crate::session::get_or_create_session;

pub fn run(cmd: TaskCommands) -> Result<()> {
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    match cmd.command {
        TaskSubcommand::Add {
            description,
            priority,
        } => {
            let session = get_or_create_session(&conn)?;
            cmd_task_add(&conn, session.session_id, &description, &priority)
        }
        TaskSubcommand::Update {
            id,
            status,
            notes,
            priority,
            blocked_by,
        } => cmd_task_update(&conn, id, status, notes, priority, blocked_by),
        TaskSubcommand::List => list(),
    }
}

/// Shortcut for 'task list'
pub fn list() -> Result<()> {
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    cmd_task_list(&conn)
}

/// Add a new task
fn cmd_task_add(
    conn: &Connection,
    session_id: i64,
    description: &str,
    priority: &str,
) -> Result<()> {
    // Validate priority
    let valid_priorities = ["low", "normal", "high", "urgent"];
    if !valid_priorities.contains(&priority) {
        bail!(
            "Invalid priority '{}'. Valid priorities: {}",
            priority,
            valid_priorities.join(", ")
        );
    }

    // Insert task
    conn.execute(
        "INSERT INTO tasks (session_id, description, status, priority) VALUES (?1, ?2, 'pending', ?3)",
        rusqlite::params![session_id, description, priority],
    )?;

    let task_id = conn.last_insert_rowid();

    // Insert into activity_log
    let summary = format!("Task added: {}", truncate(description, 50));
    conn.execute(
        "INSERT INTO activity_log (session_id, action_type, action_id, summary) VALUES (?1, 'task_update', ?2, ?3)",
        rusqlite::params![session_id, task_id, summary],
    )?;

    // Update FTS index
    conn.execute(
        "INSERT INTO tracking_fts (content, table_name, record_id) VALUES (?1, 'tasks', ?2)",
        rusqlite::params![description, task_id],
    )?;

    let priority_display = match priority {
        "urgent" => format!("[{}]", priority.red()),
        "high" => format!("[{}]", priority.yellow()),
        _ => format!("[{}]", priority),
    };

    println!(
        "{} Added task #{} {}: {}",
        "✓".green(),
        task_id,
        priority_display,
        description
    );
    Ok(())
}

/// Update an existing task
fn cmd_task_update(
    conn: &Connection,
    task_id: i64,
    status: Option<String>,
    notes: Option<String>,
    priority: Option<String>,
    blocked_by: Option<String>,
) -> Result<()> {
    // Check task exists
    let exists: bool = conn
        .query_row("SELECT 1 FROM tasks WHERE task_id = ?1", [task_id], |_| {
            Ok(true)
        })
        .unwrap_or(false);

    if !exists {
        bail!("Task #{} not found", task_id);
    }

    let mut updates = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // Handle status update
    if let Some(ref s) = status {
        let valid_statuses = [
            "pending",
            "in_progress",
            "completed",
            "blocked",
            "cancelled",
        ];
        if !valid_statuses.contains(&s.as_str()) {
            bail!(
                "Invalid status '{}'. Valid statuses: {}",
                s,
                valid_statuses.join(", ")
            );
        }
        updates.push("status = ?");
        params.push(Box::new(s.clone()));

        // If completing, set completed_at
        if s == "completed" {
            updates.push("completed_at = datetime('now')");
        }
    }

    // Handle notes update
    if let Some(ref n) = notes {
        updates.push("notes = ?");
        params.push(Box::new(n.clone()));
    }

    // Handle priority update
    if let Some(ref p) = priority {
        let valid_priorities = ["low", "normal", "high", "urgent"];
        if !valid_priorities.contains(&p.as_str()) {
            bail!(
                "Invalid priority '{}'. Valid priorities: {}",
                p,
                valid_priorities.join(", ")
            );
        }
        updates.push("priority = ?");
        params.push(Box::new(p.clone()));
    }

    // Handle blocked_by update
    if let Some(ref b) = blocked_by {
        updates.push("blocked_by = ?");
        params.push(Box::new(b.clone()));
        // Also set status to blocked if not already specified
        if status.is_none() {
            updates.push("status = 'blocked'");
        }
    }

    if updates.is_empty() {
        println!(
            "{} No updates specified for task #{}",
            "!".yellow(),
            task_id
        );
        return Ok(());
    }

    // Build and execute update query
    let query = format!("UPDATE tasks SET {} WHERE task_id = ?", updates.join(", "));

    // Create statement and bind params dynamically
    let mut stmt = conn.prepare(&query)?;

    // We need to collect references for params
    let mut param_idx = 1;
    for param in &params {
        stmt.raw_bind_parameter(param_idx, param.as_ref())?;
        param_idx += 1;
    }
    stmt.raw_bind_parameter(param_idx, task_id)?;
    stmt.raw_execute()?;

    // Build status message
    let mut changes = Vec::new();
    if let Some(s) = status {
        changes.push(format!("status → {}", s));
    }
    if let Some(p) = priority {
        changes.push(format!("priority → {}", p));
    }
    if notes.is_some() {
        changes.push("notes updated".to_string());
    }
    if let Some(b) = blocked_by {
        changes.push(format!("blocked by: {}", b));
    }

    println!(
        "{} Updated task #{}: {}",
        "✓".green(),
        task_id,
        changes.join(", ")
    );
    Ok(())
}

/// List active tasks
fn cmd_task_list(conn: &Connection) -> Result<()> {
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

    let tasks: Vec<Task> = tasks.collect::<Result<Vec<_>, _>>()?;

    if tasks.is_empty() {
        println!("No active tasks.");
        return Ok(());
    }

    println!("{}", "Active Tasks:".bold());
    println!("{}", "-".repeat(60));

    for task in tasks {
        let status_icon = match task.status.as_str() {
            "in_progress" => "◐".yellow(),
            "blocked" => "✗".red(),
            "pending" => "○".white(),
            _ => "○".white(),
        };

        let priority_display = match task.priority.as_str() {
            "urgent" => format!("[{}]", "urgent".red()),
            "high" => format!("[{}]", "high".yellow()),
            "normal" => "[normal]".to_string(),
            "low" => format!("[{}]", "low".dimmed()),
            _ => format!("[{}]", task.priority),
        };

        println!(
            "{} #{:<4} {} {}",
            status_icon, task.task_id, priority_display, task.description
        );

        if let Some(blocked_by) = &task.blocked_by {
            println!("         {} Blocked by: {}", "→".red(), blocked_by);
        }

        if let Some(notes) = &task.notes {
            println!("         Notes: {}", notes.dimmed());
        }
    }

    Ok(())
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
