// Source code analyzer for generating documentation
// Supports Rust, with extensible design for other languages

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Detected item from source code
#[derive(Debug, Clone)]
pub struct SourceItem {
    pub kind: ItemKind,
    pub name: String,
    pub visibility: Visibility,
    pub doc_comment: Option<String>,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub children: Vec<SourceItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Module,
    Struct,
    Enum,
    Trait,
    Impl,
    Function,
    Constant,
    Type,
}

impl ItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemKind::Module => "Module",
            ItemKind::Struct => "Struct",
            ItemKind::Enum => "Enum",
            ItemKind::Trait => "Trait",
            ItemKind::Impl => "Impl",
            ItemKind::Function => "Function",
            ItemKind::Constant => "Constant",
            ItemKind::Type => "Type",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    Public,
    PublicCrate,
    Private,
}

/// Project structure detected from source analysis
#[derive(Debug)]
pub struct ProjectStructure {
    pub name: String,
    pub language: Language,
    pub modules: Vec<SourceItem>,
    pub entry_points: Vec<PathBuf>,
    pub file_count: usize,
    pub total_lines: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    Go,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "ts" | "tsx" => Language::TypeScript,
            "go" => Language::Go,
            _ => Language::Unknown,
        }
    }

    pub fn extensions(&self) -> &[&str] {
        match self {
            Language::Rust => &["rs"],
            Language::Python => &["py"],
            Language::TypeScript => &["ts", "tsx"],
            Language::Go => &["go"],
            Language::Unknown => &[],
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::TypeScript => "TypeScript",
            Language::Go => "Go",
            Language::Unknown => "Unknown",
        }
    }
}

/// Analyze a project directory
pub fn analyze_project(project_root: &Path) -> Result<ProjectStructure> {
    // Detect primary language
    let language = detect_language(project_root)?;

    // Get project name
    let name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project")
        .to_string();

    // Find source files
    let source_files = find_source_files(project_root, &language)?;

    // Parse each file
    let mut modules = Vec::new();
    let mut entry_points = Vec::new();
    let mut total_lines = 0;

    for file_path in &source_files {
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {:?}", file_path))?;

        total_lines += content.lines().count();

        // Check if this is an entry point
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "main.rs" || file_name == "lib.rs" || file_name == "mod.rs" {
            entry_points.push(file_path.clone());
        }

        // Parse the file
        let items = match language {
            Language::Rust => parse_rust_file(&content, file_path)?,
            Language::Python => parse_python_file(&content, file_path)?,
            Language::TypeScript => parse_typescript_file(&content, file_path)?,
            Language::Go => parse_go_file(&content, file_path)?,
            Language::Unknown => Vec::new(),
        };

        modules.extend(items);
    }

    Ok(ProjectStructure {
        name,
        language,
        modules,
        entry_points,
        file_count: source_files.len(),
        total_lines,
    })
}

/// Detect the primary language of a project
fn detect_language(project_root: &Path) -> Result<Language> {
    // Check for language-specific files
    if project_root.join("Cargo.toml").exists() {
        return Ok(Language::Rust);
    }
    if project_root.join("package.json").exists() {
        // Could be JS or TS - check for tsconfig
        if project_root.join("tsconfig.json").exists() {
            return Ok(Language::TypeScript);
        }
    }
    if project_root.join("go.mod").exists() {
        return Ok(Language::Go);
    }
    if project_root.join("setup.py").exists()
        || project_root.join("pyproject.toml").exists()
        || project_root.join("requirements.txt").exists()
    {
        return Ok(Language::Python);
    }

    // Count files by extension
    let mut counts: HashMap<Language, usize> = HashMap::new();

    fn count_files(dir: &Path, counts: &mut HashMap<Language, usize>, depth: usize) {
        if depth > 5 {
            return; // Don't go too deep
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // Skip common non-source directories
                    if !["target", "node_modules", ".git", "vendor", "__pycache__", "build", "dist"]
                        .contains(&name)
                    {
                        count_files(&path, counts, depth + 1);
                    }
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let lang = Language::from_extension(ext);
                    if lang != Language::Unknown {
                        *counts.entry(lang).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    count_files(project_root, &mut counts, 0);

    // Return the most common language
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(lang, _)| lang)
        .unwrap_or(Language::Unknown)
        .pipe(Ok)
}

trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}

impl<T> Pipe for T {}

/// Find all source files for a language
fn find_source_files(project_root: &Path, language: &Language) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let extensions = language.extensions();

    fn walk_dir(dir: &Path, extensions: &[&str], files: &mut Vec<PathBuf>, depth: usize) {
        if depth > 10 {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // Skip non-source directories
                    if !["target", "node_modules", ".git", "vendor", "__pycache__", "build", "dist", ".tracking"]
                        .contains(&name)
                    {
                        walk_dir(&path, extensions, files, depth + 1);
                    }
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        files.push(path);
                    }
                }
            }
        }
    }

    // For Rust, start from src/ if it exists
    let start_dir = if *language == Language::Rust && project_root.join("src").is_dir() {
        project_root.join("src")
    } else {
        project_root.to_path_buf()
    };

    walk_dir(&start_dir, extensions, &mut files, 0);

    // Sort for consistent ordering
    files.sort();

    Ok(files)
}

/// Parse a Rust source file
fn parse_rust_file(content: &str, file_path: &Path) -> Result<Vec<SourceItem>> {
    let mut items = Vec::new();
    let mut current_doc = String::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Collect doc comments
        if trimmed.starts_with("///") || trimmed.starts_with("//!") {
            let doc_line = trimmed
                .trim_start_matches("///")
                .trim_start_matches("//!")
                .trim();
            if !current_doc.is_empty() {
                current_doc.push('\n');
            }
            current_doc.push_str(doc_line);
            continue;
        }

        // Skip regular comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Parse item declarations
        let visibility = if trimmed.starts_with("pub(crate)") {
            Visibility::PublicCrate
        } else if trimmed.starts_with("pub") {
            Visibility::Public
        } else {
            Visibility::Private
        };

        // Remove visibility prefix for parsing
        let without_vis = trimmed
            .trim_start_matches("pub(crate)")
            .trim_start_matches("pub")
            .trim();

        // Detect item type
        if let Some(item) = parse_rust_item(without_vis, visibility, &current_doc, file_path, line_num + 1) {
            items.push(item);
        }

        // Clear doc comment if this wasn't a blank line
        if !trimmed.is_empty() {
            current_doc.clear();
        }
    }

    Ok(items)
}

/// Parse a single Rust item declaration
fn parse_rust_item(
    line: &str,
    visibility: Visibility,
    doc_comment: &str,
    file_path: &Path,
    line_number: usize,
) -> Option<SourceItem> {
    // Parse different item types
    if line.starts_with("mod ") {
        let name = extract_name(line, "mod ");
        return Some(SourceItem {
            kind: ItemKind::Module,
            name,
            visibility,
            doc_comment: non_empty_doc(doc_comment),
            file_path: file_path.to_path_buf(),
            line_number,
            children: Vec::new(),
        });
    }

    if line.starts_with("struct ") {
        let name = extract_name(line, "struct ");
        return Some(SourceItem {
            kind: ItemKind::Struct,
            name,
            visibility,
            doc_comment: non_empty_doc(doc_comment),
            file_path: file_path.to_path_buf(),
            line_number,
            children: Vec::new(),
        });
    }

    if line.starts_with("enum ") {
        let name = extract_name(line, "enum ");
        return Some(SourceItem {
            kind: ItemKind::Enum,
            name,
            visibility,
            doc_comment: non_empty_doc(doc_comment),
            file_path: file_path.to_path_buf(),
            line_number,
            children: Vec::new(),
        });
    }

    if line.starts_with("trait ") {
        let name = extract_name(line, "trait ");
        return Some(SourceItem {
            kind: ItemKind::Trait,
            name,
            visibility,
            doc_comment: non_empty_doc(doc_comment),
            file_path: file_path.to_path_buf(),
            line_number,
            children: Vec::new(),
        });
    }

    if line.starts_with("impl ") || line.starts_with("impl<") {
        // Extract impl name (struct being implemented or trait for struct)
        let name = extract_impl_name(line);
        return Some(SourceItem {
            kind: ItemKind::Impl,
            name,
            visibility,
            doc_comment: non_empty_doc(doc_comment),
            file_path: file_path.to_path_buf(),
            line_number,
            children: Vec::new(),
        });
    }

    if line.starts_with("fn ") || line.starts_with("async fn ") {
        let prefix = if line.starts_with("async ") { "async fn " } else { "fn " };
        let name = extract_name(line, prefix);
        return Some(SourceItem {
            kind: ItemKind::Function,
            name,
            visibility,
            doc_comment: non_empty_doc(doc_comment),
            file_path: file_path.to_path_buf(),
            line_number,
            children: Vec::new(),
        });
    }

    if line.starts_with("const ") {
        let name = extract_name(line, "const ");
        return Some(SourceItem {
            kind: ItemKind::Constant,
            name,
            visibility,
            doc_comment: non_empty_doc(doc_comment),
            file_path: file_path.to_path_buf(),
            line_number,
            children: Vec::new(),
        });
    }

    if line.starts_with("type ") {
        let name = extract_name(line, "type ");
        return Some(SourceItem {
            kind: ItemKind::Type,
            name,
            visibility,
            doc_comment: non_empty_doc(doc_comment),
            file_path: file_path.to_path_buf(),
            line_number,
            children: Vec::new(),
        });
    }

    None
}

/// Extract name from a declaration line
fn extract_name(line: &str, prefix: &str) -> String {
    let after_prefix = line.strip_prefix(prefix).unwrap_or(line);
    after_prefix
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()
        .unwrap_or("")
        .to_string()
}

/// Extract impl name (handles generic impls and trait impls)
fn extract_impl_name(line: &str) -> String {
    // Handle patterns like:
    // impl Foo
    // impl<T> Foo<T>
    // impl Trait for Foo
    // impl<T> Trait for Foo<T>

    let line = line.trim_start_matches("impl").trim();

    // Skip generic parameters
    let line = if line.starts_with('<') {
        // Find matching >
        let mut depth = 0;
        let mut end = 0;
        for (i, c) in line.chars().enumerate() {
            match c {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        line[end..].trim()
    } else {
        line
    };

    // Check for "Trait for Type" pattern
    if let Some(idx) = line.find(" for ") {
        let trait_name = line[..idx].split('<').next().unwrap_or("").trim();
        let type_name = line[idx + 5..].split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("").trim();
        return format!("{} for {}", trait_name, type_name);
    }

    // Just the type name
    line.split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()
        .unwrap_or("")
        .to_string()
}

fn non_empty_doc(doc: &str) -> Option<String> {
    if doc.is_empty() {
        None
    } else {
        Some(doc.to_string())
    }
}

/// Parse a Python source file
fn parse_python_file(content: &str, file_path: &Path) -> Result<Vec<SourceItem>> {
    let mut items = Vec::new();
    let mut current_doc = String::new();
    let mut in_docstring = false;
    let mut docstring_delimiter = "";
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Handle docstrings (""" or ''')
        if !in_docstring {
            if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                docstring_delimiter = if trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
                // Check if docstring ends on same line
                let after_start = &trimmed[3..];
                if after_start.contains(docstring_delimiter) {
                    // Single-line docstring
                    let end_idx = after_start.find(docstring_delimiter).unwrap();
                    current_doc = after_start[..end_idx].to_string();
                } else {
                    in_docstring = true;
                    current_doc = after_start.to_string();
                }
                continue;
            }
        } else {
            if trimmed.contains(docstring_delimiter) {
                let end_idx = trimmed.find(docstring_delimiter).unwrap();
                if !current_doc.is_empty() {
                    current_doc.push('\n');
                }
                current_doc.push_str(&trimmed[..end_idx]);
                in_docstring = false;
                continue;
            } else {
                if !current_doc.is_empty() {
                    current_doc.push('\n');
                }
                current_doc.push_str(trimmed);
                continue;
            }
        }

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Detect class definitions
        if trimmed.starts_with("class ") {
            let name = extract_python_name(trimmed, "class ");
            items.push(SourceItem {
                kind: ItemKind::Struct, // Use Struct for Python classes
                name,
                visibility: Visibility::Public,
                doc_comment: non_empty_doc(&current_doc),
                file_path: file_path.to_path_buf(),
                line_number: line_num + 1,
                children: Vec::new(),
            });
            current_doc.clear();
        }

        // Detect function definitions
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            let prefix = if trimmed.starts_with("async ") { "async def " } else { "def " };
            let name = extract_python_name(trimmed, prefix);
            let visibility = if name.starts_with('_') && !name.starts_with("__") {
                Visibility::Private
            } else {
                Visibility::Public
            };
            items.push(SourceItem {
                kind: ItemKind::Function,
                name,
                visibility,
                doc_comment: non_empty_doc(&current_doc),
                file_path: file_path.to_path_buf(),
                line_number: line_num + 1,
                children: Vec::new(),
            });
            current_doc.clear();
        }

        // Clear doc if we hit a non-def/class line
        if !trimmed.is_empty() && !trimmed.starts_with("def ") && !trimmed.starts_with("async def ") && !trimmed.starts_with("class ") && !trimmed.starts_with('@') {
            current_doc.clear();
        }
    }

    Ok(items)
}

/// Extract name from Python definition
fn extract_python_name(line: &str, prefix: &str) -> String {
    let after_prefix = line.strip_prefix(prefix).unwrap_or(line);
    after_prefix
        .split(|c: char| c == '(' || c == ':' || c.is_whitespace())
        .next()
        .unwrap_or("")
        .to_string()
}

/// Parse a TypeScript source file
fn parse_typescript_file(content: &str, file_path: &Path) -> Result<Vec<SourceItem>> {
    let mut items = Vec::new();
    let mut current_doc = String::new();
    let mut in_jsdoc = false;
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Handle JSDoc comments
        if trimmed.starts_with("/**") {
            in_jsdoc = true;
            let after_start = trimmed.strip_prefix("/**").unwrap_or("").trim();
            if after_start.ends_with("*/") {
                // Single-line JSDoc
                current_doc = after_start.strip_suffix("*/").unwrap_or(after_start).trim().to_string();
                in_jsdoc = false;
            } else {
                current_doc = after_start.to_string();
            }
            continue;
        }

        if in_jsdoc {
            if trimmed.contains("*/") {
                let before_end = trimmed.strip_suffix("*/").unwrap_or(trimmed);
                let content_part = before_end.trim_start_matches('*').trim();
                if !current_doc.is_empty() && !content_part.is_empty() {
                    current_doc.push('\n');
                }
                current_doc.push_str(content_part);
                in_jsdoc = false;
            } else {
                let content_part = trimmed.trim_start_matches('*').trim();
                if !current_doc.is_empty() && !content_part.is_empty() {
                    current_doc.push('\n');
                }
                current_doc.push_str(content_part);
            }
            continue;
        }

        // Skip regular comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Determine visibility
        let visibility = if trimmed.starts_with("export ") || trimmed.starts_with("public ") {
            Visibility::Public
        } else if trimmed.starts_with("private ") {
            Visibility::Private
        } else {
            Visibility::Private // Default to private in TypeScript
        };

        // Remove modifiers for parsing
        let without_modifiers = trimmed
            .trim_start_matches("export ")
            .trim_start_matches("default ")
            .trim_start_matches("public ")
            .trim_start_matches("private ")
            .trim_start_matches("protected ")
            .trim_start_matches("static ")
            .trim_start_matches("async ")
            .trim_start_matches("declare ");

        // Detect interfaces
        if without_modifiers.starts_with("interface ") {
            let name = extract_ts_name(without_modifiers, "interface ");
            items.push(SourceItem {
                kind: ItemKind::Trait, // Use Trait for interfaces
                name,
                visibility,
                doc_comment: non_empty_doc(&current_doc),
                file_path: file_path.to_path_buf(),
                line_number: line_num + 1,
                children: Vec::new(),
            });
            current_doc.clear();
        }

        // Detect classes
        if without_modifiers.starts_with("class ") {
            let name = extract_ts_name(without_modifiers, "class ");
            items.push(SourceItem {
                kind: ItemKind::Struct,
                name,
                visibility,
                doc_comment: non_empty_doc(&current_doc),
                file_path: file_path.to_path_buf(),
                line_number: line_num + 1,
                children: Vec::new(),
            });
            current_doc.clear();
        }

        // Detect type aliases
        if without_modifiers.starts_with("type ") {
            let name = extract_ts_name(without_modifiers, "type ");
            items.push(SourceItem {
                kind: ItemKind::Type,
                name,
                visibility,
                doc_comment: non_empty_doc(&current_doc),
                file_path: file_path.to_path_buf(),
                line_number: line_num + 1,
                children: Vec::new(),
            });
            current_doc.clear();
        }

        // Detect enums
        if without_modifiers.starts_with("enum ") {
            let name = extract_ts_name(without_modifiers, "enum ");
            items.push(SourceItem {
                kind: ItemKind::Enum,
                name,
                visibility,
                doc_comment: non_empty_doc(&current_doc),
                file_path: file_path.to_path_buf(),
                line_number: line_num + 1,
                children: Vec::new(),
            });
            current_doc.clear();
        }

        // Detect functions
        if without_modifiers.starts_with("function ") {
            let name = extract_ts_name(without_modifiers, "function ");
            items.push(SourceItem {
                kind: ItemKind::Function,
                name,
                visibility,
                doc_comment: non_empty_doc(&current_doc),
                file_path: file_path.to_path_buf(),
                line_number: line_num + 1,
                children: Vec::new(),
            });
            current_doc.clear();
        }

        // Detect const/let declarations (top-level functions as arrow functions)
        if (without_modifiers.starts_with("const ") || without_modifiers.starts_with("let "))
            && (without_modifiers.contains(" = (") || without_modifiers.contains(" = async ("))
        {
            let keyword = if without_modifiers.starts_with("const ") { "const " } else { "let " };
            let name = extract_ts_name(without_modifiers, keyword);
            items.push(SourceItem {
                kind: ItemKind::Function,
                name,
                visibility,
                doc_comment: non_empty_doc(&current_doc),
                file_path: file_path.to_path_buf(),
                line_number: line_num + 1,
                children: Vec::new(),
            });
            current_doc.clear();
        }

        // Clear doc on non-declaration lines
        if !trimmed.is_empty() && !trimmed.starts_with('@') {
            current_doc.clear();
        }
    }

    Ok(items)
}

/// Extract name from TypeScript definition
fn extract_ts_name(line: &str, prefix: &str) -> String {
    let after_prefix = line.strip_prefix(prefix).unwrap_or(line);
    after_prefix
        .split(|c: char| c == '<' || c == '(' || c == '{' || c == ':' || c == '=' || c.is_whitespace())
        .next()
        .unwrap_or("")
        .to_string()
}

/// Parse a Go source file
fn parse_go_file(content: &str, file_path: &Path) -> Result<Vec<SourceItem>> {
    let mut items = Vec::new();
    let mut current_doc = String::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Collect doc comments (// comments before declarations)
        if trimmed.starts_with("//") {
            let comment = trimmed.strip_prefix("//").unwrap_or("").trim();
            if !current_doc.is_empty() {
                current_doc.push('\n');
            }
            current_doc.push_str(comment);
            continue;
        }

        // Determine visibility (Go uses capitalization)
        fn is_exported(name: &str) -> bool {
            name.chars().next().map_or(false, |c| c.is_uppercase())
        }

        // Detect type declarations (struct, interface)
        if trimmed.starts_with("type ") {
            // type Name struct { or type Name interface {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[1].to_string();
                let kind = match parts[2] {
                    "struct" => ItemKind::Struct,
                    "interface" => ItemKind::Trait,
                    _ => ItemKind::Type,
                };
                let visibility = if is_exported(&name) {
                    Visibility::Public
                } else {
                    Visibility::Private
                };
                items.push(SourceItem {
                    kind,
                    name,
                    visibility,
                    doc_comment: non_empty_doc(&current_doc),
                    file_path: file_path.to_path_buf(),
                    line_number: line_num + 1,
                    children: Vec::new(),
                });
            }
            current_doc.clear();
        }

        // Detect function declarations
        if trimmed.starts_with("func ") {
            // func Name(...) or func (receiver) Name(...)
            let after_func = trimmed.strip_prefix("func ").unwrap_or("");

            // Check for method (has receiver)
            let name = if after_func.starts_with('(') {
                // Method: func (r *Receiver) Name(...)
                if let Some(close_paren) = after_func.find(')') {
                    let after_receiver = after_func[close_paren + 1..].trim();
                    extract_go_func_name(after_receiver)
                } else {
                    String::new()
                }
            } else {
                // Regular function
                extract_go_func_name(after_func)
            };

            if !name.is_empty() {
                let visibility = if is_exported(&name) {
                    Visibility::Public
                } else {
                    Visibility::Private
                };
                items.push(SourceItem {
                    kind: ItemKind::Function,
                    name,
                    visibility,
                    doc_comment: non_empty_doc(&current_doc),
                    file_path: file_path.to_path_buf(),
                    line_number: line_num + 1,
                    children: Vec::new(),
                });
            }
            current_doc.clear();
        }

        // Detect const declarations
        if trimmed.starts_with("const ") {
            let after_const = trimmed.strip_prefix("const ").unwrap_or("");
            let name = after_const
                .split(|c: char| c == '=' || c == '(' || c.is_whitespace())
                .next()
                .unwrap_or("")
                .to_string();

            if !name.is_empty() && name != "(" {
                let visibility = if is_exported(&name) {
                    Visibility::Public
                } else {
                    Visibility::Private
                };
                items.push(SourceItem {
                    kind: ItemKind::Constant,
                    name,
                    visibility,
                    doc_comment: non_empty_doc(&current_doc),
                    file_path: file_path.to_path_buf(),
                    line_number: line_num + 1,
                    children: Vec::new(),
                });
            }
            current_doc.clear();
        }

        // Clear doc on non-comment, non-declaration lines
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            current_doc.clear();
        }
    }

    Ok(items)
}

/// Extract function name from Go func declaration
fn extract_go_func_name(s: &str) -> String {
    s.split(|c: char| c == '(' || c == '[' || c.is_whitespace())
        .next()
        .unwrap_or("")
        .to_string()
}

/// Generate documentation sections from project structure
pub fn generate_sections(structure: &ProjectStructure) -> Vec<GeneratedSection> {
    let mut sections = Vec::new();
    let mut sort_order = 0;

    // Overview section
    sort_order += 1;
    sections.push(GeneratedSection {
        section_id: "1".to_string(),
        title: "Overview".to_string(),
        level: 1,
        sort_order,
        content: format!(
            "This is a {} project.\n\n- **Files**: {}\n- **Lines of code**: {}\n",
            structure.language.as_str(),
            structure.file_count,
            structure.total_lines
        ),
        generated: true,
        source_file: None,
    });

    // Group items by kind
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut traits = Vec::new();
    let mut functions = Vec::new();
    let mut modules = Vec::new();

    for item in &structure.modules {
        match item.kind {
            ItemKind::Struct => structs.push(item),
            ItemKind::Enum => enums.push(item),
            ItemKind::Trait => traits.push(item),
            ItemKind::Function if item.visibility == Visibility::Public => functions.push(item),
            ItemKind::Module => modules.push(item),
            _ => {}
        }
    }

    // Modules section
    if !modules.is_empty() {
        sort_order += 1;
        let module_list = modules
            .iter()
            .map(|m| format!("- `{}`", m.name))
            .collect::<Vec<_>>()
            .join("\n");

        sections.push(GeneratedSection {
            section_id: format!("{}", sort_order),
            title: "Modules".to_string(),
            level: 1,
            sort_order,
            content: format!("The project is organized into the following modules:\n\n{}", module_list),
            generated: true,
            source_file: None,
        });
    }

    // Data Types section
    if !structs.is_empty() || !enums.is_empty() {
        sort_order += 1;
        let section_id = sort_order;
        sections.push(GeneratedSection {
            section_id: format!("{}", section_id),
            title: "Data Types".to_string(),
            level: 1,
            sort_order,
            content: "Key data structures used in the project.".to_string(),
            generated: true,
            source_file: None,
        });

        // Add structs
        for item in &structs {
            if item.visibility == Visibility::Public {
                sort_order += 1;
                let content = item.doc_comment.clone().unwrap_or_else(|| {
                    format!("Defined in `{}`", item.file_path.file_name().and_then(|n| n.to_str()).unwrap_or(""))
                });
                sections.push(GeneratedSection {
                    section_id: format!("{}.{}", section_id, sort_order - section_id),
                    title: format!("{} (struct)", item.name),
                    level: 2,
                    sort_order,
                    content,
                    generated: true,
                    source_file: Some(item.file_path.to_string_lossy().to_string()),
                });
            }
        }

        // Add enums
        for item in &enums {
            if item.visibility == Visibility::Public {
                sort_order += 1;
                let content = item.doc_comment.clone().unwrap_or_else(|| {
                    format!("Defined in `{}`", item.file_path.file_name().and_then(|n| n.to_str()).unwrap_or(""))
                });
                sections.push(GeneratedSection {
                    section_id: format!("{}.{}", section_id, sort_order - section_id),
                    title: format!("{} (enum)", item.name),
                    level: 2,
                    sort_order,
                    content,
                    generated: true,
                    source_file: Some(item.file_path.to_string_lossy().to_string()),
                });
            }
        }
    }

    // Traits section
    if !traits.is_empty() {
        sort_order += 1;
        let section_id = sort_order;
        sections.push(GeneratedSection {
            section_id: format!("{}", section_id),
            title: "Traits".to_string(),
            level: 1,
            sort_order,
            content: "Trait definitions that define shared behavior.".to_string(),
            generated: true,
            source_file: None,
        });

        for item in &traits {
            if item.visibility == Visibility::Public {
                sort_order += 1;
                let content = item.doc_comment.clone().unwrap_or_else(|| {
                    format!("Defined in `{}`", item.file_path.file_name().and_then(|n| n.to_str()).unwrap_or(""))
                });
                sections.push(GeneratedSection {
                    section_id: format!("{}.{}", section_id, sort_order - section_id),
                    title: item.name.clone(),
                    level: 2,
                    sort_order,
                    content,
                    generated: true,
                    source_file: Some(item.file_path.to_string_lossy().to_string()),
                });
            }
        }
    }

    // Public Functions section
    if !functions.is_empty() {
        sort_order += 1;
        let section_id = sort_order;
        sections.push(GeneratedSection {
            section_id: format!("{}", section_id),
            title: "Public Functions".to_string(),
            level: 1,
            sort_order,
            content: "Public functions exposed by the project.".to_string(),
            generated: true,
            source_file: None,
        });

        for item in &functions {
            sort_order += 1;
            let content = item.doc_comment.clone().unwrap_or_else(|| {
                format!("Defined in `{}`", item.file_path.file_name().and_then(|n| n.to_str()).unwrap_or(""))
            });
            sections.push(GeneratedSection {
                section_id: format!("{}.{}", section_id, sort_order - section_id),
                title: format!("{}()", item.name),
                level: 2,
                sort_order,
                content,
                generated: true,
                source_file: Some(item.file_path.to_string_lossy().to_string()),
            });
        }
    }

    sections
}

/// A section to be inserted into the docs database
#[derive(Debug)]
pub struct GeneratedSection {
    pub section_id: String,
    pub title: String,
    pub level: i32,
    pub sort_order: i32,
    pub content: String,
    pub generated: bool,
    pub source_file: Option<String>,
}
