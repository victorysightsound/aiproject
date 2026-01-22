// Extend command - add extension tables to project database

use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::config::ProjectConfig;
use crate::database::open_database;
use crate::paths::{get_config_path, get_tracking_db_path};

/// Valid extension types
const VALID_EXTENSIONS: &[&str] = &["book", "sermon", "api", "course"];

/// Extension schemas
const BOOK_EXTENSION: &str = r#"
CREATE TABLE IF NOT EXISTS book_chapters (
    chapter_id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    summary TEXT,
    word_count INTEGER DEFAULT 0,
    status TEXT DEFAULT 'draft',
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS book_notes (
    note_id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id INTEGER,
    content TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (chapter_id) REFERENCES book_chapters(chapter_id)
);
"#;

const SERMON_EXTENSION: &str = r#"
CREATE TABLE IF NOT EXISTS sermons (
    sermon_id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    scripture_ref TEXT,
    scripture_text TEXT,
    outline TEXT,
    full_text TEXT,
    status TEXT DEFAULT 'draft',
    preached_date TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS sermon_points (
    point_id INTEGER PRIMARY KEY AUTOINCREMENT,
    sermon_id INTEGER NOT NULL,
    point_number INTEGER NOT NULL,
    heading TEXT NOT NULL,
    content TEXT,
    FOREIGN KEY (sermon_id) REFERENCES sermons(sermon_id)
);
"#;

const API_EXTENSION: &str = r#"
CREATE TABLE IF NOT EXISTS api_endpoints (
    endpoint_id INTEGER PRIMARY KEY AUTOINCREMENT,
    method TEXT NOT NULL,
    path TEXT NOT NULL,
    description TEXT,
    request_schema TEXT,
    response_schema TEXT,
    status TEXT DEFAULT 'planned',
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS api_models (
    model_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    schema TEXT NOT NULL,
    description TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
"#;

const COURSE_EXTENSION: &str = r#"
CREATE TABLE IF NOT EXISTS course_modules (
    module_id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT DEFAULT 'draft',
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS course_lessons (
    lesson_id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL,
    lesson_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    duration_minutes INTEGER,
    status TEXT DEFAULT 'draft',
    FOREIGN KEY (module_id) REFERENCES course_modules(module_id)
);
"#;

pub fn run(extension_type: String) -> Result<()> {
    // Validate extension type
    if !VALID_EXTENSIONS.contains(&extension_type.as_str()) {
        bail!(
            "Invalid extension type: {}\nValid types: {}",
            extension_type,
            VALID_EXTENSIONS.join(", ")
        );
    }

    // Load config
    let config = load_config()?;

    // Check if already applied (by checking for a marker table)
    let db_path = get_tracking_db_path()?;
    let conn = open_database(&db_path)?;

    // Check if extension already exists
    let table_name = match extension_type.as_str() {
        "book" => "book_chapters",
        "sermon" => "sermons",
        "api" => "api_endpoints",
        "course" => "course_modules",
        _ => unreachable!(),
    };

    let exists: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
            [table_name],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if exists > 0 {
        println!("Extension '{}' already applied.", extension_type);
        return Ok(());
    }

    // Apply extension schema
    let schema = match extension_type.as_str() {
        "book" => BOOK_EXTENSION,
        "sermon" => SERMON_EXTENSION,
        "api" => API_EXTENSION,
        "course" => COURSE_EXTENSION,
        _ => unreachable!(),
    };

    conn.execute_batch(schema)
        .with_context(|| format!("Failed to apply '{}' extension", extension_type))?;

    // Update config to track extension
    let config_path = get_config_path()?;
    let content = std::fs::read_to_string(&config_path)?;
    let mut config_json: serde_json::Value = serde_json::from_str(&content)?;

    // Add to extensions array
    let extensions = config_json
        .get_mut("extensions")
        .and_then(|e| e.as_array_mut());

    match extensions {
        Some(arr) => {
            arr.push(serde_json::Value::String(extension_type.clone()));
        }
        None => {
            config_json["extensions"] =
                serde_json::Value::Array(vec![serde_json::Value::String(extension_type.clone())]);
        }
    }

    std::fs::write(&config_path, serde_json::to_string_pretty(&config_json)?)?;

    println!(
        "{} Applied '{}' extension to {}",
        "✓".green(),
        extension_type,
        config.name
    );

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
