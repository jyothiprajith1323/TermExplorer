//! Plugin Manager — discovers, loads, verifies, and lifecycle-manages plugins.

use std::fs;
use std::path::{Path, PathBuf};

use super::manifest::PluginManifest;
use super::sandbox::PluginInstance;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginState {
    Active,
    Disabled,
    Error(String),
}

impl PluginState {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "Active",
            Self::Disabled => "Disabled",
            Self::Error(_) => "Error",
        }
    }
}

pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub state: PluginState,
    pub path: PathBuf,
    pub instance: Option<PluginInstance>,
}

pub struct PluginManager {
    pub plugins: Vec<LoadedPlugin>,
    pub workspace_path: String,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            workspace_path: ".".to_string(),
        }
    }

    /// User plugins directory (`~/.local/share/termexplorer/plugins/`).
    pub fn user_plugins_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("termexplorer")
                .join("plugins")
        } else {
            PathBuf::from(".local/share/termexplorer/plugins")
        }
    }

    /// Discover and register all plugins in the given plugins root directory without upfront Wasm compilation.
    pub fn discover_and_load(&mut self, root: &Path, workspace_path: &str) {
        self.workspace_path = workspace_path.to_string();
        if !root.exists() || !root.is_dir() {
            return;
        }

        if let Ok(entries) = fs::read_dir(root) {
            for entry in entries.flatten() {
                let dir_path = entry.path();
                if dir_path.is_dir() {
                    let manifest_file = dir_path.join("plugin.toml");
                    if manifest_file.exists() {
                        self.load_plugin_dir(&dir_path, workspace_path);
                    }
                }
            }
        }
    }

    /// Load a single plugin directory: parses manifest and validates entrypoint file lazily.
    pub fn load_plugin_dir(&mut self, dir: &Path, workspace_path: &str) {
        self.workspace_path = workspace_path.to_string();
        let manifest_path = dir.join("plugin.toml");
        let manifest = match PluginManifest::load_from_file(&manifest_path) {
            Ok(m) => m,
            Err(_e) => {
                // If manifest fails to parse, we record as error if ID can be guessed or ignore
                return;
            }
        };

        // Don't duplicate if already loaded
        if self.plugins.iter().any(|p| p.manifest.id == manifest.id) {
            return;
        }

        let wasm_file = dir.join(&manifest.entrypoint);
        if !wasm_file.exists() {
            self.plugins.push(LoadedPlugin {
                manifest,
                state: PluginState::Error(format!(
                    "Entrypoint '{}' not found",
                    wasm_file.display()
                )),
                path: dir.to_path_buf(),
                instance: None,
            });
            return;
        }

        // Lazy initialization: Register plugin with manifest metadata, deferring Wasm compilation.
        self.plugins.push(LoadedPlugin {
            manifest,
            state: PluginState::Active,
            path: dir.to_path_buf(),
            instance: None,
        });
    }

    /// On-demand initialization and compilation of a plugin's WebAssembly instance.
    pub fn init_instance(&mut self, id: &str, workspace_path: &str) -> Result<(), String> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.manifest.id == id)
            .ok_or_else(|| format!("Plugin '{}' not found", id))?;

        if plugin.instance.is_some() {
            return Ok(());
        }

        let wasm_file = plugin.path.join(&plugin.manifest.entrypoint);
        let wasm_bytes = fs::read(&wasm_file).map_err(|e| e.to_string())?;

        match PluginInstance::new(plugin.manifest.clone(), &wasm_bytes, workspace_path) {
            Ok(instance) => {
                plugin.instance = Some(instance);
                plugin.state = PluginState::Active;
                Ok(())
            }
            Err(e) => {
                plugin.state = PluginState::Error(e.clone());
                Err(e)
            }
        }
    }

    /// Initialize all discovered plugin instances (used for batch testing and pre-warming).
    pub fn init_all_instances(&mut self, workspace_path: &str) {
        let ids: Vec<String> = self.plugins.iter().map(|p| p.manifest.id.clone()).collect();
        for id in ids {
            let _ = self.init_instance(&id, workspace_path);
        }
    }

    /// Enable a plugin by ID.
    pub fn enable(&mut self, id: &str, workspace_path: &str) -> Result<(), String> {
        self.init_instance(id, workspace_path)?;
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.manifest.id == id)
            .ok_or_else(|| format!("Plugin '{}' not found", id))?;

        plugin.state = PluginState::Active;
        Ok(())
    }

    /// Disable a plugin by ID.
    pub fn disable(&mut self, id: &str) {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.manifest.id == id) {
            plugin.instance = None;
            plugin.state = PluginState::Disabled;
        }
    }

    /// Dispatch command to a specific plugin.
    pub fn trigger_command(&mut self, id: &str, cmd: &str) -> Result<i32, String> {
        let workspace = self.workspace_path.clone();
        let _ = self.init_instance(id, &workspace);
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.manifest.id == id)
            .ok_or_else(|| format!("Plugin '{}' not found", id))?;

        if plugin.state != PluginState::Active {
            return Err(format!("Plugin '{}' is not active", id));
        }

        if let Some(instance) = &mut plugin.instance {
            instance.trigger_command(cmd)
        } else {
            Err("No active plugin instance".to_string())
        }
    }

    /// Collect status texts from all active plugins with `status_text` permission.
    pub fn collect_status_texts(&mut self) -> Vec<(String, String)> {
        let mut res = Vec::new();
        for plugin in &mut self.plugins {
            if plugin.state == PluginState::Active {
                if let Some(instance) = &mut plugin.instance {
                    if let Some(txt) = instance.poll_status_text() {
                        res.push((plugin.manifest.name.clone(), txt));
                    }
                }
            }
        }
        res
    }

    /// Update active file context for all active plugins.
    pub fn update_active_file(&mut self, filename: &str, size: u64) {
        for plugin in &mut self.plugins {
            if let Some(instance) = &mut plugin.instance {
                instance.set_active_context(filename, size);
            }
        }
    }

    /// Retrieve the first active status text from loaded plugins.
    pub fn first_active_status_text(&self) -> Option<(&str, &str)> {
        for p in &self.plugins {
            if p.state == PluginState::Active {
                if let Some(inst) = &p.instance {
                    if let Some(st) = inst.current_status_text() {
                        return Some((&p.manifest.name, st));
                    }
                }
            }
        }
        None
    }
}
