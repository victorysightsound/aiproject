// Rollback command - undo a release or restore schema from backup

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use dialoguer::{Confirm, Select};

use crate::paths::{get_global_dir, get_project_root};

pub fn run(version: Option<String>, schema: bool, list: bool) -> Result<()> {
    if list {
        return list_schema_backups();
    }

    if schema {
        return restore_schema_backup();
    }

    // Default: release rollback
    release_rollback(version)
}

// ============================================================================
// Schema Backup/Restore
// ============================================================================

/// Get the backups directory
fn get_backups_dir() -> Result<PathBuf> {
    let global_dir = get_global_dir()?;
    Ok(global_dir.join("backups"))
}

/// Get the project backup directory name
fn backup_dir_name(project_name: &str) -> String {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    format!("{}-{}", project_name, timestamp)
}

/// Create a backup of the current project's .tracking directory
/// Only keeps the most recent backup per project (deletes older ones)
pub fn create_backup(project_name: &str) -> Result<PathBuf> {
    let project_root = get_project_root()?;
    let tracking_path = project_root.join(".tracking");

    if !tracking_path.exists() {
        bail!("No .tracking directory found in current project");
    }

    let backups_dir = get_backups_dir()?;
    std::fs::create_dir_all(&backups_dir)?;

    // Delete existing backups for this project (keep only 1)
    delete_old_backups_for_project(&backups_dir, project_name)?;

    let backup_name = backup_dir_name(project_name);
    let backup_path = backups_dir.join(&backup_name);

    // Copy the entire .tracking directory
    copy_dir_recursive(&tracking_path, &backup_path)
        .with_context(|| format!("Failed to create backup at {:?}", backup_path))?;

    // Write a metadata file
    let metadata = BackupMetadata {
        project_name: project_name.to_string(),
        project_path: project_root.to_string_lossy().to_string(),
        created_at: Utc::now(),
        schema_version: get_current_schema_version(&tracking_path)?,
    };
    let metadata_path = backup_path.join("backup_metadata.json");
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(&metadata_path, metadata_json)?;

    Ok(backup_path)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct BackupMetadata {
    project_name: String,
    project_path: String,
    created_at: DateTime<Utc>,
    schema_version: String,
}

/// Get current schema version from config.json
fn get_current_schema_version(tracking_path: &Path) -> Result<String> {
    let config_path = tracking_path.join("config.json");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;
        if let Some(version) = config.get("schema_version").and_then(|v| v.as_str()) {
            return Ok(version.to_string());
        }
    }
    Ok("unknown".to_string())
}

/// Delete old backups for a project (keep only the newest)
fn delete_old_backups_for_project(backups_dir: &Path, project_name: &str) -> Result<()> {
    if !backups_dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(backups_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let metadata_path = path.join("backup_metadata.json");
            if metadata_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                    if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&content) {
                        if metadata.project_name == project_name {
                            // This is an old backup for the same project - delete it
                            let _ = std::fs::remove_dir_all(&path);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Copy a directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// List available schema backups
fn list_schema_backups() -> Result<()> {
    let backups_dir = get_backups_dir()?;

    if !backups_dir.exists() {
        println!("No backups found.");
        return Ok(());
    }

    // Get current project name for filtering
    let project_root = get_project_root().ok();
    let current_project_name = project_root.as_ref().and_then(|p| {
        let config_path = p.join(".tracking").join("config.json");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).ok()?;
            let config: serde_json::Value = serde_json::from_str(&content).ok()?;
            config.get("name").and_then(|v| v.as_str()).map(String::from)
        } else {
            None
        }
    });

    println!("{}", "Available Schema Backups".bold());
    println!();

    let mut backups: Vec<(String, BackupMetadata)> = Vec::new();

    for entry in std::fs::read_dir(&backups_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let metadata_path = path.join("backup_metadata.json");
            if metadata_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                    if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&content) {
                        let name = path.file_name().unwrap().to_string_lossy().to_string();
                        backups.push((name, metadata));
                    }
                }
            }
        }
    }

    if backups.is_empty() {
        println!("No backups found.");
        return Ok(());
    }

    // Sort by date, newest first
    backups.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    // Filter to current project if in a project
    if let Some(ref project_name) = current_project_name {
        println!("Backups for current project ({}):", project_name.cyan());
        println!();

        let project_backups: Vec<_> = backups
            .iter()
            .filter(|(_, m)| &m.project_name == project_name)
            .collect();

        if project_backups.is_empty() {
            println!("  No backups found for this project.");
        } else {
            for (name, metadata) in project_backups {
                println!(
                    "  {} {} (schema v{})",
                    "•".cyan(),
                    name,
                    metadata.schema_version
                );
                println!(
                    "    Created: {}",
                    metadata.created_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
        }

        // Also show other projects
        let other_backups: Vec<_> = backups
            .iter()
            .filter(|(_, m)| &m.project_name != project_name)
            .collect();

        if !other_backups.is_empty() {
            println!();
            println!("Other projects:");
            for (name, metadata) in other_backups.iter().take(5) {
                println!(
                    "  {} {} - {} (v{})",
                    "○".dimmed(),
                    metadata.project_name,
                    name,
                    metadata.schema_version
                );
            }
            if other_backups.len() > 5 {
                println!("  ... and {} more", other_backups.len() - 5);
            }
        }
    } else {
        // Not in a project, show all
        println!("All backups:");
        for (name, metadata) in &backups {
            println!(
                "  {} {} - {} (v{})",
                "•".cyan(),
                metadata.project_name,
                name,
                metadata.schema_version
            );
            println!(
                "    Created: {}",
                metadata.created_at.format("%Y-%m-%d %H:%M:%S")
            );
        }
    }

    println!();
    println!("To restore: {}", "proj rollback --schema".cyan());

    Ok(())
}

/// Restore schema from a backup
fn restore_schema_backup() -> Result<()> {
    let project_root = get_project_root()?;
    let tracking_path = project_root.join(".tracking");

    // Get current project name
    let config_path = tracking_path.join("config.json");
    let project_name = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;
        config
            .get("name")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        bail!("No proj configuration found in this directory");
    };

    // Find backups for this project
    let backups_dir = get_backups_dir()?;
    if !backups_dir.exists() {
        bail!("No backups found. Backups are created automatically before schema upgrades.");
    }

    let mut backups: Vec<(PathBuf, BackupMetadata)> = Vec::new();

    for entry in std::fs::read_dir(&backups_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let metadata_path = path.join("backup_metadata.json");
            if metadata_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                    if let Ok(metadata) = serde_json::from_str::<BackupMetadata>(&content) {
                        if metadata.project_name == project_name {
                            backups.push((path, metadata));
                        }
                    }
                }
            }
        }
    }

    if backups.is_empty() {
        bail!(
            "No backups found for project '{}'. Backups are created automatically before schema upgrades.",
            project_name
        );
    }

    // Sort by date, newest first
    backups.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    // Let user select which backup to restore
    let options: Vec<String> = backups
        .iter()
        .map(|(path, meta)| {
            format!(
                "{} (v{}) - {}",
                path.file_name().unwrap().to_string_lossy(),
                meta.schema_version,
                meta.created_at.format("%Y-%m-%d %H:%M")
            )
        })
        .collect();

    println!("{}", "Schema Rollback".bold());
    println!();

    let selection = Select::new()
        .with_prompt("Select backup to restore")
        .items(&options)
        .default(0)
        .interact()?;

    let (backup_path, backup_metadata) = &backups[selection];

    println!();
    println!(
        "This will restore .tracking/ from backup created on {}",
        backup_metadata.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Schema version will be rolled back to: v{}",
        backup_metadata.schema_version.yellow()
    );
    println!();
    println!(
        "{}",
        "Warning: Current .tracking/ data will be lost!".red()
    );
    println!();

    if !Confirm::new()
        .with_prompt("Are you sure you want to restore from this backup?")
        .default(false)
        .interact()?
    {
        println!("Cancelled.");
        return Ok(());
    }

    // Remove current .tracking
    if tracking_path.exists() {
        std::fs::remove_dir_all(&tracking_path)
            .with_context(|| "Failed to remove current .tracking directory")?;
    }

    // Copy backup to .tracking
    copy_dir_recursive(backup_path, &tracking_path)
        .with_context(|| "Failed to restore from backup")?;

    // Remove the backup metadata file from the restored directory
    let restored_metadata = tracking_path.join("backup_metadata.json");
    if restored_metadata.exists() {
        std::fs::remove_file(&restored_metadata)?;
    }

    println!();
    println!(
        "{} Schema restored from backup (v{})",
        "✓".green(),
        backup_metadata.schema_version
    );

    Ok(())
}

// ============================================================================
// Release Rollback (existing functionality)
// ============================================================================

fn release_rollback(version: Option<String>) -> Result<()> {
    // Get version to rollback
    let version = match version {
        Some(v) => v.trim_start_matches('v').to_string(),
        None => get_latest_release()?,
    };

    let tag = format!("v{}", version);

    println!("{}", "=== Release Rollback ===".bold());
    println!();
    println!("This will rollback release: {}", tag.red());
    println!();
    println!("Actions:");
    println!("  1. Delete GitHub release {}", tag);
    println!("  2. Delete git tag {} (local and remote)", tag);
    println!();
    println!(
        "{}",
        "Note: Users who already downloaded this version will still have it.".dimmed()
    );
    println!();

    if !Confirm::new()
        .with_prompt("Are you sure you want to rollback this release?")
        .default(false)
        .interact()?
    {
        println!("Aborted.");
        return Ok(());
    }

    println!();

    // Delete GitHub release
    print!("Deleting GitHub release... ");
    match run_command("gh", &["release", "delete", &tag, "--yes"]) {
        Ok(_) => println!("{}", "✓".green()),
        Err(e) => println!("{} ({})", "⚠".yellow(), e),
    }

    // Delete remote tag
    print!("Deleting remote tag... ");
    match run_command("git", &["push", "--delete", "origin", &tag]) {
        Ok(_) => println!("{}", "✓".green()),
        Err(e) => println!("{} ({})", "⚠".yellow(), e),
    }

    // Delete local tag
    print!("Deleting local tag... ");
    match run_command("git", &["tag", "-d", &tag]) {
        Ok(_) => println!("{}", "✓".green()),
        Err(e) => println!("{} ({})", "⚠".yellow(), e),
    }

    println!();
    println!("{} Release {} rolled back", "✓".green(), tag);
    println!();
    println!("Next steps:");
    println!("  • Fix the issue that caused the rollback");
    println!("  • Bump version in Cargo.toml to a new version");
    println!("  • Push to main - a new release will be created automatically");

    Ok(())
}

/// Get the latest release version from GitHub
fn get_latest_release() -> Result<String> {
    let output = Command::new("gh")
        .args(["release", "list", "--limit", "1", "--json", "tagName"])
        .output()
        .with_context(|| "Failed to run gh command")?;

    if !output.status.success() {
        bail!("Failed to get releases from GitHub");
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let releases = json
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid response"))?;

    if releases.is_empty() {
        bail!("No releases found");
    }

    let tag = releases[0]["tagName"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No tag name found"))?;

    Ok(tag.trim_start_matches('v').to_string())
}

/// Run a command and return success/failure
fn run_command(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run: {} {:?}", cmd, args))?;

    if !status.status.success() {
        bail!(
            "Command failed: {}",
            String::from_utf8_lossy(&status.stderr)
        );
    }

    Ok(())
}
