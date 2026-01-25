// Check command - verify database integrity

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::ProjectConfig;
use crate::database::open_database;
use crate::paths::{get_config_path, get_tracking_db_path};
use crate::SCHEMA_VERSION;

pub fn run() -> Result<()> {
    // Load config
    let config = load_config()?;

    println!("Checking database integrity...\n");

    let mut issues = Vec::new();

    // Check tracking database
    let tracking_db = get_tracking_db_path()?;
    println!("Tracking DB: {}", tracking_db.display());

    if tracking_db.exists() {
        match open_database(&tracking_db) {
            Ok(conn) => {
                // Integrity check
                match conn.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0)) {
                    Ok(result) if result == "ok" => {
                        println!("  {} Integrity check passed", "✓".green());
                    }
                    Ok(result) => {
                        println!("  {} Integrity issues: {}", "✗".red(), result);
                        issues.push(format!("Tracking DB integrity: {}", result));
                    }
                    Err(e) => {
                        println!("  {} Integrity check failed: {}", "✗".red(), e);
                        issues.push(format!("Integrity check error: {}", e));
                    }
                }

                // Check schema version
                let current_version: String = conn
                    .query_row(
                        "SELECT value FROM project_meta WHERE key = 'schema_version'",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or_else(|_| "1.0".to_string());

                if current_version == SCHEMA_VERSION {
                    println!("  {} Schema version: v{}", "✓".green(), current_version);
                } else {
                    println!(
                        "  {} Schema needs upgrade: v{} → v{}",
                        "⚠".yellow(),
                        current_version,
                        SCHEMA_VERSION
                    );
                }

                // Check table counts
                let session_count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
                    .unwrap_or(0);
                let task_count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
                    .unwrap_or(0);
                let decision_count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM decisions", [], |row| row.get(0))
                    .unwrap_or(0);

                println!(
                    "  {} {} sessions, {} tasks, {} decisions",
                    "✓".green(),
                    session_count,
                    task_count,
                    decision_count
                );
            }
            Err(e) => {
                println!("  {} Error: {}", "✗".red(), e);
                issues.push(format!("Tracking DB error: {}", e));
            }
        }
    } else {
        println!("  {} File not found", "✗".red());
        issues.push("Tracking database not found".to_string());
    }

    // Check config
    println!("\nConfig: {}", get_config_path()?.display());
    println!("  {} Project: {}", "✓".green(), config.name);
    println!("  {} Type: {}", "✓".green(), config.project_type);

    // Summary
    if issues.is_empty() {
        println!("\n{} All checks passed", "✓".green());
    } else {
        println!("\n{} Found {} issue(s):", "⚠".yellow(), issues.len());
        for issue in &issues {
            println!("  • {}", issue);
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
