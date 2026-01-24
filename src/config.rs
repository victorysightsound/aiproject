// Config module - Full implementation in Task #5

use serde::{Deserialize, Serialize};

/// Project configuration stored in .tracking/config.json
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub project_type: String,
    pub description: Option<String>,
    pub schema_version: String,
    pub auto_backup: bool,
    pub auto_session: bool,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            project_type: String::new(),
            description: None,
            schema_version: crate::SCHEMA_VERSION.to_string(),
            auto_backup: true,
            auto_session: true,
        }
    }
}

/// Global registry entry
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub path: String,
    pub name: String,
    #[serde(alias = "type")]
    pub project_type: String,
    pub registered_at: String,
    pub schema_version: String,
}

/// Global registry stored in ~/.proj/registry.json
#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub registered_projects: Vec<RegistryEntry>,
    pub current_schema_version: String,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            registered_projects: Vec::new(),
            current_schema_version: crate::SCHEMA_VERSION.to_string(),
        }
    }
}
