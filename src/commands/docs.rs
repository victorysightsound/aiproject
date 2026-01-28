// Documentation database commands

use anyhow::{bail, Result};
use colored::Colorize;

use crate::cli::{DocsCommands, DocsSubcommand, DocsTermSubcommand};
use crate::docs_db;
use crate::paths::get_project_root;
use crate::schema_docs::DocType;

pub fn run(cmd: DocsCommands) -> Result<()> {
    match cmd.command {
        DocsSubcommand::Init {
            generate,
            import,
            new,
            doc_type,
            name,
            description,
        } => cmd_init(generate, import, new, &doc_type, name, description),
        DocsSubcommand::Status => cmd_status(),
        DocsSubcommand::Refresh { force } => cmd_refresh(force),
        DocsSubcommand::Search { query } => cmd_search(&query),
        DocsSubcommand::Export { format, output } => cmd_export(&format, output),
        DocsSubcommand::Show { section } => cmd_show(section),
        DocsSubcommand::Term(term_cmd) => match term_cmd.command {
            DocsTermSubcommand::Add {
                term,
                def,
                category,
            } => cmd_term_add(&term, &def, category.as_deref()),
            DocsTermSubcommand::List => cmd_term_list(),
            DocsTermSubcommand::Search { query } => cmd_term_search(&query),
        },
    }
}

/// Initialize documentation database - interactive wizard or non-interactive mode
fn cmd_init(
    generate: bool,
    import: bool,
    new: bool,
    doc_type_str: &str,
    name: Option<String>,
    description: Option<String>,
) -> Result<()> {
    let project_root = get_project_root()?;

    // Parse doc type
    let doc_type = match DocType::from_str(doc_type_str) {
        Some(doc_type) => doc_type,
        None => {
            bail!(
                "Unknown doc type '{}'. Use: architecture, framework, guide, api, spec",
                doc_type_str
            );
        }
    };

    // Non-interactive mode: --generate flag
    if generate {
        return cmd_init_generate_auto(&project_root, doc_type, name);
    }

    // Non-interactive mode: --import flag
    if import {
        return cmd_init_import_auto(&project_root, doc_type, name);
    }

    // Non-interactive mode: --new flag
    if new {
        return cmd_init_new_auto(&project_root, doc_type, name, description);
    }

    // Interactive mode
    use dialoguer::{Confirm, Select};

    // Check if docs db already exists
    if let Some(existing) = docs_db::find_docs_db(&project_root) {
        println!(
            "{} Documentation database already exists: {}",
            "!".yellow(),
            existing.file_name().unwrap_or_default().to_string_lossy()
        );

        if !Confirm::new()
            .with_prompt("Create a new one? (existing will not be deleted)")
            .default(false)
            .interact()?
        {
            return Ok(());
        }
    }

    println!("\n{}", "Project Documentation Setup".bold());
    println!("{}\n", "─".repeat(40));

    // Ask how they want to set up docs
    let options = &[
        "None        - Skip documentation database",
        "Import      - Import from existing markdown files",
        "Generate    - Analyze codebase and generate documentation",
        "New Project - Create from project description",
    ];

    let selection = Select::new()
        .with_prompt("How would you like to handle project documentation?")
        .items(options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            println!("\n{} Skipping documentation database.", "ℹ".blue());
            Ok(())
        }
        1 => cmd_init_import(&project_root),
        2 => cmd_init_generate(&project_root),
        3 => cmd_init_new(&project_root),
        _ => unreachable!(),
    }
}

/// Generate documentation from source analysis (non-interactive)
fn cmd_init_generate_auto(
    project_root: &std::path::Path,
    doc_type: DocType,
    name: Option<String>,
) -> Result<()> {
    println!("{}", "Analyzing codebase...".cyan());

    // Analyze the project
    let structure = crate::source_analyzer::analyze_project(project_root)?;

    println!(
        "{} Detected {} project ({} files, {} lines)",
        "✓".green(),
        structure.language.as_str(),
        structure.file_count,
        structure.total_lines
    );

    if structure.modules.is_empty() {
        bail!("No analyzable items found in the codebase.");
    }

    // Get project name
    let project_name = name.unwrap_or_else(|| structure.name.clone());

    // Create database
    let db_filename = crate::schema_docs::docs_db_filename(&project_name, doc_type);
    let db_path = project_root.join(&db_filename);

    println!("{}", format!("Creating {}...", db_filename).cyan());

    let conn = docs_db::create_docs_db(&db_path, &project_name, doc_type)?;

    // Generate sections
    let sections = crate::source_analyzer::generate_sections(&structure);

    // Insert sections
    for section in &sections {
        docs_db::insert_section(
            &conn,
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

    // Record generation info
    crate::schema_docs::set_meta(&conn, "generated_from", "source_analysis")?;
    crate::schema_docs::set_meta(&conn, "language", structure.language.as_str())?;
    crate::schema_docs::set_meta(
        &conn,
        "generated_at",
        &chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.6f")
            .to_string(),
    )?;

    println!(
        "{} Created {} with {} sections",
        "✓".green(),
        db_filename,
        sections.len()
    );

    Ok(())
}

/// Import from markdown files (non-interactive)
fn cmd_init_import_auto(
    project_root: &std::path::Path,
    doc_type: DocType,
    name: Option<String>,
) -> Result<()> {
    println!("{}", "Scanning for documentation files...".cyan());

    // Find markdown files
    let mut md_files: Vec<std::path::PathBuf> = Vec::new();

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
            md_files.push(path);
        }
    }

    // Check docs/ folder
    let docs_dir = project_root.join("docs");
    if docs_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&docs_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "md") {
                    md_files.push(path);
                }
            }
        }
    }

    if md_files.is_empty() {
        bail!("No markdown files found to import.");
    }

    println!("{} Found {} markdown files", "✓".green(), md_files.len());

    // Get project name
    let project_name = name.unwrap_or_else(|| {
        project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string()
    });

    // Create database
    let db_filename = crate::schema_docs::docs_db_filename(&project_name, doc_type);
    let db_path = project_root.join(&db_filename);

    println!("{}", format!("Creating {}...", db_filename).cyan());

    let conn = docs_db::create_docs_db(&db_path, &project_name, doc_type)?;

    // Import all files
    let mut total_sections = 0;
    for file_path in &md_files {
        match import_markdown_file(&conn, file_path, project_root) {
            Ok(count) => {
                println!(
                    "  {} {}",
                    "✓".green(),
                    file_path
                        .strip_prefix(project_root)
                        .unwrap_or(file_path)
                        .display()
                );
                total_sections += count;
            }
            Err(e) => {
                println!("  {} {}: {}", "✗".red(), file_path.display(), e);
            }
        }
    }

    // Record import source
    let import_source = md_files
        .iter()
        .map(|p| {
            p.strip_prefix(project_root)
                .unwrap_or(p)
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(", ");

    crate::schema_docs::set_meta(&conn, "imported_from", &import_source)?;

    println!(
        "{} Created {} with {} sections",
        "✓".green(),
        db_filename,
        total_sections
    );

    Ok(())
}

/// Import existing markdown files (interactive)
fn cmd_init_import(project_root: &std::path::Path) -> Result<()> {
    use dialoguer::{Input, MultiSelect, Select};

    println!("\n{}", "Scanning for documentation files...".cyan());

    // Find markdown files
    let mut md_files: Vec<std::path::PathBuf> = Vec::new();

    // Check common locations
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
            md_files.push(path);
        }
    }

    // Check docs/ folder
    let docs_dir = project_root.join("docs");
    if docs_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&docs_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "md") {
                    md_files.push(path);
                }
            }
        }
    }

    if md_files.is_empty() {
        println!("{} No markdown files found.", "!".yellow());
        println!("Consider using 'Generate' or 'New Project' instead.");
        return Ok(());
    }

    // Show found files
    println!("\n{}", "Found:".green());
    let file_labels: Vec<String> = md_files
        .iter()
        .map(|p| {
            let name = p.strip_prefix(project_root).unwrap_or(p);
            let size = std::fs::metadata(p)
                .map(|m| format!("({:.1} KB)", m.len() as f64 / 1024.0))
                .unwrap_or_default();
            format!("{} {}", name.display(), size)
        })
        .collect();

    let selections = MultiSelect::new()
        .with_prompt("Select files to import")
        .items(&file_labels)
        .interact()?;

    if selections.is_empty() {
        println!("{} No files selected.", "!".yellow());
        return Ok(());
    }

    // Ask for doc type
    let type_options = &[
        "Architecture - System/application design",
        "Framework    - Library/framework documentation",
        "Guide        - User guide or manual",
        "API          - API reference",
        "Spec         - Specification document",
    ];

    let type_selection = Select::new()
        .with_prompt("What type of documentation is this?")
        .items(type_options)
        .default(0)
        .interact()?;

    let doc_type = match type_selection {
        0 => DocType::Architecture,
        1 => DocType::Framework,
        2 => DocType::Guide,
        3 => DocType::Api,
        4 => DocType::Spec,
        _ => DocType::Architecture,
    };

    // Get project name
    let default_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let project_name: String = Input::new()
        .with_prompt("Project name")
        .default(default_name.to_string())
        .interact_text()?;

    // Create database
    let db_filename = crate::schema_docs::docs_db_filename(&project_name, doc_type);
    let db_path = project_root.join(&db_filename);

    println!("\n{}", format!("Creating {}...", db_filename).cyan());

    let conn = docs_db::create_docs_db(&db_path, &project_name, doc_type)?;

    // Import selected files
    let selected_files: Vec<_> = selections.iter().map(|&i| &md_files[i]).collect();
    let mut total_sections = 0;

    for file_path in &selected_files {
        match import_markdown_file(&conn, file_path, project_root) {
            Ok(count) => {
                println!(
                    "  {} Imported {} ({} sections)",
                    "✓".green(),
                    file_path
                        .strip_prefix(project_root)
                        .unwrap_or(file_path)
                        .display(),
                    count
                );
                total_sections += count;
            }
            Err(e) => {
                println!(
                    "  {} Failed to import {}: {}",
                    "✗".red(),
                    file_path.display(),
                    e
                );
            }
        }
    }

    // Record import source
    let import_source = selected_files
        .iter()
        .map(|p| {
            p.strip_prefix(project_root)
                .unwrap_or(p)
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(", ");

    crate::schema_docs::set_meta(&conn, "imported_from", &import_source)?;
    crate::schema_docs::set_meta(
        &conn,
        "imported_at",
        &chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.6f")
            .to_string(),
    )?;

    println!(
        "\n{} Created {} with {} sections",
        "✓".green(),
        db_filename,
        total_sections
    );

    Ok(())
}

/// Import a markdown file into the database
fn import_markdown_file(
    conn: &rusqlite::Connection,
    file_path: &std::path::Path,
    project_root: &std::path::Path,
) -> Result<usize> {
    let content = std::fs::read_to_string(file_path)?;
    let relative_path = file_path
        .strip_prefix(project_root)
        .unwrap_or(file_path)
        .to_string_lossy();

    // Parse markdown into sections based on headings
    let mut sections = Vec::new();
    let mut current_section: Option<(i32, String, String)> = None; // (level, title, content)
    let mut section_counter: Vec<i32> = vec![0; 6]; // Track section numbers at each level

    for line in content.lines() {
        if let Some(heading_level) = detect_heading(line) {
            // Save previous section if exists
            if let Some((level, title, content)) = current_section.take() {
                sections.push((level, title, content.trim().to_string()));
            }

            // Update section counter
            section_counter[heading_level as usize - 1] += 1;
            // Reset lower level counters
            for i in heading_level as usize..6 {
                section_counter[i] = 0;
            }

            let title = line.trim_start_matches('#').trim().to_string();

            current_section = Some((heading_level, title, String::new()));
        } else if let Some((_, _, ref mut content)) = current_section {
            content.push_str(line);
            content.push('\n');
        }
    }

    // Don't forget the last section
    if let Some((level, title, content)) = current_section {
        sections.push((level, title, content.trim().to_string()));
    }

    // Insert sections into database
    let mut sort_order = 0;
    let mut section_ids: Vec<String> = Vec::new();

    for (level, title, content) in &sections {
        sort_order += 1;

        // Generate section_id
        let section_id = format!("{}", sort_order);

        // Determine parent_id
        let parent_id = if *level > 1 && !section_ids.is_empty() {
            // Find the most recent section with a lower level
            None // Simplified for now - full hierarchy tracking is complex
        } else {
            None
        };

        docs_db::insert_section(
            conn,
            &section_id,
            title,
            parent_id,
            *level,
            sort_order,
            content,
            false, // Not generated, imported
            Some(&relative_path),
        )?;

        section_ids.push(section_id);
    }

    Ok(sections.len())
}

/// Detect if a line is a markdown heading and return its level
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

/// Create skeleton documentation (non-interactive)
fn cmd_init_new_auto(
    project_root: &std::path::Path,
    doc_type: DocType,
    name: Option<String>,
    description: Option<String>,
) -> Result<()> {
    // Get project name
    let project_name = name.unwrap_or_else(|| {
        project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string()
    });

    let desc = description.unwrap_or_else(|| format!("Documentation for {}", project_name));

    // Create database
    let db_filename = crate::schema_docs::docs_db_filename(&project_name, doc_type);
    let db_path = project_root.join(&db_filename);

    println!("{}", format!("Creating {}...", db_filename).cyan());

    let conn = docs_db::create_docs_db(&db_path, &project_name, doc_type)?;

    // Create basic skeleton sections
    let sections = vec![
        ("1", "Overview", 1, desc.clone()),
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
            &conn, section_id, title, None, *level, sort_order, content, true, None,
        )?;
    }

    crate::schema_docs::set_meta(&conn, "generated_from", "skeleton")?;

    println!(
        "{} Created {} with {} sections",
        "✓".green(),
        db_filename,
        sections.len()
    );

    Ok(())
}

/// Generate documentation from codebase analysis
fn cmd_init_generate(project_root: &std::path::Path) -> Result<()> {
    use dialoguer::{Confirm, Input, Select};

    println!("\n{}", "Analyzing codebase...".cyan());

    // Analyze the project
    let structure = crate::source_analyzer::analyze_project(project_root)?;

    println!(
        "\n{} Detected {} project",
        "✓".green(),
        structure.language.as_str()
    );
    println!("  {} source files", structure.file_count);
    println!("  {} lines of code", structure.total_lines);
    println!("  {} items detected", structure.modules.len());

    if structure.modules.is_empty() {
        println!(
            "\n{} No analyzable items found in the codebase.",
            "!".yellow()
        );
        println!("Consider using 'Import' or 'New Project' instead.");
        return Ok(());
    }

    // Show summary of what was found
    let mut structs = 0;
    let mut enums = 0;
    let mut traits = 0;
    let mut functions = 0;
    let mut modules = 0;

    for item in &structure.modules {
        match item.kind {
            crate::source_analyzer::ItemKind::Struct => structs += 1,
            crate::source_analyzer::ItemKind::Enum => enums += 1,
            crate::source_analyzer::ItemKind::Trait => traits += 1,
            crate::source_analyzer::ItemKind::Function => functions += 1,
            crate::source_analyzer::ItemKind::Module => modules += 1,
            _ => {}
        }
    }

    println!("\n{}", "Found:".green());
    if modules > 0 {
        println!("  {} modules", modules);
    }
    if structs > 0 {
        println!("  {} structs", structs);
    }
    if enums > 0 {
        println!("  {} enums", enums);
    }
    if traits > 0 {
        println!("  {} traits", traits);
    }
    if functions > 0 {
        println!("  {} functions", functions);
    }

    // Confirm generation
    if !Confirm::new()
        .with_prompt("Generate documentation from this analysis?")
        .default(true)
        .interact()?
    {
        return Ok(());
    }

    // Ask for doc type
    let type_options = &[
        "Architecture - System/application design",
        "Framework    - Library/framework documentation",
        "API          - API reference",
    ];

    let type_selection = Select::new()
        .with_prompt("What type of documentation?")
        .items(type_options)
        .default(0)
        .interact()?;

    let doc_type = match type_selection {
        0 => DocType::Architecture,
        1 => DocType::Framework,
        2 => DocType::Api,
        _ => DocType::Architecture,
    };

    // Get project name
    let project_name: String = Input::new()
        .with_prompt("Project name")
        .default(structure.name.clone())
        .interact_text()?;

    // Create database
    let db_filename = crate::schema_docs::docs_db_filename(&project_name, doc_type);
    let db_path = project_root.join(&db_filename);

    println!("\n{}", format!("Creating {}...", db_filename).cyan());

    let conn = docs_db::create_docs_db(&db_path, &project_name, doc_type)?;

    // Generate sections
    let sections = crate::source_analyzer::generate_sections(&structure);

    // Insert sections
    for section in &sections {
        docs_db::insert_section(
            &conn,
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

    // Record generation info
    crate::schema_docs::set_meta(&conn, "generated_from", "source_analysis")?;
    crate::schema_docs::set_meta(&conn, "language", structure.language.as_str())?;
    crate::schema_docs::set_meta(
        &conn,
        "generated_at",
        &chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.6f")
            .to_string(),
    )?;

    println!(
        "\n{} Created {} with {} sections",
        "✓".green(),
        db_filename,
        sections.len()
    );
    println!("\nRun 'proj docs show' to view the table of contents.");
    println!("Run 'proj docs refresh' to update when source changes.");

    Ok(())
}

/// Create new project documentation from description (interactive)
fn cmd_init_new(project_root: &std::path::Path) -> Result<()> {
    use dialoguer::{Input, MultiSelect, Select};

    println!("\n{}", "New Project Documentation Wizard".bold());
    println!("{}\n", "─".repeat(40));
    println!("Answer a few questions to generate a documentation skeleton.\n");

    // Project name
    let default_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let project_name: String = Input::new()
        .with_prompt("Project name")
        .default(default_name.to_string())
        .interact_text()?;

    // Project description
    let description: String = Input::new()
        .with_prompt("Brief project description (1-2 sentences)")
        .interact_text()?;

    // Project type
    let type_options = &[
        "CLI Tool       - Command-line application",
        "Library        - Reusable code library/crate",
        "Web App        - Web application or API",
        "Desktop App    - Desktop GUI application",
        "Mobile App     - Mobile application",
        "Service        - Background service or daemon",
        "Plugin/Extension - Extension for another system",
    ];

    let type_selection = Select::new()
        .with_prompt("What type of project is this?")
        .items(type_options)
        .default(0)
        .interact()?;

    let project_type = match type_selection {
        0 => "CLI Tool",
        1 => "Library",
        2 => "Web Application",
        3 => "Desktop Application",
        4 => "Mobile Application",
        5 => "Service",
        6 => "Plugin/Extension",
        _ => "Application",
    };

    // Architecture patterns
    let pattern_options = &[
        "Modular         - Organized into modules/components",
        "Layered         - Presentation, business, data layers",
        "MVC/MVVM        - Model-View-Controller or similar",
        "Microservices   - Distributed services",
        "Plugin-based    - Core with plugin system",
        "Event-driven    - Event/message based",
        "None specific   - No particular pattern",
    ];

    let pattern_selections = MultiSelect::new()
        .with_prompt("Architecture patterns used (space to select, enter to confirm)")
        .items(pattern_options)
        .interact()?;

    let patterns: Vec<&str> = pattern_selections
        .iter()
        .map(|&i| match i {
            0 => "Modular",
            1 => "Layered",
            2 => "MVC/MVVM",
            3 => "Microservices",
            4 => "Plugin-based",
            5 => "Event-driven",
            _ => "Other",
        })
        .collect();

    // Key features
    let feature_options = &[
        "Database/Storage    - Persistent data storage",
        "Authentication      - User login/auth",
        "API/Network         - Network communication",
        "File I/O            - File processing",
        "Concurrency         - Parallel processing",
        "Configuration       - Config files/settings",
        "Logging             - Structured logging",
        "Testing             - Unit/integration tests",
    ];

    let feature_selections = MultiSelect::new()
        .with_prompt("Key features/components (space to select)")
        .items(feature_options)
        .interact()?;

    let features: Vec<&str> = feature_selections
        .iter()
        .map(|&i| match i {
            0 => "Database/Storage",
            1 => "Authentication",
            2 => "API/Network",
            3 => "File I/O",
            4 => "Concurrency",
            5 => "Configuration",
            6 => "Logging",
            7 => "Testing",
            _ => "Other",
        })
        .collect();

    // Doc type
    let doc_options = &[
        "Architecture - System design documentation",
        "Framework    - Library/framework reference",
        "Guide        - User/developer guide",
    ];

    let doc_selection = Select::new()
        .with_prompt("Documentation type")
        .items(doc_options)
        .default(0)
        .interact()?;

    let doc_type = match doc_selection {
        0 => DocType::Architecture,
        1 => DocType::Framework,
        2 => DocType::Guide,
        _ => DocType::Architecture,
    };

    // Generate the documentation
    println!("\n{}", "Generating documentation skeleton...".cyan());

    let db_filename = crate::schema_docs::docs_db_filename(&project_name, doc_type);
    let db_path = project_root.join(&db_filename);

    let conn = docs_db::create_docs_db(&db_path, &project_name, doc_type)?;

    // Build sections based on answers
    let mut sections = Vec::new();

    // 1. Overview
    let overview_content = format!(
        "{}\n\n**Type:** {}\n\n**Architecture:** {}\n",
        description,
        project_type,
        if patterns.is_empty() {
            "Not specified".to_string()
        } else {
            patterns.join(", ")
        }
    );
    sections.push(("1", "Overview", 1, overview_content));

    // 2. Architecture (if applicable)
    if !patterns.is_empty() || doc_type == DocType::Architecture {
        let arch_content = format!(
            "This section describes the architectural design of {}.\n\n## Patterns\n\n{}\n",
            project_name,
            patterns
                .iter()
                .map(|p| format!("- **{}**", p))
                .collect::<Vec<_>>()
                .join("\n")
        );
        sections.push(("2", "Architecture", 1, arch_content));
    }

    // 3. Components/Modules
    let components_content = if project_type == "CLI Tool" {
        "## Command Structure\n\nDescribe the main commands and subcommands.\n\n## Core Modules\n\nList the main modules and their responsibilities.\n".to_string()
    } else if project_type == "Library" {
        "## Public API\n\nDescribe the main types and functions exposed by the library.\n\n## Internal Modules\n\nDescribe internal organization.\n".to_string()
    } else {
        "## Main Components\n\nList and describe the main components of the system.\n".to_string()
    };
    sections.push(("3", "Components", 1, components_content));

    // 4. Data Model (if database feature selected)
    if features.contains(&"Database/Storage") {
        sections.push((
            "4",
            "Data Model",
            1,
            "## Entities\n\nDescribe the main data entities.\n\n## Schema\n\nDescribe the database schema or storage format.\n".to_string(),
        ));
    }

    // 5. API (if network feature selected)
    if features.contains(&"API/Network") {
        sections.push((
            "5",
            "API Reference",
            1,
            "## Endpoints\n\nList API endpoints and their functions.\n\n## Authentication\n\nDescribe API authentication.\n".to_string(),
        ));
    }

    // 6. Configuration (if config feature selected)
    if features.contains(&"Configuration") {
        sections.push((
            "6",
            "Configuration",
            1,
            "## Configuration Files\n\nDescribe configuration file locations and formats.\n\n## Settings\n\nList available settings and their defaults.\n".to_string(),
        ));
    }

    // 7. Development
    let dev_content = if features.contains(&"Testing") {
        "## Building\n\nDescribe how to build the project.\n\n## Testing\n\nDescribe how to run tests.\n\n## Contributing\n\nGuidelines for contributors.\n".to_string()
    } else {
        "## Building\n\nDescribe how to build the project.\n\n## Contributing\n\nGuidelines for contributors.\n".to_string()
    };
    sections.push(("7", "Development", 1, dev_content));

    // Insert all sections
    let mut sort_order = 0;
    for (section_id, title, level, content) in &sections {
        sort_order += 1;
        docs_db::insert_section(
            &conn, section_id, title, None, *level, sort_order, content, true, // generated
            None,
        )?;
    }

    // Store wizard answers as metadata
    crate::schema_docs::set_meta(&conn, "project_type", project_type)?;
    crate::schema_docs::set_meta(&conn, "generated_from", "wizard")?;
    if !patterns.is_empty() {
        crate::schema_docs::set_meta(&conn, "patterns", &patterns.join(", "))?;
    }
    if !features.is_empty() {
        crate::schema_docs::set_meta(&conn, "features", &features.join(", "))?;
    }

    println!(
        "\n{} Created {} with {} sections",
        "✓".green(),
        db_filename,
        sections.len()
    );
    println!("\nThis is a skeleton - edit sections with your specific details.");
    println!("Run 'proj docs show' to view the table of contents.");
    println!("Run 'proj docs export --format md' to export and edit.");

    Ok(())
}

/// Show documentation database status
fn cmd_status() -> Result<()> {
    let project_root = get_project_root()?;

    let db_path = match docs_db::find_docs_db(&project_root) {
        Some(path) => path,
        None => {
            println!("{} No documentation database found.", "!".yellow());
            println!("Run 'proj docs init' to create one.");
            return Ok(());
        }
    };

    let conn = docs_db::open_docs_db(&db_path)?;
    let mut info = docs_db::get_docs_info(&conn)?;
    info.path = db_path.clone();

    println!("\n{}", "Documentation Database".bold());
    println!("{}", "─".repeat(40));
    println!(
        "  File: {}",
        db_path.file_name().unwrap_or_default().to_string_lossy()
    );
    println!("  Project: {}", info.project_name);
    println!("  Type: {}", info.doc_type);
    if let Some(created) = &info.created_at {
        println!("  Created: {}", created);
    }
    if let Some(version) = &info.version {
        println!("  Version: {}", version);
    }
    if let Some(imported) = &info.imported_from {
        println!("  Imported from: {}", imported);
    }
    println!("  Sections: {}", info.section_count);
    println!("  Terms: {}", info.term_count);

    // Show generation info if applicable
    if let Ok(Some(generated_from)) = crate::schema_docs::get_meta(&conn, "generated_from") {
        println!("  Source: {}", generated_from);

        // Show section breakdown
        if let Ok((generated, manual)) = docs_db::get_section_counts(&conn) {
            if generated > 0 || manual > 0 {
                println!("  Generated: {}, Manual: {}", generated, manual);
            }
        }

        // Check staleness for source-generated docs
        if generated_from == "source_analysis" {
            if let Ok(Some(refreshed)) = crate::schema_docs::get_meta(&conn, "refreshed_at") {
                println!("  Last refresh: {}", refreshed);
            } else if let Ok(Some(generated)) = crate::schema_docs::get_meta(&conn, "generated_at")
            {
                println!("  Generated: {}", generated);
            }

            // Check if source files have changed since generation
            let last_update = crate::schema_docs::get_meta(&conn, "refreshed_at")
                .ok()
                .flatten()
                .or_else(|| {
                    crate::schema_docs::get_meta(&conn, "generated_at")
                        .ok()
                        .flatten()
                });

            if let Some(last_update) = last_update {
                // Check if any source files are newer than last update
                if let Ok(stale_count) = check_staleness(&project_root, &last_update) {
                    if stale_count > 0 {
                        println!(
                            "\n  {} {} source files changed since last update",
                            "!".yellow(),
                            stale_count
                        );
                        println!("  Run 'proj docs refresh' to update.");
                    }
                }
            }
        }
    }

    Ok(())
}

/// Check how many source files have changed since a given timestamp
fn check_staleness(project_root: &std::path::Path, since: &str) -> Result<usize> {
    use chrono::{DateTime, NaiveDateTime, Utc};

    // Try parsing as DateTime<Utc> first, then as NaiveDateTime
    let since_time: DateTime<Utc> = since
        .parse::<DateTime<Utc>>()
        .or_else(|_| {
            // Try parsing without timezone (our format doesn't have Z suffix)
            NaiveDateTime::parse_from_str(since, "%Y-%m-%dT%H:%M:%S%.f").map(|ndt| ndt.and_utc())
        })
        .unwrap_or_else(|_| Utc::now());

    let since_secs = since_time.timestamp();
    let mut stale_count = 0;

    // Check src directory for Rust files
    let src_dir = project_root.join("src");
    if src_dir.is_dir() {
        fn check_dir(dir: &std::path::Path, since_secs: i64, count: &mut usize) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        check_dir(&path, since_secs, count);
                    } else if path.extension().map_or(false, |e| e == "rs") {
                        if let Ok(meta) = std::fs::metadata(&path) {
                            if let Ok(modified) = meta.modified() {
                                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH)
                                {
                                    if duration.as_secs() as i64 > since_secs {
                                        *count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        check_dir(&src_dir, since_secs, &mut stale_count);
    }

    Ok(stale_count)
}

/// Refresh documentation from source
fn cmd_refresh(force: bool) -> Result<()> {
    let project_root = get_project_root()?;

    let db_path = match docs_db::find_docs_db(&project_root) {
        Some(path) => path,
        None => bail!("No documentation database found. Run 'proj docs init' first."),
    };

    let conn = docs_db::open_docs_db(&db_path)?;

    // Check if this was generated from source analysis
    let generated_from = crate::schema_docs::get_meta(&conn, "generated_from")?;
    if generated_from.as_deref() != Some("source_analysis") {
        println!(
            "{} This database was not generated from source analysis.",
            "!".yellow()
        );
        println!("Refresh only works for databases created with 'proj docs init --generate'.");
        return Ok(());
    }

    // Get current section counts
    let (_generated_count, manual_count) = docs_db::get_section_counts(&conn)?;

    if manual_count > 0 && !force {
        println!(
            "{} Found {} manually edited sections that will be preserved.",
            "ℹ".blue(),
            manual_count
        );
        println!("Use --force to regenerate all sections.");
    }

    println!("{}", "Re-analyzing codebase...".cyan());

    // Re-analyze the project
    let structure = crate::source_analyzer::analyze_project(&project_root)?;

    println!(
        "{} Detected {} changes ({} files, {} lines)",
        "✓".green(),
        structure.language.as_str(),
        structure.file_count,
        structure.total_lines
    );

    // Delete existing generated sections
    let deleted = if force {
        // Delete ALL sections when force is used
        conn.execute("DELETE FROM sections", [])?
    } else {
        // Only delete generated sections
        docs_db::delete_generated_sections(&conn)?
    };

    println!("  Removed {} old sections", deleted);

    // Generate new sections
    let sections = crate::source_analyzer::generate_sections(&structure);

    // Insert new sections
    for section in &sections {
        docs_db::insert_section(
            &conn,
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

    // Update timestamp
    crate::schema_docs::set_meta(
        &conn,
        "refreshed_at",
        &chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.6f")
            .to_string(),
    )?;

    println!("{} Refreshed with {} sections", "✓".green(), sections.len());

    Ok(())
}

/// Search documentation
fn cmd_search(query: &str) -> Result<()> {
    let project_root = get_project_root()?;

    let db_path = match docs_db::find_docs_db(&project_root) {
        Some(path) => path,
        None => bail!("No documentation database found. Run 'proj docs init' first."),
    };

    let conn = docs_db::open_docs_db(&db_path)?;
    let results = docs_db::search_sections(&conn, query)?;

    if results.is_empty() {
        println!("No results found for '{}'", query);
        return Ok(());
    }

    println!("\n{} results for '{}':\n", results.len(), query);

    for section in results {
        println!(
            "{} {} {}",
            section.section_id.cyan(),
            section.title.bold(),
            if section.generated { "(generated)" } else { "" }
        );

        // Show snippet of content
        let snippet: String = section
            .content
            .chars()
            .take(150)
            .collect::<String>()
            .replace('\n', " ");

        if !snippet.is_empty() {
            println!("  {}...", snippet);
        }
        println!();
    }

    Ok(())
}

/// Export documentation
fn cmd_export(format: &str, output: Option<String>) -> Result<()> {
    let project_root = get_project_root()?;

    let db_path = match docs_db::find_docs_db(&project_root) {
        Some(path) => path,
        None => bail!("No documentation database found. Run 'proj docs init' first."),
    };

    let conn = docs_db::open_docs_db(&db_path)?;
    let sections = docs_db::get_all_sections(&conn)?;

    let content = match format {
        "md" | "markdown" => export_markdown(&sections),
        "html" => {
            println!("{} HTML export not yet implemented.", "!".yellow());
            return Ok(());
        }
        _ => bail!("Unknown format: {}. Use 'md' or 'html'.", format),
    };

    match output {
        Some(path) => {
            std::fs::write(&path, content)?;
            println!("{} Exported to {}", "✓".green(), path);
        }
        None => {
            println!("{}", content);
        }
    }

    Ok(())
}

/// Export sections to markdown
fn export_markdown(sections: &[docs_db::Section]) -> String {
    let mut output = String::new();

    for section in sections {
        // Add heading
        let hashes = "#".repeat(section.level as usize);
        output.push_str(&format!("{} {}\n\n", hashes, section.title));

        // Add content
        if !section.content.is_empty() {
            output.push_str(&section.content);
            output.push_str("\n\n");
        }
    }

    output
}

/// Show a section
fn cmd_show(section_id: Option<String>) -> Result<()> {
    let project_root = get_project_root()?;

    let db_path = match docs_db::find_docs_db(&project_root) {
        Some(path) => path,
        None => bail!("No documentation database found. Run 'proj docs init' first."),
    };

    let conn = docs_db::open_docs_db(&db_path)?;

    match section_id {
        Some(id) => {
            // Show specific section
            let sections = docs_db::get_all_sections(&conn)?;
            let section = sections
                .iter()
                .find(|s| s.section_id == id)
                .ok_or_else(|| anyhow::anyhow!("Section '{}' not found", id))?;

            let hashes = "#".repeat(section.level as usize);
            println!("{} {}", hashes, section.title.bold());
            if !section.content.is_empty() {
                println!("\n{}", section.content);
            }
        }
        None => {
            // Show table of contents
            let sections = docs_db::get_all_sections(&conn)?;

            println!("\n{}", "Table of Contents".bold());
            println!("{}\n", "─".repeat(40));

            for section in sections {
                let indent = "  ".repeat((section.level - 1) as usize);
                println!("{}{} {}", indent, section.section_id.cyan(), section.title);
            }
        }
    }

    Ok(())
}

/// Add a term to the glossary
fn cmd_term_add(term: &str, definition: &str, category: Option<&str>) -> Result<()> {
    let project_root = get_project_root()?;

    let db_path = match docs_db::find_docs_db(&project_root) {
        Some(path) => path,
        None => bail!("No documentation database found. Run 'proj docs init' first."),
    };

    let conn = docs_db::open_docs_db(&db_path)?;
    docs_db::insert_term(&conn, term, Some(definition), category, &[])?;

    println!("{} Added term: {}", "✓".green(), term);
    Ok(())
}

/// List all terms
fn cmd_term_list() -> Result<()> {
    let project_root = get_project_root()?;

    let db_path = match docs_db::find_docs_db(&project_root) {
        Some(path) => path,
        None => bail!("No documentation database found. Run 'proj docs init' first."),
    };

    let conn = docs_db::open_docs_db(&db_path)?;
    let terms = docs_db::get_all_terms(&conn)?;

    if terms.is_empty() {
        println!("No terminology entries.");
        return Ok(());
    }

    println!("\n{}", "Terminology".bold());
    println!("{}\n", "─".repeat(40));

    for term in terms {
        print!("{}", term.canonical.cyan());
        if let Some(cat) = &term.category {
            print!(" [{}]", cat);
        }
        println!();
        if let Some(def) = &term.definition {
            println!("  {}", def);
        }
        println!();
    }

    Ok(())
}

/// Search terms
fn cmd_term_search(query: &str) -> Result<()> {
    let project_root = get_project_root()?;

    let db_path = match docs_db::find_docs_db(&project_root) {
        Some(path) => path,
        None => bail!("No documentation database found. Run 'proj docs init' first."),
    };

    let conn = docs_db::open_docs_db(&db_path)?;

    // Use FTS5 search
    let mut stmt = conn.prepare(
        r#"SELECT t.canonical, t.definition, t.category
           FROM terminology t
           JOIN terminology_fts fts ON t.id = fts.rowid
           WHERE terminology_fts MATCH ?1"#,
    )?;

    let results: Vec<(String, Option<String>, Option<String>)> = stmt
        .query_map([query], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .filter_map(|r| r.ok())
        .collect();

    if results.is_empty() {
        println!("No terms found for '{}'", query);
        return Ok(());
    }

    println!("\n{} terms matching '{}':\n", results.len(), query);

    for (canonical, definition, category) in results {
        print!("{}", canonical.cyan());
        if let Some(cat) = category {
            print!(" [{}]", cat);
        }
        println!();
        if let Some(def) = definition {
            println!("  {}", def);
        }
        println!();
    }

    Ok(())
}
