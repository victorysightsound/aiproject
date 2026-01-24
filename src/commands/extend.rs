// Extend command - add extension tables to project database

use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::config::ProjectConfig;
use crate::database::open_database;
use crate::paths::{get_config_path, get_tracking_db_path};

/// Valid extension types
const VALID_EXTENSIONS: &[&str] = &["api", "schema", "releases"];

/// Extension schemas
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

const SCHEMA_EXTENSION: &str = r#"
CREATE TABLE IF NOT EXISTS db_tables (
    table_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    status TEXT DEFAULT 'planned',
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS db_columns (
    column_id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    data_type TEXT NOT NULL,
    nullable INTEGER DEFAULT 1,
    default_value TEXT,
    is_primary_key INTEGER DEFAULT 0,
    is_foreign_key INTEGER DEFAULT 0,
    references_table TEXT,
    references_column TEXT,
    description TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (table_id) REFERENCES db_tables(table_id)
);

CREATE TABLE IF NOT EXISTS db_migrations (
    migration_id INTEGER PRIMARY KEY AUTOINCREMENT,
    version TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    sql_up TEXT,
    sql_down TEXT,
    status TEXT DEFAULT 'pending',
    applied_at TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
"#;

const RELEASES_EXTENSION: &str = r#"
CREATE TABLE IF NOT EXISTS releases (
    release_id INTEGER PRIMARY KEY AUTOINCREMENT,
    version TEXT NOT NULL,
    name TEXT,
    description TEXT,
    status TEXT DEFAULT 'planned',
    release_date TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS release_targets (
    target_id INTEGER PRIMARY KEY AUTOINCREMENT,
    release_id INTEGER NOT NULL,
    platform TEXT NOT NULL,
    environment TEXT DEFAULT 'production',
    status TEXT DEFAULT 'pending',
    deployed_at TEXT,
    notes TEXT,
    FOREIGN KEY (release_id) REFERENCES releases(release_id)
);

CREATE TABLE IF NOT EXISTS release_changes (
    change_id INTEGER PRIMARY KEY AUTOINCREMENT,
    release_id INTEGER NOT NULL,
    change_type TEXT NOT NULL,
    description TEXT NOT NULL,
    breaking INTEGER DEFAULT 0,
    task_id INTEGER,
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (release_id) REFERENCES releases(release_id),
    FOREIGN KEY (task_id) REFERENCES tasks(task_id)
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
        "api" => "api_endpoints",
        "schema" => "db_tables",
        "releases" => "releases",
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
        "api" => API_EXTENSION,
        "schema" => SCHEMA_EXTENSION,
        "releases" => RELEASES_EXTENSION,
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
