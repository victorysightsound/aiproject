// Documentation database commands

use anyhow::{bail, Result};
use colored::Colorize;

use crate::cli::{DocsCommands, DocsSubcommand, DocsTermSubcommand};
use crate::docs_db::{self, DocsDbInfo};
use crate::paths::get_project_root;
use crate::schema_docs::DocType;

pub fn run(cmd: DocsCommands) -> Result<()> {
    match cmd.command {
        DocsSubcommand::Init => cmd_init(),
        DocsSubcommand::Status => cmd_status(),
        DocsSubcommand::Refresh { force } => cmd_refresh(force),
        DocsSubcommand::Search { query } => cmd_search(&query),
        DocsSubcommand::Export { format, output } => cmd_export(&format, output),
        DocsSubcommand::Show { section } => cmd_show(section),
        DocsSubcommand::Term(term_cmd) => match term_cmd.command {
            DocsTermSubcommand::Add { term, def, category } => {
                cmd_term_add(&term, &def, category.as_deref())
            }
            DocsTermSubcommand::List => cmd_term_list(),
            DocsTermSubcommand::Search { query } => cmd_term_search(&query),
        },
    }
}

/// Initialize documentation database - interactive wizard
fn cmd_init() -> Result<()> {
    use dialoguer::{Confirm, Input, Select};

    let project_root = get_project_root()?;

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

/// Import existing markdown files
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
        &chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.6f").to_string(),
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

            let title = line
                .trim_start_matches('#')
                .trim()
                .to_string();

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

/// Generate documentation from codebase analysis
fn cmd_init_generate(_project_root: &std::path::Path) -> Result<()> {
    println!("\n{} Generate mode is not yet implemented.", "!".yellow());
    println!("This will analyze your codebase and generate documentation.");
    println!("Coming in Phase 2.");
    Ok(())
}

/// Create new project documentation from description
fn cmd_init_new(_project_root: &std::path::Path) -> Result<()> {
    println!("\n{} New Project mode is not yet implemented.", "!".yellow());
    println!("This will ask questions and generate a documentation skeleton.");
    println!("Coming in Phase 3.");
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
    if let Some(imported) = &info.imported_from {
        println!("  Imported from: {}", imported);
    }
    println!("  Sections: {}", info.section_count);
    println!("  Terms: {}", info.term_count);

    Ok(())
}

/// Refresh documentation from source
fn cmd_refresh(_force: bool) -> Result<()> {
    println!("{} Refresh is not yet implemented.", "!".yellow());
    println!("Coming in Phase 2.");
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
                println!(
                    "{}{} {}",
                    indent,
                    section.section_id.cyan(),
                    section.title
                );
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
