// Register command - add current project to global registry

use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;

use crate::config::{ProjectConfig, Registry, RegistryEntry};
use crate::paths::{ensure_dir, get_config_path, get_project_root, get_registry_path};

pub fn run() -> Result<()> {
    // Load project config
    let config = load_config()?;
    let project_root = get_project_root()?;
    let project_path = project_root.to_string_lossy().to_string();

    // Load or create registry
    let mut registry = load_or_create_registry()?;

    // Check if already registered
    if registry.registered_projects.iter().any(|p| p.path == project_path) {
        println!("Already registered: {}", config.name);
        return Ok(());
    }

    // Add to registry
    registry.registered_projects.push(RegistryEntry {
        path: project_path,
        name: config.name.clone(),
        project_type: config.project_type.clone(),
        registered_at: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        schema_version: config.schema_version.clone(),
    });

    // Save registry
    save_registry(&registry)?;

    println!("{} Registered: {}", "✓".green(), config.name);
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

/// Load existing registry or create a new one
fn load_or_create_registry() -> Result<Registry> {
    let registry_path = get_registry_path()?;

    // Ensure ~/.proj directory exists
    if let Some(parent) = registry_path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }

    if registry_path.exists() {
        let content = std::fs::read_to_string(&registry_path)
            .with_context(|| "Failed to read registry.json")?;
        let registry: Registry =
            serde_json::from_str(&content).with_context(|| "Failed to parse registry.json")?;
        Ok(registry)
    } else {
        Ok(Registry::default())
    }
}

/// Save registry to disk
fn save_registry(registry: &Registry) -> Result<()> {
    let registry_path = get_registry_path()?;

    // Ensure parent directory exists
    if let Some(parent) = registry_path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }

    let content = serde_json::to_string_pretty(registry)?;
    std::fs::write(&registry_path, content).with_context(|| "Failed to write registry.json")?;
    Ok(())
}
