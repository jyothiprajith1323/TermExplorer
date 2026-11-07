//! Interactive Settings Studio subsystem.

pub mod keybindings;
pub mod persistence;
pub mod profiles;
pub mod schema;

use crate::config::Config;
pub use keybindings::{BindingScope, ConflictReport, KeyActionBinding, KeybindingManager};
pub use persistence::SettingsPersistence;
pub use profiles::ProfileKind;
pub use schema::{SettingItem, SettingValue, SettingsSection};

/// Generate the list of editable items for a given settings section based on current config.
pub fn get_section_items(section: SettingsSection, config: &Config) -> Vec<SettingItem> {
    match section {
        SettingsSection::General => vec![
            SettingItem {
                key: "confirm_delete",
                label: "Confirm Destructive Actions",
                description: "Display confirmation prompt before deleting files or directories",
                value: SettingValue::Bool(true),
            },
            SettingItem {
                key: "restore_session",
                label: "Restore Last Workspace Session",
                description: "Reopen previous active directory and open editor buffers on startup",
                value: SettingValue::Bool(true),
            },
            SettingItem {
                key: "watch_filesystem",
                label: "Live Filesystem Monitoring (inotify)",
                description: "Automatically refresh directory view when files are created or modified externally",
                value: SettingValue::Bool(true),
            },
        ],
        SettingsSection::Appearance => vec![
            SettingItem {
                key: "ui_density",
                label: "UI Visual Density",
                description: "Clean compact layout optimized for high information density",
                value: SettingValue::ReadOnly("Compact (Developer Workstation)".to_string()),
            },
            SettingItem {
                key: "borders",
                label: "Panel Border Styling",
                description: "Rounded double borders with active focus highlights",
                value: SettingValue::ReadOnly("Rounded / High Contrast Focus".to_string()),
            },
            SettingItem {
                key: "show_git_status",
                label: "Git Repository Badges",
                description: "Display branch badge and file modification indicators in header and explorer",
                value: SettingValue::Bool(true),
            },
        ],
        SettingsSection::Themes => {
            let themes: Vec<String> = crate::theme::Theme::list_themes()
                .iter()
                .map(|&(name, _)| name.to_string())
                .collect();
            let current_idx = themes
                .iter()
                .position(|t| t.eq_ignore_ascii_case(&config.theme.name))
                .unwrap_or(0);

            vec![
                SettingItem {
                    key: "theme",
                    label: "Color Theme",
                    description: "Select one of 14 canonical themes (Catppuccin Mocha/Macchiato/Frappé/Latte, Dracula, etc.)",
                    value: SettingValue::Choice {
                        current: current_idx,
                        options: themes,
                    },
                },
                SettingItem {
                    key: "transparent_bg",
                    label: "Inherit Terminal Transparency",
                    description: "Use terminal default background color (bg: None) instead of opaque background",
                    value: SettingValue::Bool(config.theme.transparent_background),
                },
                SettingItem {
                    key: "accent_color",
                    label: "Accent Color Hex",
                    description: "Primary accent color used for headers, focus borders, and active highlights",
                    value: SettingValue::Text(config.theme.accent.clone()),
                },
                SettingItem {
                    key: "custom_themes_dir",
                    label: "Custom Themes Path",
                    description: "Directory scanned for user JSON/TOML theme definitions",
                    value: SettingValue::ReadOnly("~/.config/termexplorer/themes/".to_string()),
                },
            ]
        }
        SettingsSection::Fonts => {
            let font_status = crate::fonts::FontStatus::detect();
            vec![
                SettingItem {
                    key: "font_status",
                    label: "Host Font Detection",
                    description: "Status of JetBrains Mono font detection on the host system via fontconfig",
                    value: SettingValue::ReadOnly(font_status.status_summary()),
                },
                SettingItem {
                    key: "font_guide",
                    label: "Terminal Font Configuration",
                    description: "Guide for configuring Kitty, Alacritty, WezTerm, or GNOME Terminal with JetBrains Mono",
                    value: SettingValue::ReadOnly(
                        "Set emulator font to 'JetBrainsMono Nerd Font Mono 12'. Run installer or check docs/FONTS.md.".to_string(),
                    ),
                },
                SettingItem {
                    key: "font_installer",
                    label: "Automated Font Installer",
                    description: "Run scripts/install-fonts.sh to download and cache latest JetBrains Mono Nerd Font",
                    value: SettingValue::ReadOnly("scripts/install-fonts.sh (Available)".to_string()),
                },
            ]
        }
        SettingsSection::FileIcons => {
            let icon_packs: Vec<String> = crate::icons::IconPack::all()
                .iter()
                .map(|p| p.display_name().to_string())
                .collect();
            let current_pack = crate::icons::IconPack::parse(&config.icon_pack);
            let pack_idx = crate::icons::IconPack::all()
                .iter()
                .position(|&p| p == current_pack)
                .unwrap_or(0);

            vec![
                SettingItem {
                    key: "icon_pack",
                    label: "File & Folder Icon Pack",
                    description: "Icon styling: Catppuccin Nerd Font, Monochrome Nerd Font, Unicode, ASCII, or None",
                    value: SettingValue::Choice {
                        current: pack_idx,
                        options: icon_packs,
                    },
                },
                SettingItem {
                    key: "custom_icons_path",
                    label: "Custom Icon Rules Path",
                    description: "Configuration file for per-extension and directory icon glyph overrides",
                    value: SettingValue::ReadOnly("~/.config/termexplorer/icons.toml".to_string()),
                },
            ]
        }
        SettingsSection::Layout => vec![
            SettingItem {
                key: "workspace_layout",
                label: "Workspace Layout Architecture",
                description: "Two-Column Workstation: Left 70% File Explorer / Right 30% Contextual Preview",
                value: SettingValue::ReadOnly("Two-Column (70% Explorer / 30% Preview)".to_string()),
            },
            SettingItem {
                key: "preview_width",
                label: "Preview Width (%)",
                description: "Percentage width allocated to the universal preview panel (default 30%)",
                value: SettingValue::Number {
                    value: config.preview_width as i64,
                    min: 15,
                    max: 60,
                    step: 5,
                },
            },
            SettingItem {
                key: "terminal_height",
                label: "Terminal Default Height (Rows)",
                description: "Compact embedded PTY terminal height (~20-22% height, 6-8 content rows)",
                value: SettingValue::Number {
                    value: config.terminal_height as i64,
                    min: 6,
                    max: 20,
                    step: 1,
                },
            },
            SettingItem {
                key: "monitor_height",
                label: "System Monitor Height (Rows)",
                description: "Height allocated to the telemetry dashboard when toggled (F6)",
                value: SettingValue::Number {
                    value: config.monitor_height as i64,
                    min: 8,
                    max: 35,
                    step: 2,
                },
            },
        ],
        SettingsSection::Keybindings => {
            let defaults = KeybindingManager::default_bindings();
            let report = KeybindingManager::check_conflicts(&defaults);
            vec![
                SettingItem {
                    key: "conflict_status",
                    label: "Shortcut Conflict Status",
                    description: if report.has_conflicts { "Conflicts detected" } else { "Clean (0 conflicts across all scopes)" },
                    value: SettingValue::ReadOnly(if report.has_conflicts { "Conflicts Present".to_string() } else { "No Conflicts Detected".to_string() }),
                },
                SettingItem {
                    key: "total_bindings",
                    label: "Registered Actions",
                    description: "Central keybinding registry coverage across Explorer, Editor, and Terminal",
                    value: SettingValue::ReadOnly(format!("{} keyboard shortcuts configured", defaults.len())),
                },
            ]
        }
        SettingsSection::FileExplorer => vec![
            SettingItem {
                key: "show_hidden",
                label: "Show Hidden Files by Default",
                description: "Display dotfiles and hidden directories upon opening a directory",
                value: SettingValue::Bool(config.show_hidden),
            },
            SettingItem {
                key: "trash_integration",
                label: "Use FreeDesktop Trash by Default",
                description: "Move deleted items to Trash instead of immediate permanent deletion",
                value: SettingValue::Bool(true),
            },
        ],
        SettingsSection::Preview => vec![
            SettingItem {
                key: "max_preview_bytes",
                label: "Max Preview Size (KiB)",
                description: "Maximum file size loaded into text and source preview before truncation",
                value: SettingValue::Number {
                    value: (config.max_preview_bytes / 1024) as i64,
                    min: 64,
                    max: 8192,
                    step: 128,
                },
            },
            SettingItem {
                key: "halfblock_rendering",
                label: "Half-Block Pixel Rendering",
                description: "Render images and video thumbnails in truecolor 24-bit halfblocks",
                value: SettingValue::Bool(true),
            },
        ],
        SettingsSection::Editor => vec![
            SettingItem {
                key: "tab_width",
                label: "Indentation Tab Width",
                description: "Number of spaces per indentation level in code editor",
                value: SettingValue::Number {
                    value: config.editor.tab_width as i64,
                    min: 2,
                    max: 8,
                    step: 2,
                },
            },
            SettingItem {
                key: "auto_pair",
                label: "Auto-Pair Brackets and Quotes",
                description: "Automatically insert closing brackets: (), [], {}, \"\", '', ``",
                value: SettingValue::Bool(config.editor.auto_pair),
            },
            SettingItem {
                key: "line_numbers",
                label: "Display Line Numbers",
                description: "Show line numbers on left gutter in editor view",
                value: SettingValue::Bool(config.editor.line_numbers),
            },
            SettingItem {
                key: "show_completion",
                label: "Word & Symbol Autocompletion Popup",
                description: "Show floating autocompletion menu when typing identifiers",
                value: SettingValue::Bool(config.editor.show_completion),
            },
            SettingItem {
                key: "lsp_rust_analyzer",
                label: "Rust LSP (rust-analyzer)",
                description: "Status of rust-analyzer binary in PATH for Rust code intelligence",
                value: SettingValue::ReadOnly(
                    if crate::preview::CapabilityRegistry::is_tool_available("rust-analyzer") {
                        "Installed & Detected".to_string()
                    } else {
                        "Not Found in $PATH".to_string()
                    },
                ),
            },
        ],
        SettingsSection::Terminal => vec![
            SettingItem {
                key: "default_shell",
                label: "Default Shell",
                description: "Interactive shell launched inside embedded pseudo-terminal ($SHELL)",
                value: SettingValue::ReadOnly(
                    std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
                ),
            },
            SettingItem {
                key: "terminal_height",
                label: "Default Height (Rows)",
                description: "Compact terminal panel height (default: 8 rows, ~20-22% height)",
                value: SettingValue::Number {
                    value: config.terminal_height as i64,
                    min: 5,
                    max: 25,
                    step: 1,
                },
            },
            SettingItem {
                key: "scrollback_limit",
                label: "Scrollback Memory Limit (KiB)",
                description: "Maximum buffer size retained for terminal scrollback history",
                value: SettingValue::Number {
                    value: 1024,
                    min: 256,
                    max: 16384,
                    step: 512,
                },
            },
        ],
        SettingsSection::SystemMonitor => vec![
            SettingItem {
                key: "refresh_ms",
                label: "Telemetry Refresh Interval (ms)",
                description: "Interval in milliseconds between CPU, RAM, and process table updates",
                value: SettingValue::Number {
                    value: config.monitor.refresh_ms as i64,
                    min: 250,
                    max: 5000,
                    step: 250,
                },
            },
        ],
        SettingsSection::Plugins => vec![
            SettingItem {
                key: "plugin_marketplace",
                label: "Open Plugin Marketplace",
                description: "Browse, install, and manage extensions (Shortcut: Ctrl+Shift+X)",
                value: SettingValue::ReadOnly("Press [Enter] or Ctrl+Shift+X to open".to_string()),
            },
            SettingItem {
                key: "bundled_plugins",
                label: "Bundled Extensions",
                description: "Core plugins included with TermExplorer",
                value: SettingValue::ReadOnly("Workspace Tools, File Inspector".to_string()),
            },
            SettingItem {
                key: "plugin_sandbox",
                label: "Plugin Security Sandbox",
                description: "Isolated WebAssembly (WASI) runtime with capability grants and fuel limits",
                value: SettingValue::ReadOnly("Active (Wasmtime Fuel Enforced)".to_string()),
            },
        ],
        SettingsSection::Advanced => vec![
            SettingItem {
                key: "log_level",
                label: "Diagnostic Log Level",
                description: "Severity filter for rolling logs in ~/.local/share/termexplorer/termexplorer.log",
                value: SettingValue::Choice {
                    current: 0,
                    options: vec!["INFO".into(), "DEBUG".into(), "WARN".into(), "ERROR".into()],
                },
            },
            SettingItem {
                key: "safe_mode",
                label: "Safe Mode",
                description: "Disable experimental preview decoders and enforce strict resource bounds",
                value: SettingValue::Bool(false),
            },
        ],
    }
}
