// Path utilities - Full implementation in Task #8

use anyhow::{bail, Result};
use std::path::PathBuf;

/// Gets the project root directory by looking for .tracking/
pub fn get_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;

    loop {
        let tracking_dir = current.join(".tracking");
        if tracking_dir.exists() && tracking_dir.is_dir() {
            return Ok(current);
        }

        if !current.pop() {
            bail!("Not in a proj-tracked project (no .tracking/ directory found)")
        }
    }
}

/// Gets the path to the tracking database
pub fn get_tracking_db_path() -> Result<PathBuf> {
    let root = get_project_root()?;
    Ok(root.join(".tracking").join("tracking.db"))
}

/// Gets the path to the project config
pub fn get_config_path() -> Result<PathBuf> {
    let root = get_project_root()?;
    Ok(root.join(".tracking").join("config.json"))
}

/// Gets the global proj directory (~/.proj/)
pub fn get_global_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    Ok(home.join(".proj"))
}

/// Gets the path to the global registry
pub fn get_registry_path() -> Result<PathBuf> {
    Ok(get_global_dir()?.join("registry.json"))
}

/// Gets the backups directory
pub fn get_backups_dir() -> Result<PathBuf> {
    Ok(get_global_dir()?.join("backups"))
}

/// Gets the pending update directory for auto-update staging
pub fn get_pending_update_dir() -> Result<PathBuf> {
    Ok(get_global_dir()?.join("pending_update"))
}

/// Ensures a directory exists, creating it if necessary
pub fn ensure_dir(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}
