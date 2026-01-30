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
    /// Whether to auto-commit on session end (opt-in, default false)
    #[serde(default)]
    pub auto_commit: bool,
    /// How to handle auto-commit: "auto" (silent) or "prompt" (ask each time)
    #[serde(default = "default_auto_commit_mode")]
    pub auto_commit_mode: String,
    /// Whether to auto-commit when a task is completed (default true)
    #[serde(default = "default_true")]
    pub auto_commit_on_task: bool,
}

fn default_auto_commit_mode() -> String {
    "prompt".to_string()
}

fn default_true() -> bool {
    true
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
            auto_commit: false,
            auto_commit_mode: "prompt".to_string(),
            auto_commit_on_task: true,
        }
    }
}

impl ProjectConfig {
    /// Load config from the project's .tracking/config.json
    pub fn load() -> anyhow::Result<Self> {
        let config_path = crate::paths::get_config_path()?;
        let content = std::fs::read_to_string(&config_path)?;
        let config: ProjectConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save config to the project's .tracking/config.json
    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = crate::paths::get_config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
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
