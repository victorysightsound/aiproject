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

    // Ensure session rule in global AGENTS.md
    if !no_agents {
        if let Err(e) = ensure_agents_session_rule() {
            println!(
                "  {} Could not update global AGENTS.md: {}",
                "⚠".yellow(),
                e
            );
        } else {
            println!("  {} Session rule in global AGENTS.md", "✓".green());
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
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let zshrc = home.join(".zshrc");
    let bashrc = home.join(".bashrc");

    // Zsh hook code
    let zsh_hook = r#"# >>> proj shell integration >>>
# Automatically runs proj enter when cd'ing into a project with tracking
_proj_auto_enter() {
    if [[ -d ".tracking" ]] && command -v proj &> /dev/null; then
        proj enter
    fi
}
# Add to chpwd hooks if not already present
if [[ -z "${chpwd_functions[(r)_proj_auto_enter]}" ]]; then
    chpwd_functions+=(_proj_auto_enter)
fi
# <<< proj shell integration <<<"#;

    // Bash hook code
    let bash_hook = r#"# >>> proj shell integration >>>
# Automatically runs proj enter when cd'ing into a project with tracking
_proj_last_dir=""
_proj_auto_enter() {
    if [[ "$PWD" != "$_proj_last_dir" ]]; then
        _proj_last_dir="$PWD"
        if [[ -d ".tracking" ]] && command -v proj &> /dev/null; then
            proj enter
        fi
    fi
}
# Add to PROMPT_COMMAND if not already present
if [[ "$PROMPT_COMMAND" != *"_proj_auto_enter"* ]]; then
    PROMPT_COMMAND="_proj_auto_enter${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi
# <<< proj shell integration <<<"#;

    let hook_marker = "# >>> proj shell integration >>>";

    // Install for zsh if exists and not already installed
    if zshrc.exists() {
        let content = std::fs::read_to_string(&zshrc).unwrap_or_default();
        if !content.contains(hook_marker) {
            let mut new_content = content;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push('\n');
            new_content.push_str(zsh_hook);
            new_content.push('\n');
            std::fs::write(&zshrc, new_content)?;
        }
    }

    // Install for bash if exists and not already installed
    if bashrc.exists() {
        let content = std::fs::read_to_string(&bashrc).unwrap_or_default();
        if !content.contains(hook_marker) {
            let mut new_content = content;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push('\n');
            new_content.push_str(bash_hook);
            new_content.push('\n');
            std::fs::write(&bashrc, new_content)?;
        }
    }

    Ok(())
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

/// Session management rule to add to global AGENTS.md
const SESSION_RULE: &str = r#"
## Session Management (proj)

**At the start of every conversation**, if the current directory has a `.tracking/` folder:
1. Run `proj status` BEFORE responding to the user's first message
2. This loads project context and starts session tracking
3. Stale sessions (8+ hours) auto-close automatically

If no `.tracking/` folder exists, skip this step.

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

### Ending Sessions

Before ending, the session activity will be displayed. Review it to ensure nothing was missed.

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
