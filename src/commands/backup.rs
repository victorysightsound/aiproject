// Backup command - manual backup of tracking data

use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Utc;

use crate::config::ProjectConfig;
use crate::database::backup_database;
use crate::paths::{ensure_dir, get_backups_dir, get_config_path, get_tracking_db_path};

pub fn run() -> Result<()> {
    // Load project config
    let config = load_config()?;

    // Create backup
    let backup_path = backup_tracking_db(&config.name, "manual")?;

    println!("Backup created: {}", backup_path.display());

    // List recent backups
    let backups_dir = get_backups_dir()?;
    let mut backups: Vec<_> = std::fs::read_dir(&backups_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{}_tracking_", config.name))
        })
        .collect();

    backups.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    if !backups.is_empty() {
        println!("\nRecent backups ({} total):", backups.len());
        for b in backups.iter().take(5) {
            let meta = b.metadata()?;
            let size_kb = meta.len() as f64 / 1024.0;
            println!(
                "  • {} ({:.1} KB)",
                b.file_name().to_string_lossy(),
                size_kb
            );
        }
    }

    Ok(())
}

/// Create a backup of the tracking database
fn backup_tracking_db(project_name: &str, reason: &str) -> Result<PathBuf> {
    let db_path = get_tracking_db_path()?;
    let backups_dir = get_backups_dir()?;

    ensure_dir(&backups_dir)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_tracking_{}_{}.db", project_name, timestamp, reason);
    let backup_path = backups_dir.join(&backup_name);

    backup_database(&db_path, &backup_path)
        .with_context(|| format!("Failed to create backup at {:?}", backup_path))?;

    Ok(backup_path)
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
