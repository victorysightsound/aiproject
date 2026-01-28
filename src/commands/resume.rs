// Resume command - detailed context for resuming work

use anyhow::{Context, Result};
use colored::Colorize;
use rusqlite::Connection;
use serde::Serialize;

use crate::config::ProjectConfig;
use crate::database::open_database;
use crate::models::{Blocker, Decision, Question, Task};
use crate::paths::{get_config_path, get_tracking_db_path};
use crate::session::{get_last_completed_session, get_or_create_session};

pub fn run(for_ai: bool) -> Result<()> {
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)
        .with_context(|| format!("Failed to open tracking database at {:?}", db_path))?;

    let config = load_config()?;

    if for_ai {
        output_json(&conn, &config)
    } else {
        output_human(&conn, &config)
    }
}

/// Load project configuration
fn load_config() -> Result<ProjectConfig> {
    let config_path = get_config_path()?;
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config at {:?}", config_path))?;
    let config: ProjectConfig =
        serde_json::from_str(&content).with_context(|| "Failed to parse config.json")?;
    Ok(config)
}

/// JSON output for AI consumption
#[derive(Serialize)]
struct ResumeContext {
    project: ProjectInfo,
    current_session: Option<SessionInfo>,
    last_session: Option<SessionInfo>,
    active_blockers: Vec<BlockerInfo>,
    active_tasks: Vec<TaskInfo>,
    recent_decisions: Vec<DecisionInfo>,
    open_questions: Vec<QuestionInfo>,
}

#[derive(Serialize)]
struct ProjectInfo {
    name: String,
    project_type: String,
    description: Option<String>,
}

#[derive(Serialize)]
struct SessionInfo {
    session_id: i64,
    started_at: String,
    ended_at: Option<String>,
    summary: Option<String>,
    structured_summary: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct BlockerInfo {
    blocker_id: i64,
    description: String,
    created_at: String,
}

#[derive(Serialize)]
struct TaskInfo {
    task_id: i64,
    description: String,
    status: String,
    priority: String,
    blocked_by: Option<String>,
}

#[derive(Serialize)]
struct DecisionInfo {
    decision_id: i64,
    topic: String,
    decision: String,
    rationale: Option<String>,
}

#[derive(Serialize)]
struct QuestionInfo {
    question_id: i64,
    question: String,
    context: Option<String>,
}

fn output_json(conn: &Connection, config: &ProjectConfig) -> Result<()> {
    let session = get_or_create_session(conn)?;
    let last_session = get_last_completed_session(conn)?;

    let context = ResumeContext {
        project: ProjectInfo {
            name: config.name.clone(),
            project_type: config.project_type.clone(),
            description: config.description.clone(),
        },
        current_session: Some(SessionInfo {
            session_id: session.session_id,
            started_at: session.started_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            ended_at: session
                .ended_at
                .map(|e| e.format("%Y-%m-%d %H:%M:%S").to_string()),
            summary: session.summary.clone(),
            structured_summary: session.structured_summary.as_ref().and_then(|s| serde_json::from_str(s).ok()),
        }),
        last_session: last_session.map(|s| SessionInfo {
            session_id: s.session_id,
            started_at: s.started_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            ended_at: s
                .ended_at
                .map(|e| e.format("%Y-%m-%d %H:%M:%S").to_string()),
            summary: s.summary.clone(),
            structured_summary: s.structured_summary.as_ref().and_then(|ss| serde_json::from_str(ss).ok()),
        }),
        active_blockers: get_active_blockers(conn)?
            .into_iter()
            .map(|b| BlockerInfo {
                blocker_id: b.blocker_id,
                description: b.description,
                created_at: b.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect(),
        active_tasks: get_active_tasks(conn)?
            .into_iter()
            .map(|t| TaskInfo {
                task_id: t.task_id,
                description: t.description,
                status: t.status,
                priority: t.priority,
                blocked_by: t.blocked_by,
            })
            .collect(),
        recent_decisions: get_recent_decisions(conn, 10)?
            .into_iter()
            .map(|d| DecisionInfo {
                decision_id: d.decision_id,
                topic: d.topic,
                decision: d.decision,
                rationale: d.rationale,
            })
            .collect(),
        open_questions: get_open_questions(conn)?
            .into_iter()
            .map(|q| QuestionInfo {
                question_id: q.question_id,
                question: q.question,
                context: q.context,
            })
            .collect(),
    };

    println!("{}", serde_json::to_string_pretty(&context)?);
    Ok(())
}

fn output_human(conn: &Connection, config: &ProjectConfig) -> Result<()> {
    let session = get_or_create_session(conn)?;

    println!("{}", "=".repeat(60));
    println!("{}", "RESUME CONTEXT".bold());
    println!("{}", "=".repeat(60));
    println!();

    // Project info
    println!("Project: {} ({})", config.name.bold(), config.project_type);
    if let Some(desc) = &config.description {
        println!("Description: {}", desc);
    }
    println!();

    // Current session
    println!("{}", "Current Session".underline());
    println!(
        "Session #{} started {}",
        session.session_id,
        session.started_at.format("%Y-%m-%d %H:%M")
    );
    println!();

    // Last session summary
    if let Some(last) = get_last_completed_session(conn)? {
        println!("{}", "Last Session".underline());
        println!(
            "#{} ended {}",
            last.session_id,
            last.ended_at
                .map(|e| e.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default()
        );
        if let Some(summary) = &last.summary {
            println!("Summary: {}", summary);
        }
        // Show structured summary details if available
        if let Some(ref structured) = last.structured_summary {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(structured) {
                if let Some(arr) = parsed.get("decisions").and_then(|v| v.as_array()) {
                    if !arr.is_empty() {
                        println!("  Decisions made:");
                        for d in arr {
                            if let Some(s) = d.as_str() {
                                println!("    - {}", s);
                            }
                        }
                    }
                }
                if let Some(arr) = parsed.get("git_commits").and_then(|v| v.as_array()) {
                    if !arr.is_empty() {
                        println!("  Commits ({}):", arr.len());
                        for c in arr.iter().take(5) {
                            if let Some(s) = c.as_str() {
                                println!("    - {}", s);
                            }
                        }
                    }
                }
            }
        }
        println!();
    }

    // Active blockers - these are critical
    let blockers = get_active_blockers(conn)?;
    if !blockers.is_empty() {
        println!("{}", "BLOCKERS (resolve these first!)".red().bold());
        for b in &blockers {
            println!("  {} {}", "✗".red(), b.description);
        }
        println!();
    }

    // Active tasks
    let tasks = get_active_tasks(conn)?;
    if !tasks.is_empty() {
        println!("{}", "Active Tasks".underline());
        for t in &tasks {
            let status_icon = match t.status.as_str() {
                "in_progress" => "◐".yellow(),
                "blocked" => "✗".red(),
                _ => "○".white(),
            };
            let priority_marker = match t.priority.as_str() {
                "urgent" => " [URGENT]".red(),
                "high" => " [high]".yellow(),
                _ => "".white(),
            };
            println!(
                "  {} [{}] {}{}",
                status_icon, t.task_id, t.description, priority_marker
            );
        }
        println!();
    }

    // Recent decisions - important for context
    let decisions = get_recent_decisions(conn, 5)?;
    if !decisions.is_empty() {
        println!("{}", "Recent Decisions".underline());
        for d in &decisions {
            println!("  • {}: {}", d.topic.bold(), d.decision);
            if let Some(rationale) = &d.rationale {
                println!("    Why: {}", rationale.dimmed());
            }
        }
        println!();
    }

    // Open questions
    let questions = get_open_questions(conn)?;
    if !questions.is_empty() {
        println!("{}", "Open Questions".underline());
        for q in &questions {
            println!("  ? {}", q.question);
            if let Some(ctx) = &q.context {
                println!("    Context: {}", ctx.dimmed());
            }
        }
        println!();
    }

    // Suggested next action
    println!("{}", "Suggested Next Action".green().bold());
    if !blockers.is_empty() {
        println!("  Resolve blocker: {}", blockers[0].description);
    } else if let Some(task) = tasks.iter().find(|t| t.status == "in_progress") {
        println!("  Continue: {}", task.description);
    } else if let Some(task) = tasks
        .iter()
        .find(|t| t.priority == "urgent" || t.priority == "high")
    {
        println!("  Start high-priority task: {}", task.description);
    } else if !tasks.is_empty() {
        println!("  Start next task: {}", tasks[0].description);
    } else {
        println!("  No pending tasks. Check if there's anything to add.");
    }

    Ok(())
}

// Helper functions for database queries

fn get_active_tasks(conn: &Connection) -> Result<Vec<Task>> {
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

    tasks.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

fn get_active_blockers(conn: &Connection) -> Result<Vec<Blocker>> {
    let mut stmt = conn.prepare(
        "SELECT blocker_id, session_id, created_at, resolved_at, description, status, resolution, related_task_id
         FROM blockers
         WHERE status = 'active'
         ORDER BY created_at DESC"
    )?;

    let blockers = stmt.query_map([], |row| {
        Ok(Blocker {
            blocker_id: row.get(0)?,
            session_id: row.get(1)?,
            created_at: parse_datetime(row.get::<_, String>(2)?),
            resolved_at: row.get::<_, Option<String>>(3)?.map(parse_datetime),
            description: row.get(4)?,
            status: row.get(5)?,
            resolution: row.get(6)?,
            related_task_id: row.get(7)?,
        })
    })?;

    blockers
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.into())
}

fn get_recent_decisions(conn: &Connection, limit: usize) -> Result<Vec<Decision>> {
    let mut stmt = conn.prepare(
        "SELECT decision_id, session_id, created_at, topic, decision, rationale, alternatives, status, superseded_by
         FROM decisions
         WHERE status = 'active'
         ORDER BY created_at DESC
         LIMIT ?1"
    )?;

    let decisions = stmt.query_map([limit as i64], |row| {
        Ok(Decision {
            decision_id: row.get(0)?,
            session_id: row.get(1)?,
            created_at: parse_datetime(row.get::<_, String>(2)?),
            topic: row.get(3)?,
            decision: row.get(4)?,
            rationale: row.get(5)?,
            alternatives: row.get(6)?,
            status: row.get(7)?,
            superseded_by: row.get(8)?,
        })
    })?;

    decisions
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.into())
}

fn get_open_questions(conn: &Connection) -> Result<Vec<Question>> {
    let mut stmt = conn.prepare(
        "SELECT question_id, session_id, created_at, answered_at, question, context, answer, status
         FROM questions
         WHERE status = 'open'
         ORDER BY created_at DESC",
    )?;

    let questions = stmt.query_map([], |row| {
        Ok(Question {
            question_id: row.get(0)?,
            session_id: row.get(1)?,
            created_at: parse_datetime(row.get::<_, String>(2)?),
            answered_at: row.get::<_, Option<String>>(3)?.map(parse_datetime),
            question: row.get(4)?,
            context: row.get(5)?,
            answer: row.get(6)?,
            status: row.get(7)?,
        })
    })?;

    questions
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.into())
}

/// Parse datetime string from SQLite
fn parse_datetime(s: String) -> chrono::DateTime<chrono::Utc> {
    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|_| chrono::Utc::now())
}
