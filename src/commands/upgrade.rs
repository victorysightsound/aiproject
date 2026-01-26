// Upgrade command - database schema migration system

use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;
use crate::config::{ProjectConfig, Registry};
use crate::database::{get_schema_version, open_database, set_schema_version};
use crate::paths::{get_config_path, get_registry_path, get_tracking_db_path};
use crate::SCHEMA_VERSION;

/// Schema change definition
struct SchemaChange {
    risk: &'static str,
    description: &'static str,
    sql: &'static str,
    verify: &'static str,
}

/// Schema upgrade definition
struct SchemaUpgrade {
    from_version: &'static str,
    to_version: &'static str,
    changes: &'static [SchemaChange],
}

/// Upgrade registry - all schema changes between versions
const UPGRADE_REGISTRY: &[SchemaUpgrade] = &[
    SchemaUpgrade {
        from_version: "1.0",
        to_version: "1.1",
        changes: &[
            SchemaChange {
                risk: "safe",
                description: "Track context snapshots for delta updates",
                sql: "CREATE TABLE IF NOT EXISTS context_snapshots (
                    snapshot_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    session_id INTEGER,
                    created_at TEXT DEFAULT (datetime('now')),
                    snapshot_type TEXT NOT NULL,
                    content_hash TEXT NOT NULL,
                    item_counts TEXT,
                    FOREIGN KEY (session_id) REFERENCES sessions(session_id)
                )",
                verify: "SELECT 1 FROM sqlite_master WHERE type='table' AND name='context_snapshots'",
            },
            SchemaChange {
                risk: "safe",
                description: "Store compressed session summaries",
                sql: "CREATE TABLE IF NOT EXISTS compressed_sessions (
                    compression_id INTEGER PRIMARY KEY AUTOINCREMENT,
                    created_at TEXT DEFAULT (datetime('now')),
                    session_ids TEXT NOT NULL,
                    date_range_start TEXT,
                    date_range_end TEXT,
                    compressed_summary TEXT NOT NULL,
                    original_token_estimate INTEGER,
                    compressed_token_estimate INTEGER
                )",
                verify: "SELECT 1 FROM sqlite_master WHERE type='table' AND name='compressed_sessions'",
            },
            SchemaChange {
                risk: "safe",
                description: "Index for context snapshot lookups",
                sql: "CREATE INDEX IF NOT EXISTS idx_context_snapshots_session ON context_snapshots(session_id)",
                verify: "SELECT 1 FROM sqlite_master WHERE type='index' AND name='idx_context_snapshots_session'",
            },
        ],
    },
    SchemaUpgrade {
        from_version: "1.1",
        to_version: "1.2",
        changes: &[
            SchemaChange {
                risk: "safe",
                description: "Track whether full context was shown this session",
                sql: "ALTER TABLE sessions ADD COLUMN full_context_shown INTEGER DEFAULT 0",
                verify: "SELECT full_context_shown FROM sessions LIMIT 0",
            },
        ],
    },
    SchemaUpgrade {
        from_version: "1.2",
        to_version: "1.3",
        changes: &[
            SchemaChange {
                risk: "safe",
                description: "FTS5 virtual table for full-text search across decisions, notes, tasks",
                sql: "CREATE VIRTUAL TABLE IF NOT EXISTS tracking_fts USING fts5(content, table_name, record_id, content='', tokenize='porter')",
                verify: "SELECT 1 FROM sqlite_master WHERE type='table' AND name='tracking_fts'",
            },
        ],
    },
];

/// Upgrade compatibility result
struct UpgradeCompatibility {
    can_upgrade: bool,
    current_version: String,
    target_version: String,
    pending_upgrades: Vec<&'static SchemaUpgrade>,
    safe_changes: Vec<ChangeInfo>,
    warnings: Vec<String>,
    errors: Vec<String>,
}

/// Change info for display
struct ChangeInfo {
    description: String,
    risk: String,
    status: String,
}

pub fn run(info: bool, all: bool, auto: bool) -> Result<()> {
    if all {
        upgrade_all_projects(info, auto)
    } else {
        upgrade_current_project(info)
    }
}

/// Upgrade all registered projects
fn upgrade_all_projects(info_mode: bool, auto_mode: bool) -> Result<()> {
    let registry = load_registry()?;

    if registry.registered_projects.is_empty() {
        println!("No projects registered.");
        return Ok(());
    }

    println!(
        "\nChecking {} registered project(s)...\n",
        registry.registered_projects.len()
    );

    let mut upgradeable = Vec::new();
    let mut up_to_date = Vec::new();
    let mut errors = Vec::new();

    for proj in &registry.registered_projects {
        let proj_path = Path::new(&proj.path);
        if !proj_path.exists() {
            errors.push((proj.name.clone(), "Path not found".to_string()));
            continue;
        }

        let db_path = proj_path.join(".tracking").join("tracking.db");
        if !db_path.exists() {
            errors.push((proj.name.clone(), "No tracking.db".to_string()));
            continue;
        }

        match check_upgrade_compatibility(&db_path) {
            Ok(compat) => {
                if compat.current_version == compat.target_version {
                    up_to_date.push((proj.name.clone(), compat.current_version.clone()));
                } else if compat.can_upgrade {
                    upgradeable.push((proj.name.clone(), proj.path.clone(), compat));
                } else {
                    let err_msg = compat.errors.join("; ");
                    errors.push((proj.name.clone(), err_msg));
                }
            }
            Err(e) => {
                errors.push((proj.name.clone(), e.to_string()));
            }
        }
    }

    // Show summary
    if !up_to_date.is_empty() {
        println!("Already up to date ({}):", up_to_date.len());
        for (name, version) in &up_to_date {
            println!("  {} {} (v{})", "✓".green(), name, version);
        }
    }

    if !upgradeable.is_empty() {
        println!("\nReady to upgrade ({}):", upgradeable.len());
        for (name, _, compat) in &upgradeable {
            println!(
                "  {} {}: v{} → v{}",
                "↑".cyan(),
                name,
                compat.current_version,
                compat.target_version
            );
            if info_mode {
                println!("    Changes:");
                for change in &compat.safe_changes {
                    if change.status == "pending" {
                        println!("      + {}", change.description);
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        println!("\nCannot upgrade ({}):", errors.len());
        for (name, err) in &errors {
            println!("  {} {}: {}", "✗".red(), name, err);
        }
    }

    if upgradeable.is_empty() {
        println!("\nNo projects need upgrading.");
        return Ok(());
    }

    if info_mode {
        println!("\n[DRY-RUN] Would upgrade {} project(s)", upgradeable.len());
        return Ok(());
    }

    // Confirm upgrade
    if !auto_mode {
        println!("\nUpgrade {} project(s)?", upgradeable.len());
        print!("Type 'yes' to continue: ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "yes" {
            println!("Upgrade cancelled.");
            return Ok(());
        }
    }

    // Perform upgrades
    println!("\nUpgrading {} project(s)...\n", upgradeable.len());
    let mut success_count = 0;
    let mut fail_count = 0;

    for (name, path, _) in upgradeable {
        let db_path = Path::new(&path).join(".tracking").join("tracking.db");
        let config_path = Path::new(&path).join(".tracking").join("config.json");

        match apply_upgrades(&db_path, &config_path) {
            Ok(_) => {
                println!(
                    "  {} {}: Upgraded to v{}",
                    "✓".green(),
                    name,
                    SCHEMA_VERSION
                );
                success_count += 1;
            }
            Err(e) => {
                println!("  {} {}: {}", "✗".red(), name, e);
                fail_count += 1;
            }
        }
    }

    println!(
        "\nUpgrade complete: {} succeeded, {} failed",
        success_count, fail_count
    );
    Ok(())
}

/// Upgrade the current project
fn upgrade_current_project(info_mode: bool) -> Result<()> {
    let _config = load_config()?;
    let db_path = get_tracking_db_path()?;
    let config_path = get_config_path()?;

    let compat = check_upgrade_compatibility(&db_path)?;

    if !compat.can_upgrade {
        if compat.current_version == compat.target_version {
            println!("Database is up to date (v{}).", compat.current_version);
        } else {
            println!("Cannot upgrade: {}", compat.errors.join("; "));
        }
        return Ok(());
    }

    // Show upgrade info
    println!(
        "Upgrade available: v{} → v{}",
        compat.current_version, compat.target_version
    );
    println!();
    println!("Changes to apply:");
    for change in &compat.safe_changes {
        if change.status == "pending" {
            let risk_indicator = match change.risk.as_str() {
                "safe" => "+".green(),
                "moderate" => "~".yellow(),
                _ => "!".red(),
            };
            println!(
                "  {} {} ({})",
                risk_indicator, change.description, change.risk
            );
        }
    }

    if info_mode {
        println!("\n[DRY-RUN] No changes made.");
        return Ok(());
    }

    // Apply upgrades
    println!();
    apply_upgrades(&db_path, &config_path)?;
    println!("{} Upgraded to v{}", "✓".green(), SCHEMA_VERSION);

    Ok(())
}

/// Check if an upgrade is safe
fn check_upgrade_compatibility(db_path: &Path) -> Result<UpgradeCompatibility> {
    let conn = open_database(db_path)?;
    let current_version =
        get_schema_version(&conn)?.unwrap_or_else(|| "1.0".to_string());
    let target_version = SCHEMA_VERSION.to_string();

    let mut result = UpgradeCompatibility {
        can_upgrade: true,
        current_version: current_version.clone(),
        target_version: target_version.clone(),
        pending_upgrades: Vec::new(),
        safe_changes: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    };

    // Already up to date?
    if current_version == target_version {
        result.can_upgrade = false;
        return Ok(result);
    }

    // Get pending upgrades
    result.pending_upgrades = get_pending_upgrades(&current_version, &target_version);

    if result.pending_upgrades.is_empty() {
        result.warnings.push(format!(
            "No upgrade path found from v{} to v{}",
            current_version, target_version
        ));
        result.can_upgrade = false;
        return Ok(result);
    }

    // Validate each change
    for upgrade in &result.pending_upgrades {
        for change in upgrade.changes {
            let mut change_info = ChangeInfo {
                description: change.description.to_string(),
                risk: change.risk.to_string(),
                status: "pending".to_string(),
            };

            // Check if already applied
            if let Ok(_) = conn.execute(change.verify, []) {
                // Verify query succeeded, change is already applied
                change_info.status = "already_applied".to_string();
            } else if conn.query_row(change.verify, [], |_| Ok(())).is_ok() {
                change_info.status = "already_applied".to_string();
            }

            result.safe_changes.push(change_info);
        }
    }

    Ok(result)
}

/// Apply pending upgrades to the database
fn apply_upgrades(db_path: &Path, config_path: &Path) -> Result<()> {
    let conn = open_database(db_path)?;
    let current_version =
        get_schema_version(&conn)?.unwrap_or_else(|| "1.0".to_string());

    let pending = get_pending_upgrades(&current_version, SCHEMA_VERSION);

    for upgrade in pending {
        for change in upgrade.changes {
            // Check if already applied
            let already_applied = conn.query_row(change.verify, [], |_| Ok(())).is_ok();

            if !already_applied {
                conn.execute_batch(change.sql)
                    .with_context(|| format!("Failed to apply: {}", change.description))?;
            }
        }
    }

    // Update schema version in database
    set_schema_version(&conn, SCHEMA_VERSION)?;

    // Update config file
    if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&content) {
            config["schema_version"] = serde_json::Value::String(SCHEMA_VERSION.to_string());
            let updated = serde_json::to_string_pretty(&config)?;
            std::fs::write(config_path, updated)?;
        }
    }

    Ok(())
}

/// Get upgrades needed between two versions
fn get_pending_upgrades(current: &str, target: &str) -> Vec<&'static SchemaUpgrade> {
    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
    let target_parts: Vec<u32> = target.split('.').filter_map(|s| s.parse().ok()).collect();

    UPGRADE_REGISTRY
        .iter()
        .filter(|upgrade| {
            let from_parts: Vec<u32> = upgrade
                .from_version
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();
            let to_parts: Vec<u32> = upgrade
                .to_version
                .split('.')
                .filter_map(|s| s.parse().ok())
                .collect();

            // Include if current <= from < to <= target
            compare_versions(&current_parts, &from_parts) <= std::cmp::Ordering::Equal
                && compare_versions(&to_parts, &target_parts) <= std::cmp::Ordering::Equal
        })
        .collect()
}

/// Compare version tuples
fn compare_versions(a: &[u32], b: &[u32]) -> std::cmp::Ordering {
    for (av, bv) in a.iter().zip(b.iter()) {
        match av.cmp(bv) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    a.len().cmp(&b.len())
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
