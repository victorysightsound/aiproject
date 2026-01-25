// Migrate command - convert existing project to proj format

use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};

use crate::config::{ProjectConfig, Registry, RegistryEntry};
use crate::database::open_database;
use crate::paths::{ensure_dir, get_registry_path};
use crate::schema::TRACKING_SCHEMA;
use crate::SCHEMA_VERSION;

pub fn run() -> Result<()> {
    let project_root = std::env::current_dir()?;
    let tracking_path = project_root.join(".tracking");

    if tracking_path.exists() {
        println!("Project already has proj tracking. Use 'proj status' instead.");
        return Ok(());
    }

    println!("Migrating project in: {}", project_root.display());

    // Detect existing content
    let existing = detect_existing_content(&project_root);

    println!("\nExisting content found:");
    if existing.readme {
        println!("  {} README.md", "✓".green());
    }
    if existing.agents_md {
        println!("  {} AGENTS.md", "✓".green());
    }
    if existing.markdown_count > 0 {
        println!(
            "  {} {} markdown files",
            "✓".green(),
            existing.markdown_count
        );
    }
    if existing.git {
        println!("  {} Git repository", "✓".green());
    }

    // Choose migration mode
    let modes = &["Smart - Auto-detect and confirm", "Quick - Minimal prompts"];
    let mode = Select::new()
        .with_prompt("Migration mode")
        .items(modes)
        .default(0)
        .interact()?;

    // Detect project type
    let detected_type = detect_project_type(&project_root);
    let project_type = if let Some(detected) = &detected_type {
        println!("\nDetected project type: {}", detected);
        if mode == 0 {
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
            detected.clone()
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

    // Create .tracking directory
    println!("\nMigrating project...");
    ensure_dir(&tracking_path)?;

    // Create config.json
    let config = ProjectConfig {
        name: project_name.clone(),
        project_type: project_type.clone(),
        description: None,
        schema_version: SCHEMA_VERSION.to_string(),
        auto_backup: true,
        auto_session: true,
        auto_commit: false,
        auto_commit_mode: "prompt".to_string(),
    };

    let config_path = tracking_path.join("config.json");
    let config_json = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, config_json)?;
    println!("  {} Created config.json", "✓".green());

    // Create tracking.db
    let db_path = tracking_path.join("tracking.db");
    let conn = open_database(&db_path)?;

    conn.execute_batch(TRACKING_SCHEMA)
        .with_context(|| "Failed to initialize tracking database schema")?;

    conn.execute(
        "INSERT INTO project_meta (key, value) VALUES ('schema_version', ?1)",
        [SCHEMA_VERSION],
    )?;

    // Log initial note about migration
    conn.execute(
        "INSERT INTO context_notes (category, title, content, status)
         VALUES ('migration', 'Project Migrated', 'Project converted to proj tracking format.', 'active')",
        [],
    )?;
    println!("  {} Created tracking.db", "✓".green());

    // Register in global registry
    if let Err(e) = register_project(&project_root, &project_name, &project_type) {
        println!("  {} Could not register: {}", "⚠".yellow(), e);
    } else {
        println!("  {} Registered globally", "✓".green());
    }

    println!(
        "\n{} Migration complete! Project '{}' is now tracked.",
        "✓".green(),
        project_name
    );
    println!("\nNext steps:");
    println!("  • Run 'proj status' to see project state");
    println!("  • Run 'proj log decision ...' to record existing decisions");

    Ok(())
}

/// Detected existing content
struct ExistingContent {
    readme: bool,
    agents_md: bool,
    markdown_count: usize,
    git: bool,
}

/// Detect existing content in project directory
fn detect_existing_content(path: &PathBuf) -> ExistingContent {
    let readme = path.join("README.md").exists();
    let agents_md = path.join("AGENTS.md").exists() || path.join("CLAUDE.md").exists();
    let git = path.join(".git").is_dir();

    let markdown_count = std::fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
                .count()
        })
        .unwrap_or(0);

    ExistingContent {
        readme,
        agents_md,
        markdown_count,
        git,
    }
}

/// Detect project type from files
fn detect_project_type(path: &PathBuf) -> Option<String> {
    if path.join("Cargo.toml").exists() {
        return Some("rust".to_string());
    }
    if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
        return Some("python".to_string());
    }
    if path.join("package.json").exists() {
        return Some("javascript".to_string());
    }
    None
}

/// Prompt for project type
fn prompt_project_type() -> Result<String> {
    let types = &[
        "rust",
        "python",
        "javascript",
        "web",
        "documentation",
        "other",
    ];
    let selection = Select::new()
        .with_prompt("Select project type")
        .items(types)
        .default(0)
        .interact()?;
    Ok(types[selection].to_string())
}

/// Register project in global registry
fn register_project(path: &PathBuf, name: &str, project_type: &str) -> Result<()> {
    let registry_path = get_registry_path()?;

    if let Some(parent) = registry_path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }

    let mut registry = if registry_path.exists() {
        let content = std::fs::read_to_string(&registry_path)?;
        serde_json::from_str::<Registry>(&content)?
    } else {
        Registry::default()
    };

    let path_str = path.to_string_lossy().to_string();

    if registry
        .registered_projects
        .iter()
        .any(|p| p.path == path_str)
    {
        return Ok(());
    }

    registry.registered_projects.push(RegistryEntry {
        path: path_str,
        name: name.to_string(),
        project_type: project_type.to_string(),
        registered_at: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        schema_version: SCHEMA_VERSION.to_string(),
    });

    let content = serde_json::to_string_pretty(&registry)?;
    std::fs::write(&registry_path, content)?;

    Ok(())
}
