//! Plugin manifest definition (`plugin.toml`).

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::permissions::PluginPermission;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    #[serde(default = "default_entrypoint")]
    pub entrypoint: String,
    #[serde(default)]
    pub permissions: HashSet<PluginPermission>,
    pub sha256: Option<String>,
}

fn default_entrypoint() -> String {
    "plugin.wasm".to_string()
}

impl PluginManifest {
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        toml::from_str(content).map_err(|e| format!("Invalid plugin manifest: {}", e))
    }

    pub fn has_permission(&self, perm: PluginPermission) -> bool {
        self.permissions.contains(&perm)
    }
}
