// Snapshot command - generate AI context snapshot

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;

use crate::config::ProjectConfig;
use crate::database::open_database;
use crate::paths::{get_config_path, get_tracking_db_path};

#[derive(Serialize)]
struct ContextSnapshot {
    project: String,
    project_type: String,
    generated_at: String,
    active_session: Option<SessionSnapshot>,
    active_tasks: Vec<TaskSnapshot>,
    active_blockers: Vec<BlockerSnapshot>,
    open_questions: Vec<QuestionSnapshot>,
    recent_decisions: Vec<DecisionSnapshot>,
}

#[derive(Serialize)]
struct SessionSnapshot {
    session_id: i64,
    started_at: String,
    agent: Option<String>,
}

#[derive(Serialize)]
struct TaskSnapshot {
    task_id: i64,
    description: String,
    status: String,
    priority: Option<String>,
}

#[derive(Serialize)]
struct BlockerSnapshot {
    blocker_id: i64,
    description: String,
}

#[derive(Serialize)]
struct QuestionSnapshot {
    question_id: i64,
    question: String,
    context: Option<String>,
}

#[derive(Serialize)]
struct DecisionSnapshot {
    topic: String,
    decision: String,
    rationale: Option<String>,
}

pub fn run() -> Result<()> {
    // Load config
    let config = load_config()?;

    // Open database
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)?;

    // Get active session
    let active_session: Option<SessionSnapshot> = conn
        .query_row(
            "SELECT session_id, started_at, agent FROM sessions WHERE status = 'active' LIMIT 1",
            [],
            |row| {
                Ok(SessionSnapshot {
                    session_id: row.get(0)?,
                    started_at: row.get(1)?,
                    agent: row.get(2)?,
                })
            },
        )
        .ok();

    // Get active tasks
    let mut stmt = conn.prepare(
        "SELECT task_id, description, status, priority FROM tasks
         WHERE status NOT IN ('completed', 'cancelled')
         ORDER BY priority DESC, created_at",
    )?;
    let active_tasks: Vec<TaskSnapshot> = stmt
        .query_map([], |row| {
            Ok(TaskSnapshot {
                task_id: row.get(0)?,
                description: row.get(1)?,
                status: row.get(2)?,
                priority: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Get active blockers
    let mut stmt =
        conn.prepare("SELECT blocker_id, description FROM blockers WHERE status = 'active'")?;
    let active_blockers: Vec<BlockerSnapshot> = stmt
        .query_map([], |row| {
            Ok(BlockerSnapshot {
                blocker_id: row.get(0)?,
                description: row.get(1)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Get open questions
    let mut stmt =
        conn.prepare("SELECT question_id, question, context FROM questions WHERE status = 'open'")?;
    let open_questions: Vec<QuestionSnapshot> = stmt
        .query_map([], |row| {
            Ok(QuestionSnapshot {
                question_id: row.get(0)?,
                question: row.get(1)?,
                context: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Get recent decisions
    let mut stmt = conn.prepare(
        "SELECT topic, decision, rationale FROM decisions
         WHERE status = 'active'
         ORDER BY created_at DESC LIMIT 10",
    )?;
    let recent_decisions: Vec<DecisionSnapshot> = stmt
        .query_map([], |row| {
            Ok(DecisionSnapshot {
                topic: row.get(0)?,
                decision: row.get(1)?,
                rationale: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Build snapshot
    let snapshot = ContextSnapshot {
        project: config.name,
        project_type: config.project_type,
        generated_at: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        active_session,
        active_tasks,
        active_blockers,
        open_questions,
        recent_decisions,
    };

    // Output as JSON
    println!("{}", serde_json::to_string_pretty(&snapshot)?);

    Ok(())
}

/// Load project configuration
fn load_config() -> Result<ProjectConfig> {
    let config_path = get_config_path()?;
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| "No project found. Run 'proj init' to initialize.")?;
    let config: ProjectConfig =
        serde_json::from_str(&content).with_context(|| "Failed to parse config.json")?;
    Ok(config)
}
