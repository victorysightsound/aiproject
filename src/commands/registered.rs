// Registered command - list all registered projects

use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::Registry;
use crate::paths::get_registry_path;

pub fn run() -> Result<()> {
    let registry = load_registry()?;

    if registry.registered_projects.is_empty() {
        println!("No projects registered.");
        println!("Run 'proj init' or 'proj register' in a project directory.");
        return Ok(());
    }

    println!("\nRegistered Projects ({}):\n", registry.registered_projects.len());

    for p in &registry.registered_projects {
        let path = Path::new(&p.path);
        let exists = path.exists();
        let has_tracking = if exists {
            path.join(".tracking").exists()
        } else {
            false
        };

        let status = if exists && has_tracking {
            "✓".green()
        } else if exists {
            "?".yellow()
        } else {
            "✗".red()
        };

        println!("  {} {}", status, p.name.bold());
        println!("      Type: {}", p.project_type);
        println!("      Path: {}", p.path);
        println!("      Schema: v{}", p.schema_version);
        println!();
    }

    Ok(())
}

/// Load the global registry
fn load_registry() -> Result<Registry> {
    let registry_path = get_registry_path()?;

    if !registry_path.exists() {
        return Ok(Registry::default());
    }

    let content = std::fs::read_to_string(&registry_path)
        .with_context(|| "Failed to read registry.json")?;
    let registry: Registry =
        serde_json::from_str(&content).with_context(|| "Failed to parse registry.json")?;
    Ok(registry)
}
