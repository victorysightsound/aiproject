// Uninstall command - remove proj from projects and system

use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::Confirm;

use crate::commands::shell;
use crate::config::Registry;
use crate::paths::get_registry_path;

pub fn run(shell_only: bool, project_only: bool, all: bool, force: bool) -> Result<()> {
    if shell_only {
        return uninstall_shell();
    }

    if project_only {
        return uninstall_current_project(force);
    }

    if all {
        return uninstall_all(force);
    }

    // No flags - show help
    println!("{}", "proj uninstall".bold());
    println!();
    println!("Remove proj tracking from projects and/or system.");
    println!();
    println!("Options:");
    println!(
        "  {} Remove shell hook only, keep all project data",
        "--shell".cyan()
    );
    println!(
        "  {} Remove .tracking/ from current project only",
        "--project".cyan()
    );
    println!(
        "  {} Remove shell hook + .tracking/ from ALL registered projects",
        "--all".cyan()
    );
    println!();
    println!("Examples:");
    println!("  proj uninstall --shell    # Remove shell integration");
    println!("  proj uninstall --project  # Remove tracking from current project");
    println!("  proj uninstall --all      # Complete removal from all projects");

    Ok(())
}

/// Remove shell hook only
fn uninstall_shell() -> Result<()> {
    shell::uninstall()
}

/// Remove .tracking from current project
fn uninstall_current_project(force: bool) -> Result<()> {
    let project_root = std::env::current_dir()?;
    let tracking_path = project_root.join(".tracking");

    if !tracking_path.exists() {
        println!("No proj tracking found in this directory.");
        return Ok(());
    }

    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("this project");

    if !force {
        println!("{}", "Uninstall proj from current project".bold());
        println!();
        println!(
            "This will {} .tracking/ directory from {}",
            "permanently delete".red(),
            project_name
        );
        println!();
        println!("Data that will be lost:");
        println!("  • All session history");
        println!("  • All logged decisions, notes, tasks");
        println!("  • Project configuration");
        println!();

        if !Confirm::new()
            .with_prompt("Are you sure you want to remove proj from this project?")
            .default(false)
            .interact()?
        {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Remove the tracking directory
    std::fs::remove_dir_all(&tracking_path)
        .with_context(|| format!("Failed to remove {:?}", tracking_path))?;

    println!("{} Removed .tracking/ from {}", "✓".green(), project_name);

    // Also remove from registry
    remove_from_registry(&project_root)?;

    Ok(())
}

/// Remove everything - shell hook + all registered projects
fn uninstall_all(force: bool) -> Result<()> {
    // Load registry to see what will be removed
    let registry = load_registry()?;
    let project_count = registry.registered_projects.len();

    if project_count == 0 && !shell::is_installed() {
        println!("Nothing to uninstall.");
        return Ok(());
    }

    if !force {
        println!("{}", "Complete proj Uninstall".bold().red());
        println!();

        println!("This will:");
        if shell::is_installed() {
            println!("  • Remove shell integration from ~/.zshrc and/or ~/.bashrc");
        }
        if project_count > 0 {
            println!(
                "  • {} .tracking/ from {} registered project(s):",
                "Permanently delete".red(),
                project_count
            );
            for proj in &registry.registered_projects {
                println!("    - {} ({})", proj.name, proj.path);
            }
        }
        println!();

        if !Confirm::new()
            .with_prompt("Are you absolutely sure? This cannot be undone.")
            .default(false)
            .interact()?
        {
            println!("Cancelled.");
            return Ok(());
        }

        // Extra confirmation for complete removal
        if project_count > 0 {
            println!();
            println!(
                "{}",
                "Type 'DELETE ALL' to confirm complete removal:".yellow()
            );
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() != "DELETE ALL" {
                println!("Cancelled.");
                return Ok(());
            }
        }
    }

    println!();

    // Remove shell integration
    if shell::is_installed() {
        shell::uninstall()?;
    }

    // Remove tracking from all projects
    let mut removed_count = 0;
    let mut failed_count = 0;

    for proj in &registry.registered_projects {
        let tracking_path = Path::new(&proj.path).join(".tracking");
        if tracking_path.exists() {
            match std::fs::remove_dir_all(&tracking_path) {
                Ok(_) => {
                    println!("{} Removed tracking from: {}", "✓".green(), proj.name);
                    removed_count += 1;
                }
                Err(e) => {
                    println!("{} Failed to remove {}: {}", "✗".red(), proj.name, e);
                    failed_count += 1;
                }
            }
        } else {
            println!("{} Already removed: {}", "○".dimmed(), proj.name);
        }
    }

    // Clear the registry
    clear_registry()?;

    println!();
    println!(
        "{} Uninstall complete: {} removed, {} failed",
        "✓".green(),
        removed_count,
        failed_count
    );

    if failed_count > 0 {
        println!();
        println!("Some projects could not be removed. You may need to delete their .tracking/ directories manually.");
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

/// Remove a project from the registry
fn remove_from_registry(project_path: &Path) -> Result<()> {
    let registry_path = get_registry_path()?;

    if !registry_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&registry_path)?;
    let mut registry: Registry = serde_json::from_str(&content)?;

    let path_str = project_path.to_string_lossy().to_string();
    registry.registered_projects.retain(|p| p.path != path_str);

    let content = serde_json::to_string_pretty(&registry)?;
    std::fs::write(&registry_path, content)?;

    Ok(())
}

/// Clear the registry completely
fn clear_registry() -> Result<()> {
    let registry_path = get_registry_path()?;

    if registry_path.exists() {
        let empty_registry = Registry::default();
        let content = serde_json::to_string_pretty(&empty_registry)?;
        std::fs::write(&registry_path, content)?;
    }

    Ok(())
}
