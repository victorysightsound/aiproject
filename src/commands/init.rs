// Init command - initialize a new project

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};

use crate::config::{ProjectConfig, Registry, RegistryEntry};
use crate::database::open_database;
use crate::paths::{ensure_dir, get_registry_path};
use crate::schema::TRACKING_SCHEMA;
use crate::SCHEMA_VERSION;

/// Project types
const PROJECT_TYPES: &[&str] = &[
    "rust",
    "python",
    "javascript",
    "web",
    "documentation",
    "other",
];

pub fn run() -> Result<()> {
    let project_root = std::env::current_dir()?;
    let tracking_path = project_root.join(".tracking");

    if tracking_path.exists() {
        println!("Project already initialized. Use 'proj status' to see current state.");
        return Ok(());
    }

    println!("Initializing project in: {}", project_root.display());

    // Detect project type
    let detected_type = detect_project_type(&project_root);
    let project_type = if let Some(detected) = &detected_type {
        println!("\nDetected project type: {}", detected.replace('_', " "));
        if Confirm::new()
            .with_prompt("Is this correct?")
            .default(true)
            .interact()?
        {
            detected.clone()
        } else {
            prompt_project_type()?
        }
    } else {
        prompt_project_type()?
    };

    // Get project name
    let default_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    let project_name: String = Input::new()
        .with_prompt("Project name")
        .default(default_name)
        .interact_text()?;

    // Get optional description
    let description: String = Input::new()
        .with_prompt("Description (optional)")
        .allow_empty(true)
        .interact_text()?;

    let description = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    // Create .tracking directory
    println!("\nCreating project structure...");
    ensure_dir(&tracking_path)?;

    // Create config.json
    let config = ProjectConfig {
        name: project_name.clone(),
        project_type: project_type.clone(),
        description,
        schema_version: SCHEMA_VERSION.to_string(),
        auto_backup: true,
        auto_session: true,
    };

    let config_path = tracking_path.join("config.json");
    let config_json = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, config_json)?;
    println!("  {} config.json", "✓".green());

    // Create tracking.db
    let db_path = tracking_path.join("tracking.db");
    let conn = open_database(&db_path)?;

    // Initialize schema
    conn.execute_batch(TRACKING_SCHEMA)
        .with_context(|| "Failed to initialize tracking database schema")?;

    // Store schema version
    conn.execute(
        "INSERT INTO project_meta (key, value) VALUES ('schema_version', ?1)",
        [SCHEMA_VERSION],
    )?;
    println!("  {} tracking.db", "✓".green());

    // Register project in global registry
    if let Err(e) = register_project(&project_root, &project_name, &project_type) {
        println!("  {} Could not register project: {}", "⚠".yellow(), e);
    } else {
        println!("  {} Registered in global registry", "✓".green());
    }

    // Ensure session rule in global AGENTS.md
    if let Err(e) = ensure_agents_session_rule() {
        println!(
            "  {} Could not update global AGENTS.md: {}",
            "⚠".yellow(),
            e
        );
    } else {
        println!("  {} Session rule in global AGENTS.md", "✓".green());
    }

    println!(
        "\n{} Project '{}' initialized successfully!",
        "✓".green(),
        project_name
    );
    println!("\nNext steps:");
    println!("  • Run 'proj status' to see project state");
    println!("  • Run 'proj log decision \"topic\" \"decision\" \"why\"' to log decisions");
    println!("  • Run 'proj task add \"description\"' to add tasks");

    Ok(())
}

/// Detect project type from files in directory
fn detect_project_type(path: &PathBuf) -> Option<String> {
    if path.join("Cargo.toml").exists() {
        return Some("rust".to_string());
    }
    if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
        return Some("python".to_string());
    }
    if path.join("package.json").exists() {
        let pkg = std::fs::read_to_string(path.join("package.json")).ok()?;
        if pkg.contains("react") || pkg.contains("vue") || pkg.contains("svelte") {
            return Some("web".to_string());
        }
        return Some("javascript".to_string());
    }
    if path.join("index.html").exists() {
        return Some("web".to_string());
    }
    if path.join("README.md").exists() || path.join("docs").is_dir() {
        return Some("documentation".to_string());
    }
    None
}

/// Prompt user to select project type
fn prompt_project_type() -> Result<String> {
    let selection = Select::new()
        .with_prompt("Select project type")
        .items(PROJECT_TYPES)
        .default(0)
        .interact()?;

    Ok(PROJECT_TYPES[selection].to_string())
}

/// Register project in global registry
fn register_project(path: &PathBuf, name: &str, project_type: &str) -> Result<()> {
    let registry_path = get_registry_path()?;

    // Ensure registry directory exists
    if let Some(parent) = registry_path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }

    // Load or create registry
    let mut registry = if registry_path.exists() {
        let content = std::fs::read_to_string(&registry_path)?;
        serde_json::from_str::<Registry>(&content)?
    } else {
        Registry::default()
    };

    let path_str = path.to_string_lossy().to_string();

    // Check if already registered
    if registry
        .registered_projects
        .iter()
        .any(|p| p.path == path_str)
    {
        return Ok(()); // Already registered
    }

    // Add to registry
    registry.registered_projects.push(RegistryEntry {
        path: path_str,
        name: name.to_string(),
        project_type: project_type.to_string(),
        registered_at: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        schema_version: SCHEMA_VERSION.to_string(),
    });

    // Save registry
    let content = serde_json::to_string_pretty(&registry)?;
    std::fs::write(&registry_path, content)?;

    Ok(())
}

/// Session management rule to add to global AGENTS.md
const SESSION_RULE: &str = r#"
## Session Management (proj)

**At the start of every conversation**, if the current directory has a `.tracking/` folder:
1. Run `proj status` BEFORE responding to the user's first message
2. This loads project context and starts session tracking
3. Stale sessions (8+ hours) auto-close automatically

If no `.tracking/` folder exists, skip this step.
"#;

/// Ensure session management rule exists in global AGENTS.md
fn ensure_agents_session_rule() -> Result<()> {
    // Try common locations for global AGENTS.md
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    let possible_paths = [
        home.join("projects/global/AGENTS.md"),
        home.join("AGENTS.md"),
    ];

    let agents_path = possible_paths.iter().find(|p| p.exists());

    let agents_path = match agents_path {
        Some(p) => p.clone(),
        None => return Ok(()), // No global AGENTS.md found, skip silently
    };

    // Read current content
    let content = std::fs::read_to_string(&agents_path)?;

    // Check if rule already exists
    if content.contains("## Session Management (proj)") {
        return Ok(()); // Already has the rule
    }

    // Find insertion point (after the initial instructions, before "## About Me" if it exists)
    let new_content = if let Some(pos) = content.find("## About Me") {
        let (before, after) = content.split_at(pos);
        format!("{}{}\n{}", before.trim_end(), SESSION_RULE, after)
    } else {
        // Append to end
        format!("{}\n{}", content.trim_end(), SESSION_RULE)
    };

    std::fs::write(&agents_path, new_content)?;

    Ok(())
}
