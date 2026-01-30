// Init command - initialize a new project

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};

use crate::config::{ProjectConfig, Registry, RegistryEntry};
use crate::database::open_database;
use crate::docs_db;
use crate::paths::{ensure_dir, get_registry_path};
use crate::schema::init_tracking_schema;
use crate::schema_docs::DocType;
use crate::source_analyzer;
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

/// Check if we're running in an interactive terminal
fn is_interactive() -> bool {
    atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout)
}

/// Parse doc type from string
fn parse_doc_type(s: &str) -> DocType {
    match s.to_lowercase().as_str() {
        "framework" => DocType::Framework,
        "guide" => DocType::Guide,
        "api" => DocType::Api,
        "spec" => DocType::Spec,
        _ => DocType::Architecture,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    path: Option<String>,
    name: Option<String>,
    project_type: Option<String>,
    description: Option<String>,
    skip_docs: bool,
    docs_generate: bool,
    docs_import: bool,
    docs_new: bool,
    docs_type: String,
    auto_commit: bool,
    commit_mode: String,
    no_agents: bool,
    shell_hook: bool,
) -> Result<()> {
    // Determine project root - use --path if provided, otherwise current directory
    let project_root = if let Some(ref p) = path {
        let path_buf = PathBuf::from(p);
        // Convert to absolute path if relative
        if path_buf.is_absolute() {
            path_buf
        } else {
            std::env::current_dir()?.join(path_buf)
        }
    } else {
        std::env::current_dir()?
    };

    // Create directory if it doesn't exist
    if !project_root.exists() {
        println!("Creating directory: {}", project_root.display());
        std::fs::create_dir_all(&project_root)?;
    }

    let tracking_path = project_root.join(".tracking");

    if tracking_path.exists() {
        println!("Project already initialized. Use 'proj status' to see current state.");
        return Ok(());
    }

    // Determine if we're in non-interactive mode
    // Non-interactive if: name and type are provided, OR path is provided, OR we're not in a terminal
    let non_interactive =
        (name.is_some() && project_type.is_some()) || path.is_some() || !is_interactive();

    if non_interactive {
        run_non_interactive(
            project_root,
            tracking_path,
            name,
            project_type,
            description,
            skip_docs,
            docs_generate,
            docs_import,
            docs_new,
            docs_type,
            auto_commit,
            commit_mode,
            no_agents,
            shell_hook,
        )
    } else {
        run_interactive(project_root, tracking_path)
    }
}

#[allow(clippy::too_many_arguments)]
fn run_non_interactive(
    project_root: PathBuf,
    tracking_path: PathBuf,
    name: Option<String>,
    project_type: Option<String>,
    description: Option<String>,
    skip_docs: bool,
    docs_generate: bool,
    docs_import: bool,
    docs_new: bool,
    docs_type: String,
    auto_commit: bool,
    commit_mode: String,
    no_agents: bool,
    shell_hook: bool,
) -> Result<()> {
    // Validate required fields
    let project_name = name.unwrap_or_else(|| {
        project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string()
    });

    let project_type_str = project_type.unwrap_or_else(|| {
        detect_project_type(&project_root).unwrap_or_else(|| "other".to_string())
    });

    // Validate project type
    if !PROJECT_TYPES.contains(&project_type_str.as_str()) {
        bail!(
            "Invalid project type '{}'. Valid types: {}",
            project_type_str,
            PROJECT_TYPES.join(", ")
        );
    }

    // Validate commit mode
    if commit_mode != "prompt" && commit_mode != "auto" {
        bail!(
            "Invalid commit mode '{}'. Valid modes: prompt, auto",
            commit_mode
        );
    }

    println!("Initializing project in: {}", project_root.display());

    // Check if this is a git repository (for auto-commit)
    let is_git_repo = project_root.join(".git").exists();
    let effective_auto_commit = auto_commit && is_git_repo;

    // Create .tracking directory
    println!("\nCreating project structure...");
    ensure_dir(&tracking_path)?;

    // Create config.json
    let config = ProjectConfig {
        name: project_name.clone(),
        project_type: project_type_str.clone(),
        description: description.clone(),
        schema_version: SCHEMA_VERSION.to_string(),
        auto_backup: true,
        auto_session: true,
        auto_commit: effective_auto_commit,
        auto_commit_mode: commit_mode,
        auto_commit_on_task: true,
    };

    config.save()?;
    println!("  {} config.json", "✓".green());

    // Create tracking.db
    let db_path = tracking_path.join("tracking.db");
    let conn = open_database(&db_path)?;

    // Initialize schema
    init_tracking_schema(&conn).with_context(|| "Failed to initialize tracking database schema")?;
    println!("  {} tracking.db", "✓".green());

    // Register project in global registry
    if let Err(e) = register_project(&project_root, &project_name, &project_type_str) {
        println!("  {} Could not register project: {}", "⚠".yellow(), e);
    } else {
        println!("  {} Registered in global registry", "✓".green());
    }

    // Create project-local AGENTS.md with CLAUDE.md/GEMINI.md symlinks
    if !no_agents {
        if let Err(e) = setup_project_agents(&project_root) {
            println!(
                "  {} Could not setup project AGENTS.md: {}",
                "⚠".yellow(),
                e
            );
        } else {
            println!("  {} Project AGENTS.md + symlinks created", "✓".green());
        }
    }

    // Documentation setup
    let doc_type = parse_doc_type(&docs_type);

    if !skip_docs {
        if docs_generate {
            setup_docs_generate(&project_root, &project_name, doc_type)?;
        } else if docs_import {
            setup_docs_import(&project_root, &project_name, doc_type)?;
        } else if docs_new {
            setup_docs_skeleton(&project_root, &project_name, doc_type, description)?;
        }
        // If none specified, skip docs silently in non-interactive mode
    }

    // Install shell hook if requested and not already installed
    if shell_hook && !crate::commands::shell::is_installed() {
        println!("\nInstalling shell integration...");
        if let Err(e) = install_shell_hook_silent() {
            println!("  {} Could not install shell hook: {}", "⚠".yellow(), e);
        } else {
            println!("  {} Shell hook installed", "✓".green());
        }
    }

    println!(
        "\n{} Project '{}' initialized successfully!",
        "✓".green(),
        project_name
    );
    println!("\nNext steps:");
    println!("  • Run 'proj status' to see project state");
    println!("  • Run 'proj log decision \"topic\" \"decision\" \"why\"' to log decisions");

    Ok(())
}

/// Install shell hook without interactive prompts (for non-interactive mode)
fn install_shell_hook_silent() -> Result<()> {
    // Delegate to shell::install with force=true to skip prompts
    crate::commands::shell::install(true)
}

fn run_interactive(mut project_root: PathBuf, mut tracking_path: PathBuf) -> Result<()> {
    // Ask for project directory
    let current_dir = std::env::current_dir()?;
    let current_dir_str = current_dir.to_string_lossy().to_string();

    let chosen_path: String = Input::new()
        .with_prompt("Project directory")
        .default(current_dir_str.clone())
        .interact_text()?;

    // Check if user chose a different directory
    let chosen_path_buf = PathBuf::from(&chosen_path);
    let chosen_path_abs = if chosen_path_buf.is_absolute() {
        chosen_path_buf
    } else {
        current_dir.join(&chosen_path_buf)
    };

    let path_changed = chosen_path_abs != current_dir;

    if path_changed {
        // Create directory if it doesn't exist
        if !chosen_path_abs.exists() {
            println!("Creating directory: {}", chosen_path_abs.display());
            std::fs::create_dir_all(&chosen_path_abs)?;
        }
        project_root = chosen_path_abs.clone();
        tracking_path = project_root.join(".tracking");

        // Check if already initialized
        if tracking_path.exists() {
            println!(
                "Project already initialized at {}. Use 'proj status' to see current state.",
                project_root.display()
            );
            return Ok(());
        }
    }

    println!("\nInitializing project in: {}", project_root.display());

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
    let description_for_docs = description.clone();

    // Check if this is a git repository
    let is_git_repo = project_root.join(".git").exists();

    // Ask about auto-commit if it's a git repo
    let (auto_commit, auto_commit_mode) = if is_git_repo {
        println!();
        let enable_auto_commit = Confirm::new()
            .with_prompt(
                "Enable auto-commit on session end? (creates git commit with session summary)",
            )
            .default(false)
            .interact()?;

        if enable_auto_commit {
            let mode_options = &["Prompt each time (recommended)", "Fully automatic"];
            let mode_selection = Select::new()
                .with_prompt("Auto-commit mode")
                .items(mode_options)
                .default(0)
                .interact()?;

            let mode = if mode_selection == 0 {
                "prompt".to_string()
            } else {
                "auto".to_string()
            };

            (true, mode)
        } else {
            (false, "prompt".to_string())
        }
    } else {
        (false, "prompt".to_string())
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
        auto_commit,
        auto_commit_mode,
        auto_commit_on_task: true,
    };

    config.save()?;
    println!("  {} config.json", "✓".green());

    // Create tracking.db
    let db_path = tracking_path.join("tracking.db");
    let conn = open_database(&db_path)?;

    // Initialize schema
    init_tracking_schema(&conn).with_context(|| "Failed to initialize tracking database schema")?;
    println!("  {} tracking.db", "✓".green());

    // Register project in global registry
    if let Err(e) = register_project(&project_root, &project_name, &project_type) {
        println!("  {} Could not register project: {}", "⚠".yellow(), e);
    } else {
        println!("  {} Registered in global registry", "✓".green());
    }

    // Create project-local AGENTS.md with CLAUDE.md/GEMINI.md symlinks
    if let Err(e) = setup_project_agents(&project_root) {
        println!(
            "  {} Could not setup project AGENTS.md: {}",
            "⚠".yellow(),
            e
        );
    } else {
        println!("  {} Project AGENTS.md + symlinks created", "✓".green());
    }

    // Documentation database setup
    println!("\n{}", "Documentation Setup".bold());
    println!("{}", "─".repeat(40));

    let docs_options = &[
        "Skip        - Set up documentation later",
        "Generate    - Analyze codebase and generate docs",
        "Import      - Import from existing markdown files",
        "New Project - Create documentation skeleton",
    ];

    let docs_selection = Select::new()
        .with_prompt("Set up project documentation?")
        .items(docs_options)
        .default(0)
        .interact()?;

    match docs_selection {
        0 => {
            println!("  {} Skipping documentation setup", "ℹ".blue());
            println!("  Run 'proj docs init' later to set up documentation.");
        }
        1 => {
            let doc_type = select_doc_type()?;
            setup_docs_generate(&project_root, &project_name, doc_type)?;
        }
        2 => {
            let doc_type = select_doc_type()?;
            setup_docs_import(&project_root, &project_name, doc_type)?;
        }
        3 => {
            let doc_type = select_doc_type()?;
            setup_docs_skeleton(&project_root, &project_name, doc_type, description_for_docs)?;
        }
        _ => {}
    }

    // Offer to install shell hook if not already installed
    if !crate::commands::shell::is_installed() {
        println!();
        println!("{}", "Shell Integration".bold());
        println!("{}", "─".repeat(40));
        println!("Sessions can start automatically when you cd into tracked projects.");

        if Confirm::new()
            .with_prompt("Enable automatic session tracking? (installs shell hook)")
            .default(true)
            .interact()?
        {
            if let Err(e) = install_shell_hook_silent() {
                println!("  {} Could not install shell hook: {}", "⚠".yellow(), e);
            } else {
                println!("  {} Shell hook installed", "✓".green());
                println!("  Open a new terminal or run 'source ~/.zshrc' to activate.");
            }
        } else {
            println!(
                "  {} Skipped. Run 'proj shell install' later if you change your mind.",
                "ℹ".blue()
            );
        }
    }

    println!(
        "\n{} Project '{}' initialized successfully!",
        "✓".green(),
        project_name
    );
    println!("\nNext steps:");
    println!("  • Run 'proj status' to see project state");
    println!("  • Run 'proj docs show' to view documentation");
    println!("  • Run 'proj log decision \"topic\" \"decision\" \"why\"' to log decisions");

    // If user chose a different directory, offer to show cd command
    if path_changed {
        println!();
        println!("{} To work on this project, run:", "→".cyan());
        println!("  cd {}", project_root.display());
    }

    Ok(())
}

/// Setup docs by generating from source
fn setup_docs_generate(
    project_root: &PathBuf,
    project_name: &str,
    doc_type: DocType,
) -> Result<()> {
    println!("\n  {}", "Analyzing codebase...".cyan());
    match source_analyzer::analyze_project(project_root) {
        Ok(structure) => {
            if structure.modules.is_empty() {
                println!("  {} No analyzable code found, skipping.", "!".yellow());
            } else {
                println!(
                    "  {} Detected {} ({} files, {} items)",
                    "✓".green(),
                    structure.language.as_str(),
                    structure.file_count,
                    structure.modules.len()
                );

                let db_filename = crate::schema_docs::docs_db_filename(project_name, doc_type);
                let db_path = project_root.join(&db_filename);

                let doc_conn = docs_db::create_docs_db(&db_path, project_name, doc_type)?;
                let sections = source_analyzer::generate_sections(&structure);

                for section in &sections {
                    docs_db::insert_section(
                        &doc_conn,
                        &section.section_id,
                        &section.title,
                        None,
                        section.level,
                        section.sort_order,
                        &section.content,
                        section.generated,
                        section.source_file.as_deref(),
                    )?;
                }

                crate::schema_docs::set_meta(&doc_conn, "generated_from", "source_analysis")?;
                crate::schema_docs::set_meta(&doc_conn, "language", structure.language.as_str())?;

                println!(
                    "  {} {} ({} sections)",
                    "✓".green(),
                    db_filename,
                    sections.len()
                );
            }
        }
        Err(e) => {
            println!("  {} Could not analyze codebase: {}", "⚠".yellow(), e);
        }
    }
    Ok(())
}

/// Setup docs by importing markdown
fn setup_docs_import(project_root: &PathBuf, project_name: &str, doc_type: DocType) -> Result<()> {
    let md_files = find_markdown_files(project_root);
    if md_files.is_empty() {
        println!("  {} No markdown files found, skipping.", "!".yellow());
    } else {
        println!("  Found {} markdown files", md_files.len());

        let db_filename = crate::schema_docs::docs_db_filename(project_name, doc_type);
        let db_path = project_root.join(&db_filename);

        let doc_conn = docs_db::create_docs_db(&db_path, project_name, doc_type)?;

        let mut total_sections = 0;
        for file_path in &md_files {
            match import_markdown_to_db(&doc_conn, file_path, project_root) {
                Ok(count) => total_sections += count,
                Err(e) => println!(
                    "  {} Failed to import {:?}: {}",
                    "⚠".yellow(),
                    file_path.file_name().unwrap_or_default(),
                    e
                ),
            }
        }

        crate::schema_docs::set_meta(&doc_conn, "generated_from", "import")?;
        println!(
            "  {} {} ({} sections)",
            "✓".green(),
            db_filename,
            total_sections
        );
    }
    Ok(())
}

/// Setup docs with skeleton
fn setup_docs_skeleton(
    project_root: &PathBuf,
    project_name: &str,
    doc_type: DocType,
    description: Option<String>,
) -> Result<()> {
    let db_filename = crate::schema_docs::docs_db_filename(project_name, doc_type);
    let db_path = project_root.join(&db_filename);

    let doc_conn = docs_db::create_docs_db(&db_path, project_name, doc_type)?;

    let desc = description.unwrap_or_else(|| format!("Documentation for {}", project_name));
    let sections = vec![
        ("1", "Overview", 1, desc),
        (
            "2",
            "Architecture",
            1,
            "Describe the system architecture and design decisions.".to_string(),
        ),
        (
            "3",
            "Components",
            1,
            "List and describe the main components.".to_string(),
        ),
        (
            "4",
            "Data Model",
            1,
            "Describe the data structures and storage.".to_string(),
        ),
        (
            "5",
            "API Reference",
            1,
            "Document the public API.".to_string(),
        ),
        (
            "6",
            "Configuration",
            1,
            "Document configuration options.".to_string(),
        ),
        (
            "7",
            "Development",
            1,
            "Build instructions and contribution guidelines.".to_string(),
        ),
    ];

    let mut sort_order = 0;
    for (section_id, title, level, content) in &sections {
        sort_order += 1;
        docs_db::insert_section(
            &doc_conn, section_id, title, None, *level, sort_order, content, true, None,
        )?;
    }

    crate::schema_docs::set_meta(&doc_conn, "generated_from", "skeleton")?;
    println!(
        "  {} {} ({} sections)",
        "✓".green(),
        db_filename,
        sections.len()
    );
    Ok(())
}

/// Select documentation type
fn select_doc_type() -> Result<DocType> {
    let type_options = &[
        "Architecture - System/application design",
        "Framework    - Library/framework docs",
        "Guide        - User guide or manual",
        "API          - API reference",
        "Spec         - Specification document",
    ];

    let selection = Select::new()
        .with_prompt("Documentation type")
        .items(type_options)
        .default(0)
        .interact()?;

    Ok(match selection {
        0 => DocType::Architecture,
        1 => DocType::Framework,
        2 => DocType::Guide,
        3 => DocType::Api,
        4 => DocType::Spec,
        _ => DocType::Architecture,
    })
}

/// Find markdown files in project
fn find_markdown_files(project_root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();

    let check_files = [
        "README.md",
        "ARCHITECTURE.md",
        "CONTRIBUTING.md",
        "API.md",
        "GUIDE.md",
    ];
    for file in &check_files {
        let path = project_root.join(file);
        if path.exists() {
            files.push(path);
        }
    }

    let docs_dir = project_root.join("docs");
    if docs_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&docs_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "md") {
                    files.push(path);
                }
            }
        }
    }

    files
}

/// Import a markdown file into docs database
fn import_markdown_to_db(
    conn: &rusqlite::Connection,
    file_path: &std::path::Path,
    project_root: &std::path::Path,
) -> Result<usize> {
    let content = std::fs::read_to_string(file_path)?;
    let relative_path = file_path
        .strip_prefix(project_root)
        .unwrap_or(file_path)
        .to_string_lossy();

    let mut sections = Vec::new();
    let mut current_section: Option<(i32, String, String)> = None;

    for line in content.lines() {
        if let Some(level) = detect_heading(line) {
            if let Some((lvl, title, content)) = current_section.take() {
                sections.push((lvl, title, content.trim().to_string()));
            }
            let title = line.trim_start_matches('#').trim().to_string();
            current_section = Some((level, title, String::new()));
        } else if let Some((_, _, ref mut content)) = current_section {
            content.push_str(line);
            content.push('\n');
        }
    }

    if let Some((level, title, content)) = current_section {
        sections.push((level, title, content.trim().to_string()));
    }

    let mut sort_order = 0;
    for (level, title, content) in &sections {
        sort_order += 1;
        let section_id = format!("{}", sort_order);
        docs_db::insert_section(
            conn,
            &section_id,
            title,
            None,
            *level,
            sort_order,
            content,
            false,
            Some(&relative_path),
        )?;
    }

    Ok(sections.len())
}

/// Detect markdown heading level
fn detect_heading(line: &str) -> Option<i32> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') {
        let level = trimmed.chars().take_while(|&c| c == '#').count();
        if level <= 6 && trimmed.chars().nth(level) == Some(' ') {
            return Some(level as i32);
        }
    }
    None
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

/// Template for project-local AGENTS.md file with complete proj instructions
/// This is the single source of truth for AI agent instructions
pub const PROJECT_AGENTS_TEMPLATE: &str = r#"# Project Context & Rules
This file is the single source of truth for Gemini, Claude, and Codex agents.

## Project Tracking

This project uses `proj` for session and decision tracking. Follow these instructions to maintain project continuity.

### Session Management

**At the start of every conversation**, if the current directory has a `.tracking/` folder:
1. Run `proj status` BEFORE responding to the user's first message
2. This loads project context and starts session tracking
3. Stale sessions (8+ hours) auto-close automatically

If no `.tracking/` folder exists, skip this step.

**When switching projects mid-conversation** (e.g., user runs a command to change to a different project directory):
1. If the new directory has a `.tracking/` folder, run `proj status`
2. This ensures each project has proper session tracking even when moving between projects within the same conversation

### Logging During Sessions (IMPORTANT)

You MUST actively log decisions, tasks, and blockers as they occur. Do not wait until the end.

**Log decisions immediately when:**
- User chooses between options ("let's use X instead of Y")
- A technical approach is selected ("we'll implement it this way")
- Architecture or design choices are made
- Any "why" question is answered with a decision

```bash
proj log decision "<topic>" "<what was decided>" "<why/rationale>"
```

**Add tasks when:**
- User says: "todo", "need to", "should", "we'll have to", "don't forget to"
- Work is identified for later: "we can do that next", "that's for phase 2"
- Bugs or issues are found that aren't fixed immediately

```bash
proj task add "<description>" --priority <urgent|high|normal|low>
```

**Log blockers when:**
- User says: "blocked", "waiting on", "can't because", "need X before"
- External dependencies are identified
- Missing information prevents progress

```bash
proj log blocker "<what is blocking progress>"
```

### Two-Pass Logging

Use a two-pass approach to minimize data loss:

1. **First pass (during work):** Log decisions, tasks, and blockers as they happen
2. **Second pass (before ending):** Run `proj review` to catch anything missed

```bash
proj review
```

This shows what was logged vs git activity, with suggestions for items that may need logging.

If `proj status` shows a nudge like "Session active 30+ min, 0 decisions logged", either log decisions or run `proj review`.

### Ending Sessions

**Before ending a session**, you MUST:
1. **Commit any uncommitted changes** - Run `git status` to check for changes, then commit with a descriptive message
2. Review the session activity to ensure nothing was missed

**If session activity is empty:**
When `proj session end` shows "No activity was logged", you have three options:
1. **Add manually** - Run proj log/task commands yourself to capture what happened
2. **AI review** - Review the conversation and log items that should have been captured:
   - Look for decisions made (technical choices, architecture, approach)
   - Look for tasks identified (todos, future work, bugs found)
   - Look for blockers encountered (waiting on, can't proceed, need info)
   Then run the appropriate `proj log` or `proj task` commands
3. **End anyway** - Run: `proj session end --force "<summary>"`

Write substantive summaries (1-3 sentences) that answer "what was accomplished?" Avoid generic summaries like "reviewed status" - future sessions need specific context to resume effectively.

```bash
proj session end "<summary of what was accomplished>"
```

### Running proj Commands in LLM CLIs

Interactive wizards don't work in LLM CLI environments (Claude Code, Codex, etc.). When running proj commands that normally have wizards, **ask the user first**, then use command-line flags.

**proj init** (initializing a new project):
Ask user: project name, type (rust/python/javascript/web/documentation/other), description, enable auto-commit?, install shell hook?, set up documentation database? (skip/generate/import/new + doc type if not skip)
```bash
proj init --name "<name>" --type <type> --description "<desc>" [--auto-commit] [--shell-hook] [--skip-docs | --docs-generate | --docs-import | --docs-new --docs-type <type>]
```

**proj uninstall --project** (removing tracking):
Confirm with user: "Remove proj tracking from this project? This deletes all session history, decisions, and tasks."
```bash
proj uninstall --project --force
```

**proj uninstall --all** (complete removal):
Confirm with user: "Remove proj from ALL registered projects? This cannot be undone."
```bash
proj uninstall --all --force
```

**proj shell install** (shell integration):
Ask user: "Install shell hook for automatic session tracking?"
```bash
proj shell install --force
```

**proj session end** (ending session):
Either ask "What was accomplished?" or generate summary from conversation context.
```bash
proj session end "<summary>"
```
If session has no logged activity, either log missed items first or use `--force`.

**proj upgrade** (schema upgrades):
No questions needed - use `--auto` to skip confirmation.
```bash
proj upgrade --auto
```

**proj docs init** (documentation database):
Ask user: generate from source, import markdown, or create skeleton? What doc type?
```bash
proj docs init --generate --doc-type architecture
proj docs init --import --doc-type guide
proj docs init --new --name "<name>" --description "<desc>"
```

### Mid-Session Context Recall

When you need to recall previous decisions, check context, or understand project history:
- Use `proj context "<topic>"` to search decisions, notes, and git history
- Use `proj context "<topic>" --ranked` for relevance-scored results
- Use `proj context recent --recent` for the last 10 logged items
- Prefer `proj context` over re-reading files - it uses fewer tokens

Before making a decision that might duplicate or contradict a previous one, check:
```bash
proj context "<relevant topic>"
```

### Committing Changes

**After completing a task:**
Mark the task as completed and proj will automatically commit the changes:
```bash
proj task update <id> --status completed
```

**Before ending a session:**
Run `git status` to verify no uncommitted changes remain.

### Quick Reference

| Command | Purpose |
|---------|---------|
| `proj status` | Current state + auto-start session (syncs git commits) |
| `proj resume` | Detailed "where we left off" context |
| `proj context <topic>` | Query decisions, notes, and git commits |
| `proj context <topic> --ranked` | Relevance-scored search results |
| `proj context recent --recent` | Last 10 items across all tables |
| `proj tasks` | List current tasks |
| `proj review` | Cleanup pass - shows logged items vs git activity |
| `proj log decision "topic" "decision" "rationale"` | Record a decision |
| `proj session end "summary"` | Close session with summary |

### Database Queries (for AI agents)

For direct database access when more efficient:

```sql
-- Get last session summary (with structured data if available)
SELECT summary, structured_summary FROM sessions WHERE status = 'completed' ORDER BY ended_at DESC LIMIT 1;

-- Get active tasks
SELECT task_id, description, status, priority FROM tasks WHERE status NOT IN ('completed', 'cancelled') ORDER BY priority, created_at;

-- Get recent decisions on a topic
SELECT decision, rationale, created_at FROM decisions WHERE topic LIKE '%keyword%' AND status = 'active';

-- Get recent git commits
SELECT short_hash, message, files_changed, committed_at FROM git_commits ORDER BY committed_at DESC LIMIT 10;

-- Search all tracked content (decisions, notes, tasks, commit messages)
SELECT * FROM tracking_fts WHERE tracking_fts MATCH 'search term';
```

Tracking database: `.tracking/tracking.db`

### Principles

1. **Start with `proj status`** - never guess project state
2. **Log decisions when made** - not later when you might forget the rationale
3. **Keep task status current** - update as you work, not in batches
4. **End sessions with summaries** - future you (or another agent) will thank you
5. **Query before re-reading** - a SQL query uses fewer tokens than re-reading files
"#;

/// Setup project-local AGENTS.md with CLAUDE.md and GEMINI.md symlinks
/// This creates the unified agent configuration in the project directory
/// Setup project-local AGENTS.md with CLAUDE.md and GEMINI.md symlinks
/// This creates the unified agent configuration in the project directory
/// Public so it can be called from status.rs to ensure AGENTS.md exists
pub fn setup_project_agents(project_root: &std::path::Path) -> Result<()> {
    let agents_path = project_root.join("AGENTS.md");
    let claude_path = project_root.join("CLAUDE.md");
    let gemini_path = project_root.join("GEMINI.md");

    // Check if AGENTS.md already exists
    if agents_path.exists() {
        // Check if it already has proj tracking section
        let content = std::fs::read_to_string(&agents_path)?;
        if !content.contains("## Project Tracking") {
            // Append the project tracking section
            let new_content = format!(
                "{}\n\n{}",
                content.trim_end(),
                PROJECT_AGENTS_TEMPLATE
                    .trim_start_matches("# Project Context & Rules\n")
                    .trim_start_matches("This file is the single source of truth for Gemini, Claude, and Codex agents.\n\n")
            );
            std::fs::write(&agents_path, new_content)?;
        }
    } else {
        // Check if CLAUDE.md or GEMINI.md exist as real files (not symlinks)
        // If so, promote one to AGENTS.md (like unify-agents does)
        if claude_path.exists() && !claude_path.is_symlink() {
            std::fs::rename(&claude_path, &agents_path)?;
            // Append proj tracking if not present
            let content = std::fs::read_to_string(&agents_path)?;
            if !content.contains("## Project Tracking") {
                let new_content = format!(
                    "{}\n\n{}",
                    content.trim_end(),
                    PROJECT_AGENTS_TEMPLATE
                        .trim_start_matches("# Project Context & Rules\n")
                        .trim_start_matches("This file is the single source of truth for Gemini, Claude, and Codex agents.\n\n")
                );
                std::fs::write(&agents_path, new_content)?;
            }
        } else if gemini_path.exists() && !gemini_path.is_symlink() {
            std::fs::rename(&gemini_path, &agents_path)?;
            // Append proj tracking if not present
            let content = std::fs::read_to_string(&agents_path)?;
            if !content.contains("## Project Tracking") {
                let new_content = format!(
                    "{}\n\n{}",
                    content.trim_end(),
                    PROJECT_AGENTS_TEMPLATE
                        .trim_start_matches("# Project Context & Rules\n")
                        .trim_start_matches("This file is the single source of truth for Gemini, Claude, and Codex agents.\n\n")
                );
                std::fs::write(&agents_path, new_content)?;
            }
        } else {
            // Create fresh AGENTS.md
            std::fs::write(&agents_path, PROJECT_AGENTS_TEMPLATE)?;
        }
    }

    // Create/update symlinks for CLAUDE.md and GEMINI.md
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        // Handle CLAUDE.md
        if claude_path.exists() {
            if claude_path.is_symlink() {
                // Already a symlink, update it
                std::fs::remove_file(&claude_path)?;
            } else {
                // Real file - back it up
                let backup = project_root.join("CLAUDE.md.bak");
                std::fs::rename(&claude_path, &backup)?;
            }
        }
        symlink("AGENTS.md", &claude_path)?;

        // Handle GEMINI.md
        if gemini_path.exists() {
            if gemini_path.is_symlink() {
                // Already a symlink, update it
                std::fs::remove_file(&gemini_path)?;
            } else {
                // Real file - back it up
                let backup = project_root.join("GEMINI.md.bak");
                std::fs::rename(&gemini_path, &backup)?;
            }
        }
        symlink("AGENTS.md", &gemini_path)?;
    }

    #[cfg(windows)]
    {
        // On Windows, create copies instead of symlinks (symlinks require admin)
        if !claude_path.exists() {
            std::fs::copy(&agents_path, &claude_path)?;
        }
        if !gemini_path.exists() {
            std::fs::copy(&agents_path, &gemini_path)?;
        }
    }

    Ok(())
}

/// Update project-local AGENTS.md if its proj instructions are outdated
/// Called during `proj upgrade` to ensure AI logging instructions are current
/// Returns list of updated file paths
pub fn update_agents_rules_if_outdated() -> Result<Vec<String>> {
    let mut updated_files = Vec::new();

    // Only update the current project's AGENTS.md
    if let Ok(cwd) = std::env::current_dir() {
        let agents_path = cwd.join("AGENTS.md");
        if agents_path.exists() {
            if update_single_agents_file(&agents_path)? {
                updated_files.push(agents_path.display().to_string());
            }
        }
    }

    Ok(updated_files)
}

/// Update a single AGENTS.md file if its proj instructions are outdated
/// Returns true if the file was updated
fn update_single_agents_file(path: &std::path::Path) -> Result<bool> {
    let content = std::fs::read_to_string(path)?;

    // Check if the file has proj tracking section at all
    if !content.contains("## Project Tracking") {
        return Ok(false); // No proj section to update
    }

    // Check if it has the latest instructions by looking for key sections
    // v1.8.3+: "### Two-Pass Logging" and "### Mid-Session Context Recall"
    if content.contains("### Two-Pass Logging")
        && content.contains("### Mid-Session Context Recall")
    {
        return Ok(false); // Already has latest instructions
    }

    // Instructions are outdated - need to replace the entire Project Tracking section
    let section_start = match content.find("## Project Tracking") {
        Some(pos) => pos,
        None => return Ok(false),
    };

    // Find where the section ends (next ## heading or end of file)
    let after_header = section_start + "## Project Tracking".len();
    let section_end = content[after_header..]
        .find("\n## ")
        .map(|pos| after_header + pos)
        .unwrap_or(content.len());

    // Extract just the Project Tracking section from the template
    let tracking_section = PROJECT_AGENTS_TEMPLATE
        .trim_start_matches("# Project Context & Rules\n")
        .trim_start_matches(
            "This file is the single source of truth for Gemini, Claude, and Codex agents.\n\n",
        );

    // Build new content
    let before_section = &content[..section_start];
    let after_section = &content[section_end..];

    let new_content = format!(
        "{}\n\n{}{}",
        before_section.trim_end(),
        tracking_section.trim(),
        if after_section.trim().is_empty() {
            String::from("\n")
        } else {
            format!("\n\n{}", after_section.trim_start())
        }
    );

    std::fs::write(path, new_content)?;
    Ok(true)
}
