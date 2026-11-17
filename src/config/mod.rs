//! Application configuration — loaded from and persisted to TOML.
//! Stored at `~/.config/termexplorer/config.toml` adhering to Schema Version 2.

pub mod first_run;
pub mod migration;

pub use first_run::{FirstRunManager, FirstRunState, FirstRunStatus};
pub use migration::{ConfigMigrator, MigrationReport, CURRENT_SCHEMA_VERSION};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ── Default helpers ──────────────────────────────────────────────────────────

fn default_schema_version() -> u32 {
    CURRENT_SCHEMA_VERSION
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_theme() -> String {
    "Catppuccin Mocha".to_string()
}
fn default_icon_pack() -> String {
    "catppuccin".to_string()
}
fn default_accent() -> String {
    "#89b4fa".to_string()
}
fn default_sidebar_width() -> u16 {
    20
}
fn default_preview_width() -> u16 {
    75
}
fn default_terminal_height() -> u16 {
    8
}
fn default_monitor_height() -> u16 {
    14
}
fn default_tab_width() -> usize {
    4
}
fn default_max_preview_bytes() -> u64 {
    512 * 1024 // 512 KiB
}
fn default_scrollback_kib() -> usize {
    1024
}
fn default_explorer_ratio() -> f32 {
    0.25
}
fn default_preview_ratio() -> f32 {
    0.75
}
fn default_terminal_height_percent() -> u16 {
    15
}
fn default_monitor_interval() -> u64 {
    1000
}

// ── General Configuration ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneralConfig {
    #[serde(default = "default_false")]
    pub first_run_completed: bool,
    #[serde(default = "default_true")]
    pub confirm_delete: bool,
    #[serde(default = "default_true")]
    pub restore_session: bool,
    #[serde(default = "default_true")]
    pub watch_filesystem: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            first_run_completed: false,
            confirm_delete: true,
            restore_session: true,
            watch_filesystem: true,
        }
    }
}

// ── Shortcut configuration ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeybindingsConfig {
    #[serde(default = "default_true")]
    pub use_defaults: bool,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self { use_defaults: true }
    }
}

// ── LSP server configuration ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LspConfig {
    /// rust-analyzer binary path (default: search PATH)
    pub rust_analyzer: Option<String>,
    /// clangd binary path
    pub clangd: Option<String>,
    /// pyright-langserver or similar Python LSP
    pub pyright: Option<String>,
    /// typescript-language-server
    pub typescript_ls: Option<String>,
}

// ── Editor configuration ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditorConfig {
    #[serde(default = "default_tab_width")]
    pub tab_width: usize,
    #[serde(default = "default_true")]
    pub auto_pair: bool,
    #[serde(default = "default_true")]
    pub line_numbers: bool,
    #[serde(default = "default_true")]
    pub syntax_highlight: bool,
    #[serde(default = "default_true")]
    pub word_wrap: bool,
    #[serde(default = "default_true")]
    pub show_completion: bool,
    #[serde(default)]
    pub lsp: LspConfig,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: default_tab_width(),
            auto_pair: true,
            line_numbers: true,
            syntax_highlight: true,
            word_wrap: true,
            show_completion: true,
            lsp: LspConfig::default(),
        }
    }
}

// ── Terminal configuration ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalConfig {
    pub default_shell: Option<String>,
    #[serde(default = "default_terminal_height")]
    pub terminal_height: u16,
    #[serde(default = "default_scrollback_kib")]
    pub scrollback_limit_kib: usize,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            default_shell: None,
            terminal_height: default_terminal_height(),
            scrollback_limit_kib: default_scrollback_kib(),
        }
    }
}

// ── Search configuration ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchConfig {
    #[serde(default = "default_false")]
    pub case_sensitive: bool,
    #[serde(default = "default_true")]
    pub respect_ignore: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            respect_ignore: true,
        }
    }
}

// ── Developer Tools configuration ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeveloperConfig {
    #[serde(default = "default_true")]
    pub git_status_enabled: bool,
    #[serde(default = "default_true")]
    pub task_runner_enabled: bool,
}

impl Default for DeveloperConfig {
    fn default() -> Self {
        Self {
            git_status_enabled: true,
            task_runner_enabled: true,
        }
    }
}

// ── Theme configuration ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ThemeConfig {
    #[serde(default = "default_theme")]
    pub name: String,
    #[serde(default = "default_accent")]
    pub accent: String,
    /// Use terminal default background (transparent-friendly).
    #[serde(default = "default_true")]
    pub transparent_background: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: default_theme(),
            accent: default_accent(),
            transparent_background: true,
        }
    }
}

// ── Pinned folder entry ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PinnedFolder {
    pub name: String,
    pub path: PathBuf,
}

// ── Custom sidebar shortcut ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomShortcut {
    pub name: String,
    pub path: PathBuf,
}

// ── Monitor configuration ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonitorConfig {
    /// Refresh interval in milliseconds.
    #[serde(default = "default_monitor_interval")]
    pub refresh_ms: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self { refresh_ms: 1000 }
    }
}

// ── Layout configuration ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutSettings {
    #[serde(default = "default_explorer_ratio")]
    pub explorer_ratio: f32,
    #[serde(default = "default_preview_ratio")]
    pub preview_ratio: f32,
    #[serde(default = "default_terminal_height_percent")]
    pub terminal_height_percent: u16,
    #[serde(default = "default_false")]
    pub show_terminal: bool,
    #[serde(default = "default_true")]
    pub show_preview: bool,
    #[serde(default = "default_false")]
    pub show_sidebar: bool,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            explorer_ratio: 0.25,
            preview_ratio: 0.75,
            terminal_height_percent: 15,
            show_terminal: false,
            show_preview: true,
            show_sidebar: false,
        }
    }
}

impl LayoutSettings {
    /// Return the file explorer width percentage as an integer (e.g. 25).
    pub fn explorer_percent(&self) -> u16 {
        (self.explorer_ratio * 100.0).round() as u16
    }

    /// Return the preview panel width percentage as an integer (e.g. 75).
    pub fn preview_percent(&self) -> u16 {
        (self.preview_ratio * 100.0).round() as u16
    }

    /// Set file explorer ratio percentage (15%..50%) and automatically derive preview ratio.
    pub fn set_explorer_percent(&mut self, pct: u16) {
        let clamped_exp = pct.clamp(15, 50);
        self.explorer_ratio = clamped_exp as f32 / 100.0;
        self.preview_ratio = (100 - clamped_exp) as f32 / 100.0;
    }

    /// Set preview panel ratio percentage (50%..85%) and automatically derive explorer ratio.
    pub fn set_preview_percent(&mut self, pct: u16) {
        let clamped_exp = (100u16.saturating_sub(pct)).clamp(15, 50);
        self.explorer_ratio = clamped_exp as f32 / 100.0;
        self.preview_ratio = (100 - clamped_exp) as f32 / 100.0;
    }
}

// ── Root configuration struct ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Configuration Schema Version (Version 2).
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// General application and first-run settings.
    #[serde(default)]
    pub general: GeneralConfig,

    /// Layout proportions and pane visibility.
    #[serde(default)]
    pub layout: LayoutSettings,

    /// Color theme and visual accents.
    #[serde(default)]
    pub theme: ThemeConfig,

    /// Integrated code editor settings.
    #[serde(default)]
    pub editor: EditorConfig,

    /// Embedded PTY terminal settings.
    #[serde(default)]
    pub terminal: TerminalConfig,

    /// Unified project search settings.
    #[serde(default)]
    pub search: SearchConfig,

    /// Developer workbench tools.
    #[serde(default)]
    pub developer: DeveloperConfig,

    /// System resource monitor settings.
    #[serde(default)]
    pub monitor: MonitorConfig,

    /// Keyboard shortcuts settings.
    #[serde(default)]
    pub keybindings: KeybindingsConfig,

    /// Pinned folders persisted across sessions.
    #[serde(default)]
    pub pinned_folders: Vec<PinnedFolder>,

    #[serde(default)]
    pub custom_shortcuts: Vec<CustomShortcut>,

    // Backward-compatible panel visibility defaults.
    #[serde(default = "default_false")]
    pub show_sidebar: bool,
    #[serde(default = "default_true")]
    pub show_preview: bool,
    #[serde(default = "default_false")]
    pub show_terminal: bool,
    #[serde(default = "default_false")]
    pub show_monitor: bool,

    // Backward-compatible panel dimensions.
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u16,
    #[serde(default = "default_preview_width")]
    pub preview_width: u16,
    #[serde(default = "default_terminal_height")]
    pub terminal_height: u16,
    #[serde(default = "default_monitor_height")]
    pub monitor_height: u16,

    // Show hidden files by default?
    #[serde(default = "default_false")]
    pub show_hidden: bool,

    // Max bytes to load for file preview.
    #[serde(default = "default_max_preview_bytes")]
    pub max_preview_bytes: u64,

    #[serde(default = "default_icon_pack")]
    pub icon_pack: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            general: GeneralConfig::default(),
            layout: LayoutSettings::default(),
            theme: ThemeConfig::default(),
            editor: EditorConfig::default(),
            terminal: TerminalConfig::default(),
            search: SearchConfig::default(),
            developer: DeveloperConfig::default(),
            monitor: MonitorConfig::default(),
            keybindings: KeybindingsConfig::default(),
            pinned_folders: Vec::new(),
            custom_shortcuts: Vec::new(),
            show_sidebar: false,
            show_preview: true,
            show_terminal: false,
            show_monitor: false,
            sidebar_width: default_sidebar_width(),
            preview_width: default_preview_width(),
            terminal_height: default_terminal_height(),
            monitor_height: default_monitor_height(),
            show_hidden: false,
            max_preview_bytes: default_max_preview_bytes(),
            icon_pack: default_icon_pack(),
        }
    }
}

impl Config {
    /// Return the XDG config directory for TermExplorer.
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("termexplorer")
    }

    /// Return the path to the config file.
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Return the path to the backup config file.
    pub fn backup_path() -> PathBuf {
        Self::config_dir().join("config.toml.bak")
    }

    /// Set file explorer ratio percentage (15%..50%) and automatically derive preview ratio.
    pub fn set_explorer_percent(&mut self, pct: u16) {
        self.layout.set_explorer_percent(pct);
        self.preview_width = self.layout.preview_percent();
        self.validate_and_clamp();
    }

    /// Set preview panel ratio percentage (50%..85%) and automatically derive explorer ratio.
    pub fn set_preview_percent(&mut self, pct: u16) {
        self.layout.set_preview_percent(pct);
        self.preview_width = self.layout.preview_percent();
        self.validate_and_clamp();
    }

    /// Validate and clamp layout ratios, dimensions, and theme settings to valid ranges.
    pub fn validate_and_clamp(&mut self) {
        // Enforce approved 2-pane workstation limits: Explorer 15%..50%, Preview 50%..85%
        // Guarantee horizontal ratios always sum to 1.0 (100%)
        let exp = if self.layout.explorer_ratio.is_nan() || self.layout.explorer_ratio < 0.15 {
            0.15
        } else if self.layout.explorer_ratio > 0.50 {
            0.50
        } else {
            self.layout.explorer_ratio
        };
        let exp_pct = (exp * 100.0).round() as u16;
        let clamped_exp = exp_pct.clamp(15, 50);
        self.layout.explorer_ratio = clamped_exp as f32 / 100.0;
        self.layout.preview_ratio = (100 - clamped_exp) as f32 / 100.0;

        // Terminal drawer height: 10%..40% (default 15%)
        if self.layout.terminal_height_percent == 0 {
            self.layout.terminal_height_percent = 15;
        } else {
            self.layout.terminal_height_percent = self.layout.terminal_height_percent.clamp(10, 40);
        }

        // Clamp preview_width to valid bounds
        self.preview_width = self.preview_width.clamp(15, 85);

        // Tab width: 2, 4, or 8
        if self.editor.tab_width != 2 && self.editor.tab_width != 4 && self.editor.tab_width != 8 {
            self.editor.tab_width = 4;
        }

        // Preview bytes: minimum 64 KiB, maximum 64 MiB
        self.max_preview_bytes = self.max_preview_bytes.clamp(64 * 1024, 64 * 1024 * 1024);

        // Synchronize top-level and layout fields
        if self.layout.show_terminal {
            self.show_terminal = true;
        } else if !self.show_terminal {
            self.layout.show_terminal = false;
        }
        if self.layout.show_sidebar {
            self.show_sidebar = true;
        } else if !self.show_sidebar {
            self.layout.show_sidebar = false;
        }

        // Canonical theme normalization
        let current_theme = self.theme.name.to_lowercase();
        if current_theme == "default dark"
            || current_theme == "default_dark"
            || current_theme.is_empty()
        {
            self.theme.name = "Catppuccin Mocha".to_string();
        }
        if self.icon_pack.is_empty() || self.icon_pack == "default" {
            self.icon_pack = "catppuccin".to_string();
        }
    }

    /// Load configuration, applying migrations and recovering automatically from backups if needed.
    pub fn load_or_create() -> Result<Self> {
        let path = Self::config_path();
        let config_dir = Self::config_dir();

        if path.exists() {
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!(
                        "Failed to read config file {}: {}. Attempting backup.",
                        path.display(),
                        e
                    );
                    return Self::recover_from_backup_or_default();
                }
            };

            // Run migration pipeline
            match ConfigMigrator::migrate(&content, Some(&config_dir)) {
                Ok((config, report)) => {
                    if report.migrated {
                        tracing::info!(
                            "Migrated configuration from v{} to v{}. Changes: {:?}",
                            report.initial_version,
                            report.final_version,
                            report.changes
                        );
                        let _ = config.save();
                    }
                    Ok(config)
                }
                Err(parse_err) => {
                    tracing::warn!(
                        "Configuration file {} is corrupted or incompatible: {}. Initiating recovery sequence.",
                        path.display(),
                        parse_err
                    );
                    Self::recover_from_backup_or_default()
                }
            }
        } else {
            let mut config = Self::default();
            config.validate_and_clamp();
            config.save()?;
            Ok(config)
        }
    }

    /// Attempt recovery from config.toml.bak, or archive corrupted file and return pristine defaults.
    fn recover_from_backup_or_default() -> Result<Self> {
        let bak_path = Self::backup_path();
        if bak_path.exists() {
            if let Ok(bak_content) = std::fs::read_to_string(&bak_path) {
                if let Ok((config, _)) = ConfigMigrator::migrate(&bak_content, None) {
                    tracing::info!(
                        "Successfully recovered configuration from {}",
                        bak_path.display()
                    );
                    let primary = Self::config_path();
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let archive_path =
                        Self::config_dir().join(format!("config.toml.corrupted.{}", timestamp));
                    let _ = std::fs::rename(&primary, &archive_path);
                    let _ = config.save();
                    return Ok(config);
                }
            }
        }

        // Archive corrupted file and generate pristine defaults
        let primary = Self::config_path();
        if primary.exists() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let archive_path =
                Self::config_dir().join(format!("config.toml.corrupted.{}", timestamp));
            let _ = std::fs::rename(&primary, &archive_path);
        }

        let mut config = Self::default();
        config.validate_and_clamp();
        config.save()?;
        Ok(config)
    }

    /// Persist the current configuration to disk atomically with automatic backup.
    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Creating config dir: {}", dir.display()))?;

        let path = Self::config_path();
        let bak_path = Self::backup_path();

        // 1. Create automatic backup of previous valid configuration
        if path.exists() {
            let _ = std::fs::copy(&path, &bak_path);
        }

        // 2. Serialize to TOML
        let content = toml::to_string_pretty(self).context("Serialising configuration to TOML")?;

        // 3. Atomic write using NamedTempFile in same directory
        crate::filesystem::operations::save_file_atomic(&path, content.as_bytes())
            .with_context(|| format!("Writing config atomically: {}", path.display()))?;

        Ok(())
    }

    /// Reset configuration to default values and persist atomically.
    pub fn reset_to_default(&mut self) -> Result<()> {
        *self = Self::default();
        self.save()
    }

    /// Export configuration to a specified path.
    pub fn export_to_file(&self, target_path: &Path) -> Result<()> {
        let content =
            toml::to_string_pretty(self).context("Serialising configuration for export")?;
        crate::filesystem::operations::save_file_atomic(target_path, content.as_bytes())
    }

    /// Import configuration from a specified path after full schema validation.
    pub fn import_from_file(source_path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(source_path)
            .with_context(|| format!("Reading import file: {}", source_path.display()))?;
        let (mut config, _) = ConfigMigrator::migrate(&content, None).with_context(|| {
            format!("Validating configuration schema: {}", source_path.display())
        })?;
        config.validate_and_clamp();
        config.save()?;
        Ok(config)
    }

    /// Add a pinned folder (no-op if already pinned at that path).
    pub fn pin_folder(&mut self, name: String, path: PathBuf) {
        if !self.pinned_folders.iter().any(|p| p.path == path) {
            self.pinned_folders.push(PinnedFolder { name, path });
        }
    }

    /// Remove a pinned folder by index.
    pub fn unpin_folder(&mut self, index: usize) {
        if index < self.pinned_folders.len() {
            self.pinned_folders.remove(index);
        }
    }
}
