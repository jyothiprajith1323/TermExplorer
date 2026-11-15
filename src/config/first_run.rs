//! First-run experience detection, state tracking, and upgrade verification.
//!
//! Stored at `~/.local/share/termexplorer/first_run.json` adhering to XDG base directory specifications.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CURRENT_APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirstRunStatus {
    /// True first-time user: neither configuration nor first-run state file exists.
    FreshFirstLaunch,
    /// Existing user with a valid configuration where onboarding has already been completed.
    ExistingConfigValid,
    /// Existing user with an older/unversioned configuration (migrated without interrupting user).
    ExistingConfigLegacy,
    /// Configuration file exists but has syntax errors or corruption.
    ExistingConfigCorrupted,
    /// Application was upgraded from an earlier version.
    ApplicationUpgrade {
        previous_version: String,
        current_version: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FirstRunState {
    pub completed: bool,
    pub completed_at: Option<String>,
    pub app_version: String,
    pub launch_count: u64,
}

impl Default for FirstRunState {
    fn default() -> Self {
        Self {
            completed: false,
            completed_at: None,
            app_version: CURRENT_APP_VERSION.to_string(),
            launch_count: 0,
        }
    }
}

pub struct FirstRunManager;

impl FirstRunManager {
    /// Return the XDG data directory path for first-run state.
    pub fn state_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("termexplorer")
    }

    /// Return the full path to `first_run.json`.
    pub fn state_path() -> PathBuf {
        Self::state_dir().join("first_run.json")
    }

    /// Load the persistent first-run state from disk.
    pub fn load_state() -> Option<FirstRunState> {
        Self::load_state_from_path(&Self::state_path())
    }

    /// Load the persistent first-run state from an explicit path.
    pub fn load_state_from_path(path: &Path) -> Option<FirstRunState> {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str::<FirstRunState>(&content) {
                    return Some(state);
                }
            }
        }
        None
    }

    /// Persist the first-run state atomically to disk.
    pub fn save_state(state: &FirstRunState) -> Result<()> {
        Self::save_state_to_path(state, &Self::state_path())
    }

    /// Persist the first-run state atomically to an explicit path.
    pub fn save_state_to_path(state: &FirstRunState, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Creating state directory: {}", parent.display()))?;
        }

        let content =
            serde_json::to_string_pretty(state).context("Serializing first-run state to JSON")?;

        crate::filesystem::operations::save_file_atomic(path, content.as_bytes())
            .with_context(|| format!("Writing first-run state atomically: {}", path.display()))?;

        Ok(())
    }

    /// Detect the first-run status of the workstation using default state path.
    pub fn detect_status(config_path: &Path) -> FirstRunStatus {
        Self::detect_status_with_paths(config_path, None)
    }

    /// Detect the first-run status with optional explicit state path (useful for testing).
    pub fn detect_status_with_paths(
        config_path: &Path,
        custom_state_path: Option<&Path>,
    ) -> FirstRunStatus {
        let state_opt = match custom_state_path {
            Some(p) => Self::load_state_from_path(p),
            None => Self::load_state(),
        };
        let config_exists = config_path.exists();

        // Check if state file already exists
        if let Some(state) = state_opt {
            if state.completed {
                if state.app_version != CURRENT_APP_VERSION {
                    return FirstRunStatus::ApplicationUpgrade {
                        previous_version: state.app_version,
                        current_version: CURRENT_APP_VERSION.to_string(),
                    };
                }
                return FirstRunStatus::ExistingConfigValid;
            }
        }

        // State file is absent or incomplete: inspect configuration file
        if config_exists {
            match std::fs::read_to_string(config_path) {
                Ok(content) => {
                    // Try parsing as raw TOML Value
                    if let Ok(table) = toml::from_str::<toml::Value>(&content) {
                        // Check if schema_version exists
                        if let Some(ver) = table.get("schema_version").and_then(|v| v.as_integer())
                        {
                            if ver >= 2 {
                                // If general.first_run_completed is true
                                if let Some(gen) = table.get("general") {
                                    if gen
                                        .get("first_run_completed")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false)
                                    {
                                        return FirstRunStatus::ExistingConfigValid;
                                    }
                                }
                            }
                        }
                        // Existing configuration from Sprint 1-6A without schema_version or v1
                        FirstRunStatus::ExistingConfigLegacy
                    } else {
                        FirstRunStatus::ExistingConfigCorrupted
                    }
                }
                Err(_) => FirstRunStatus::ExistingConfigCorrupted,
            }
        } else {
            // Neither state file nor config file exists -> Genuine brand new user launch
            FirstRunStatus::FreshFirstLaunch
        }
    }

    /// Mark first-run onboarding as completed and persist to disk.
    pub fn mark_completed() -> Result<()> {
        let mut state = Self::load_state().unwrap_or_default();
        state.completed = true;
        state.completed_at = Some(chrono::Utc::now().to_rfc3339());
        state.app_version = CURRENT_APP_VERSION.to_string();
        state.launch_count += 1;
        Self::save_state(&state)
    }

    /// Increment launch count without changing completed status.
    pub fn record_launch() -> Result<()> {
        let mut state = Self::load_state().unwrap_or_default();
        state.launch_count += 1;
        state.app_version = CURRENT_APP_VERSION.to_string();
        Self::save_state(&state)
    }

    /// Reset first-run state (for testing or user request to replay onboarding).
    pub fn reset() -> Result<()> {
        let path = Self::state_path();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        Ok(())
    }
}
