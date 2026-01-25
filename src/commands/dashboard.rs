// Dashboard command - overview of all registered projects

use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::Registry;
use crate::database::open_database;
use crate::paths::get_registry_path;

/// Project data for dashboard display
struct ProjectData {
    name: String,
    project_type: String,
    path: String,
    exists: bool,
    active_tasks: i64,
    last_session: Option<String>,
    status: ProjectStatus,
}

#[derive(Clone, Copy)]
enum ProjectStatus {
    Active,
    New,
    Missing,
    Error,
}

impl ProjectStatus {
    fn icon(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "●",
            ProjectStatus::New => "○",
            ProjectStatus::Missing => "✗",
            ProjectStatus::Error => "?",
        }
    }
}

pub fn run() -> Result<()> {
    let registry = load_registry()?;

    if registry.registered_projects.is_empty() {
        println!("No projects registered.");
        return Ok(());
    }

    println!("\n{}", "=".repeat(70));
    println!("{}", "PROJECT DASHBOARD".bold());
    println!("{}\n", "=".repeat(70));

    let mut project_data = Vec::new();

    for p in &registry.registered_projects {
        let path = Path::new(&p.path);
        let exists = path.exists();
        let db_path = path.join(".tracking").join("tracking.db");

        let mut data = ProjectData {
            name: p.name.clone(),
            project_type: p.project_type.clone(),
            path: p.path.clone(),
            exists,
            active_tasks: 0,
            last_session: None,
            status: if exists {
                ProjectStatus::New
            } else {
                ProjectStatus::Missing
            },
        };

        if exists && db_path.exists() {
            match open_database(&db_path) {
                Ok(conn) => {
                    // Get active tasks count
                    if let Ok(count) = conn.query_row(
                        "SELECT COUNT(*) FROM tasks WHERE status NOT IN ('completed', 'cancelled')",
                        [],
                        |row| row.get::<_, i64>(0),
                    ) {
                        data.active_tasks = count;
                    }

                    // Get last session
                    if let Ok((ended_at, _summary)) = conn.query_row(
                        "SELECT ended_at, summary FROM sessions WHERE status = 'completed' ORDER BY ended_at DESC LIMIT 1",
                        [],
                        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?)),
                    ) {
                        data.last_session = ended_at;
                        data.status = ProjectStatus::Active;
                    }
                }
                Err(_) => {
                    data.status = ProjectStatus::Error;
                }
            }
        }

        project_data.push(data);
    }

    // Display projects
    for (i, data) in project_data.iter().enumerate() {
        let status_icon = match data.status {
            ProjectStatus::Active => data.status.icon().green(),
            ProjectStatus::New => data.status.icon().white(),
            ProjectStatus::Missing => data.status.icon().red(),
            ProjectStatus::Error => data.status.icon().yellow(),
        };

        println!(
            "  {}. {} {} ({})",
            i + 1,
            status_icon,
            data.name.bold(),
            data.project_type
        );

        if data.exists {
            println!("       Tasks: {} active", data.active_tasks);
            if let Some(ref last) = data.last_session {
                println!("       Last: {}", format_date(last));
            }
        } else {
            println!("       {} Path not found", "⚠".yellow());
        }
        println!();
    }

    // Interactive selection
    println!("{}", "-".repeat(70));
    print!("Enter number to switch to project (or Enter to exit): ");
    io::stdout().flush()?;

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let input = input.trim();
        if !input.is_empty() {
            if let Ok(idx) = input.parse::<usize>() {
                if idx > 0 && idx <= project_data.len() {
                    let selected = &project_data[idx - 1];
                    if selected.exists {
                        println!("\nTo switch to {}:", selected.name);
                        println!("  cd {}", selected.path);
                    } else {
                        println!("\nProject path not found: {}", selected.path);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Load the global registry
fn load_registry() -> Result<Registry> {
    let registry_path = get_registry_path()?;

    if !registry_path.exists() {
        return Ok(Registry::default());
    }

    let content =
        std::fs::read_to_string(&registry_path).with_context(|| "Failed to read registry.json")?;
    let registry: Registry =
        serde_json::from_str(&content).with_context(|| "Failed to parse registry.json")?;
    Ok(registry)
}

/// Format a datetime string for display (just the date portion)
fn format_date(datetime_str: &str) -> &str {
    // SQLite datetime is "YYYY-MM-DD HH:MM:SS", take just the date
    if datetime_str.len() >= 10 {
        &datetime_str[..10]
    } else {
        datetime_str
    }
}
