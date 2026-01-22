// Archive command - archive a completed project

use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use dialoguer::Confirm;

use crate::config::{ProjectConfig, Registry};
use crate::database::open_database;
use crate::paths::{ensure_dir, get_backups_dir, get_config_path, get_project_root, get_registry_path, get_tracking_db_path};

pub fn run() -> Result<()> {
    // Load config
    let config = load_config()?;
    let project_root = get_project_root()?;

    println!("Archiving project: {}", config.name);
    println!("Location: {}", project_root.display());

    // Confirm
    let confirm = Confirm::new()
        .with_prompt("Are you sure you want to archive this project?")
        .default(false)
        .interact()?;

    if !confirm {
        println!("Cancelled.");
        return Ok(());
    }

    // Create final backup
    if let Ok(backup_path) = backup_tracking_db(&config.name, "archive") {
        println!("Final backup created: {}", backup_path.file_name().unwrap().to_string_lossy());
    }

    // End any active session
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)?;
    conn.execute(
        "UPDATE sessions SET status = 'completed', ended_at = datetime('now'), summary = 'Project archived'
         WHERE status = 'active'",
        [],
    )?;

    // Update config to mark as archived
    let config_path = get_config_path()?;
    let content = std::fs::read_to_string(&config_path)?;
    let mut config_json: serde_json::Value = serde_json::from_str(&content)?;
    config_json["archived"] = serde_json::Value::Bool(true);
    config_json["archived_at"] = serde_json::Value::String(
        Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()
    );
    std::fs::write(&config_path, serde_json::to_string_pretty(&config_json)?)?;

    // Remove from registry
    unregister_project(&project_root.to_string_lossy())?;

    println!("\n{} Project archived successfully.", "✓".green());
    println!("\nThe project data remains in place but is marked as archived.");
    println!("It has been removed from the global registry.");

    Ok(())
}

/// Create a backup of the tracking database
fn backup_tracking_db(project_name: &str, reason: &str) -> Result<std::path::PathBuf> {
    let db_path = get_tracking_db_path()?;
    let backups_dir = get_backups_dir()?;

    ensure_dir(&backups_dir)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_tracking_{}_{}.db", project_name, timestamp, reason);
    let backup_path = backups_dir.join(&backup_name);

    std::fs::copy(&db_path, &backup_path)
        .with_context(|| format!("Failed to create backup at {:?}", backup_path))?;

    Ok(backup_path)
}

/// Remove project from global registry
fn unregister_project(project_path: &str) -> Result<()> {
    let registry_path = get_registry_path()?;

    if !registry_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&registry_path)?;
    let mut registry: Registry = serde_json::from_str(&content)?;

    registry.registered_projects.retain(|p| p.path != project_path);

    let content = serde_json::to_string_pretty(&registry)?;
    std::fs::write(&registry_path, content)?;

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
