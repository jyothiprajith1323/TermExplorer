//! Deterministic configuration schema versioning and migrations.
//!
//! Handles migration from unversioned (v1) configurations to Schema Version 2,
//! ensuring backward compatibility, user setting preservation, and safe recovery.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

pub const CURRENT_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub initial_version: u32,
    pub final_version: u32,
    pub migrated: bool,
    pub backup_path: Option<PathBuf>,
    pub changes: Vec<String>,
}

pub struct ConfigMigrator;

impl ConfigMigrator {
    /// Detect the schema version of a raw TOML configuration string.
    pub fn detect_version(raw_toml: &str) -> u32 {
        if let Ok(value) = toml::from_str::<toml::Value>(raw_toml) {
            if let Some(v) = value.get("schema_version").and_then(|v| v.as_integer()) {
                return v as u32;
            }
        }
        // If schema_version is absent, it represents an unversioned v1 schema
        1
    }

    /// Migrate raw TOML to the current Schema Version 2.
    pub fn migrate(
        raw_toml: &str,
        config_dir: Option<&Path>,
    ) -> Result<(super::Config, MigrationReport)> {
        let initial_version = Self::detect_version(raw_toml);

        if initial_version > CURRENT_SCHEMA_VERSION {
            bail!(
                "Configuration schema version {} is newer than supported version {}. Manual upgrade required.",
                initial_version,
                CURRENT_SCHEMA_VERSION
            );
        }

        let mut changes = Vec::new();
        let mut migrated = false;
        let mut backup_path = None;

        if initial_version < 2 {
            // Apply migration v1 -> v2
            migrated = true;
            changes.push("Upgraded schema_version to 2".to_string());
            changes.push("Added typed [general] table with first_run_completed = true".to_string());
            changes.push("Added typed [terminal] table".to_string());
            changes.push("Added typed [search] table".to_string());
            changes.push("Added typed [developer] table".to_string());

            // Create v1 backup if directory provided
            if let Some(dir) = config_dir {
                let original_file = dir.join("config.toml");
                if original_file.exists() {
                    let v1_bak = dir.join("config.toml.v1.bak");
                    if std::fs::copy(&original_file, &v1_bak).is_ok() {
                        backup_path = Some(v1_bak);
                    }
                }
            }
        }

        // Parse into typed Config
        let mut config: super::Config = toml::from_str(raw_toml)
            .context("Parsing migrated configuration into typed structure")?;

        // Ensure schema version is set
        config.schema_version = CURRENT_SCHEMA_VERSION;

        // If this was a legacy v1 config, ensure first_run_completed is marked true
        if initial_version < 2 {
            config.general.first_run_completed = true;
        }

        // Validate and clamp all fields
        config.validate_and_clamp();

        let report = MigrationReport {
            initial_version,
            final_version: CURRENT_SCHEMA_VERSION,
            migrated,
            backup_path,
            changes,
        };

        Ok((config, report))
    }
}
