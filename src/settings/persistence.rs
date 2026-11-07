//! Configuration persistence, validation, and automated backups.

use crate::config::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct SettingsPersistence;

impl SettingsPersistence {
    /// Validate config values to ensure boundaries are respected.
    pub fn validate(config: &mut Config) -> Vec<String> {
        let mut warnings = Vec::new();

        if config.sidebar_width < 10 || config.sidebar_width > 50 {
            warnings.push(format!(
                "Sidebar width {} clamped to 10..50",
                config.sidebar_width
            ));
            config.sidebar_width = config.sidebar_width.clamp(10, 50);
        }

        if config.preview_width < 15 || config.preview_width > 80 {
            warnings.push(format!(
                "Preview width {} clamped to 15..80",
                config.preview_width
            ));
            config.preview_width = config.preview_width.clamp(15, 80);
        }

        if config.terminal_height < 5 || config.terminal_height > 50 {
            warnings.push(format!(
                "Terminal height {} clamped to 5..50",
                config.terminal_height
            ));
            config.terminal_height = config.terminal_height.clamp(5, 50);
        }

        if config.monitor_height < 5 || config.monitor_height > 50 {
            warnings.push(format!(
                "Monitor height {} clamped to 5..50",
                config.monitor_height
            ));
            config.monitor_height = config.monitor_height.clamp(5, 50);
        }

        warnings
    }

    /// Save configuration with automatic backup.
    pub fn save_with_backup(config: &Config) -> Result<()> {
        let config_path = Config::config_path();

        // Create backup if existing config exists
        if config_path.exists() {
            let backup_path = config_path.with_extension("toml.bak");
            let _ = fs::copy(&config_path, &backup_path);
        }

        config.save()
    }

    /// Restore configuration from backup file if available.
    pub fn restore_backup() -> Result<Config> {
        let backup_path = Config::config_path().with_extension("toml.bak");
        let content = fs::read_to_string(&backup_path)
            .with_context(|| format!("Reading backup config: {}", backup_path.display()))?;
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Parsing backup config: {}", backup_path.display()))?;
        config.save()?;
        Ok(config)
    }

    /// Export configuration to a specified external path.
    pub fn export_to_file(config: &Config, target_path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(config)?;
        fs::write(target_path, content)?;
        Ok(())
    }

    /// Import configuration from a specified external path.
    pub fn import_from_file(source_path: &Path) -> Result<Config> {
        let content = fs::read_to_string(source_path)?;
        let mut config: Config = toml::from_str(&content)?;
        Self::validate(&mut config);
        config.save()?;
        Ok(config)
    }
}
