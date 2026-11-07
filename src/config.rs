//! Application configuration — loaded from and persisted to TOML.
//! Stored at `~/.config/termexplorer/config.toml`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Default helpers ──────────────────────────────────────────────────────────

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
    "#61AFEF".to_string()
}
fn default_sidebar_width() -> u16 {
    20
}
fn default_preview_width() -> u16 {
    30
}
fn default_terminal_height() -> u16 {
    12
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

// ── Shortcut configuration ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

// ── Theme configuration ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomShortcut {
    pub name: String,
    pub path: PathBuf,
}

// ── Monitor configuration ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Refresh interval in milliseconds.
    #[serde(default = "default_monitor_interval")]
    pub refresh_ms: u64,
}

fn default_monitor_interval() -> u64 {
    1000
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self { refresh_ms: 1000 }
    }
}

// ── Root configuration struct ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Pinned folders persisted across sessions.
    #[serde(default)]
    pub pinned_folders: Vec<PinnedFolder>,

    #[serde(default)]
    pub custom_shortcuts: Vec<CustomShortcut>,

    // Panel visibility defaults (Workspace sidebar removed by default; Terminal shown by default).
    #[serde(default = "default_false")]
    pub show_sidebar: bool,
    #[serde(default = "default_true")]
    pub show_preview: bool,
    #[serde(default = "default_true")]
    pub show_terminal: bool,
    #[serde(default = "default_false")]
    pub show_monitor: bool,

    // Panel dimensions.
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

    #[serde(default)]
    pub theme: ThemeConfig,

    #[serde(default)]
    pub editor: EditorConfig,

    #[serde(default)]
    pub monitor: MonitorConfig,

    #[serde(default)]
    pub keybindings: KeybindingsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pinned_folders: Vec::new(),
            custom_shortcuts: Vec::new(),
            show_sidebar: false,
            show_preview: true,
            show_terminal: true,
            show_monitor: false,
            sidebar_width: default_sidebar_width(),
            preview_width: default_preview_width(),
            terminal_height: default_terminal_height(),
            monitor_height: default_monitor_height(),
            show_hidden: false,
            max_preview_bytes: default_max_preview_bytes(),
            icon_pack: default_icon_pack(),
            theme: ThemeConfig::default(),
            editor: EditorConfig::default(),
            monitor: MonitorConfig::default(),
            keybindings: KeybindingsConfig::default(),
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

    /// Load configuration, creating a default if it does not exist.
    pub fn load_or_create() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Reading config: {}", path.display()))?;
            let config: Self = toml::from_str(&content)
                .with_context(|| format!("Parsing config: {}", path.display()))?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Persist the current configuration to disk.
    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Creating config dir: {}", dir.display()))?;
        let content = toml::to_string_pretty(self).context("Serialising configuration to TOML")?;
        let path = Self::config_path();
        std::fs::write(&path, content)
            .with_context(|| format!("Writing config: {}", path.display()))?;
        Ok(())
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
