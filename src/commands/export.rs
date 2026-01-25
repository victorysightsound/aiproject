// Export command - export session history

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;

use crate::config::ProjectConfig;
use crate::database::open_database;
use crate::paths::{get_config_path, get_tracking_db_path};

#[derive(Serialize)]
struct ExportData {
    project: ProjectInfo,
    exported_at: String,
    sessions: Vec<SessionExport>,
    decisions: Vec<DecisionExport>,
    tasks: Vec<TaskExport>,
}

#[derive(Serialize)]
struct ProjectInfo {
    name: String,
    project_type: String,
}

#[derive(Serialize)]
struct SessionExport {
    session_id: i64,
    started_at: String,
    ended_at: Option<String>,
    summary: Option<String>,
    agent: Option<String>,
    status: String,
}

#[derive(Serialize)]
struct DecisionExport {
    topic: String,
    decision: String,
    rationale: Option<String>,
    created_at: String,
}

#[derive(Serialize)]
struct TaskExport {
    description: String,
    status: String,
    priority: Option<String>,
    created_at: String,
    completed_at: Option<String>,
}

pub fn run(format: String) -> Result<()> {
    // Load config
    let config = load_config()?;

    // Open database
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)?;

    // Gather data
    let mut stmt = conn.prepare(
        "SELECT session_id, started_at, ended_at, summary, agent, status
         FROM sessions ORDER BY started_at",
    )?;
    let sessions: Vec<SessionExport> = stmt
        .query_map([], |row| {
            Ok(SessionExport {
                session_id: row.get(0)?,
                started_at: row.get(1)?,
                ended_at: row.get(2)?,
                summary: row.get(3)?,
                agent: row.get(4)?,
                status: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt = conn.prepare(
        "SELECT topic, decision, rationale, created_at
         FROM decisions WHERE status = 'active' ORDER BY created_at",
    )?;
    let decisions: Vec<DecisionExport> = stmt
        .query_map([], |row| {
            Ok(DecisionExport {
                topic: row.get(0)?,
                decision: row.get(1)?,
                rationale: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut stmt = conn.prepare(
        "SELECT description, status, priority, created_at, completed_at
         FROM tasks ORDER BY created_at",
    )?;
    let tasks: Vec<TaskExport> = stmt
        .query_map([], |row| {
            Ok(TaskExport {
                description: row.get(0)?,
                status: row.get(1)?,
                priority: row.get(2)?,
                created_at: row.get(3)?,
                completed_at: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Output based on format
    match format.as_str() {
        "json" => {
            let export_data = ExportData {
                project: ProjectInfo {
                    name: config.name,
                    project_type: config.project_type,
                },
                exported_at: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                sessions,
                decisions,
                tasks,
            };
            println!("{}", serde_json::to_string_pretty(&export_data)?);
        }
        "md" | _ => {
            println!("# Project: {}\n", config.name);
            println!("Type: {}", config.project_type);
            println!("Exported: {}\n", Utc::now().format("%Y-%m-%d %H:%M"));

            println!("## Sessions ({} total)\n", sessions.len());
            for s in &sessions {
                let ended = s.ended_at.as_deref().unwrap_or("ongoing");
                let summary = s.summary.as_deref().unwrap_or("No summary");
                println!("### Session #{} ({})\n", s.session_id, s.status);
                println!("- Started: {}", s.started_at);
                println!("- Ended: {}", ended);
                println!("- Summary: {}\n", summary);
            }

            println!("## Decisions ({} active)\n", decisions.len());
            for d in &decisions {
                println!("### {}\n", d.topic);
                println!("{}", d.decision);
                if let Some(rationale) = &d.rationale {
                    println!("\n*Rationale: {}*", rationale);
                }
                println!();
            }

            println!("## Tasks ({} total)\n", tasks.len());
            for t in &tasks {
                let status_marker = match t.status.as_str() {
                    "completed" => "[x]",
                    _ => "[ ]",
                };
                let priority = t.priority.as_deref().unwrap_or("normal");
                println!("- {} {} ({})", status_marker, t.description, priority);
            }
        }
    }

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
