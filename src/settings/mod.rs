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
                value: SettingValue::Bool(config.general.confirm_delete),
            },
            SettingItem {
                key: "restore_session",
                label: "Restore Last Workspace Session",
                description: "Reopen previous active directory and open editor buffers on startup",
                value: SettingValue::Bool(config.general.restore_session),
            },
            SettingItem {
                key: "watch_filesystem",
                label: "Live Filesystem Monitoring (inotify)",
                description: "Automatically refresh directory view when files are created or modified externally",
                value: SettingValue::Bool(config.general.watch_filesystem),
            },
            SettingItem {
                key: "run_onboarding",
                label: "Replay Onboarding Setup Wizard",
                description: "Press [Enter] to run the first-launch setup wizard again",
                value: SettingValue::ReadOnly("Press [Enter] to Launch".to_string()),
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
                value: SettingValue::Bool(config.developer.git_status_enabled),
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
                    description: "Select one of 14 canonical themes (Catppuccin Mocha is default)",
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
                    description: "Terminal emulators control the actual rendered font. Recommended: 'JetBrainsMono Nerd Font'",
                    value: SettingValue::ReadOnly(
                        "Set emulator font to 'JetBrainsMono Nerd Font'. See docs/WSL2_SETUP.md.".to_string(),
                    ),
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
                key: "explorer_ratio",
                label: "File Explorer Ratio (%)",
                description: "Width percentage allocated to File Explorer (default 25%, min 15%, max 50%)",
                value: SettingValue::Number {
                    value: (config.layout.explorer_ratio * 100.0).round() as i64,
                    min: 15,
                    max: 50,
                    step: 5,
                },
            },
            SettingItem {
                key: "preview_ratio",
                label: "Preview / Inspector Ratio (%)",
                description: "Width percentage allocated to Universal Previewer (default 75%, min 50%, max 85%)",
                value: SettingValue::Number {
                    value: (config.layout.preview_ratio * 100.0).round() as i64,
                    min: 50,
                    max: 85,
                    step: 5,
                },
            },
            SettingItem {
                key: "terminal_height_percent",
                label: "Terminal Drawer Height (%)",
                description: "Height allocated to collapsible PTY terminal drawer when opened (default: 15%)",
                value: SettingValue::Number {
                    value: config.layout.terminal_height_percent as i64,
                    min: 10,
                    max: 40,
                    step: 5,
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
                SettingItem {
                    key: "view_shortcuts",
                    label: "View All Keyboard Shortcuts",
                    description: "Press [Enter] to open the complete keyboard shortcut cheat-sheet (F1)",
                    value: SettingValue::ReadOnly("Press [Enter] to View".to_string()),
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
                description: "Move deleted items to Trash (~/.local/share/Trash) instead of permanent deletion",
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
                    max: 16384,
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
                description: "Number of spaces per indentation level in code editor (2, 4, or 8)",
                value: SettingValue::Choice {
                    current: match config.editor.tab_width {
                        2 => 0,
                        8 => 2,
                        _ => 1,
                    },
                    options: vec!["2 spaces".into(), "4 spaces".into(), "8 spaces".into()],
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
                key: "syntax_highlight",
                label: "Syntax Highlighting",
                description: "Colorize source code using syntect Sublime syntax definitions",
                value: SettingValue::Bool(config.editor.syntax_highlight),
            },
            SettingItem {
                key: "word_wrap",
                label: "Word Wrap",
                description: "Soft wrap long source code lines in the editor viewport",
                value: SettingValue::Bool(config.editor.word_wrap),
            },
            SettingItem {
                key: "show_completion",
                label: "Word & Symbol Autocompletion Popup",
                description: "Show floating autocompletion menu when typing identifiers",
                value: SettingValue::Bool(config.editor.show_completion),
            },
        ],
        SettingsSection::Terminal => {
            let caps = crate::shell::TerminalCapabilities::detect();
            let shells: Vec<String> = caps.available_shells.iter().map(|s| s.path.display().to_string()).collect();
            let current_shell = config.terminal.default_shell.clone().unwrap_or_else(|| caps.default_shell.path.display().to_string());
            let shell_idx = shells.iter().position(|s| s == &current_shell).unwrap_or(0);

            vec![
                SettingItem {
                    key: "default_shell",
                    label: "Default Shell",
                    description: "Interactive shell launched inside embedded pseudo-terminal ($SHELL)",
                    value: if shells.is_empty() {
                        SettingValue::ReadOnly(current_shell)
                    } else {
                        SettingValue::Choice {
                            current: shell_idx,
                            options: shells,
                        }
                    },
                },
                SettingItem {
                    key: "pty_status",
                    label: "PTY Subsystem Status",
                    description: "Unix pseudo-terminal allocation via /dev/ptmx",
                    value: SettingValue::ReadOnly(if caps.pty_available { "Available (/dev/ptmx)".to_string() } else { "Unavailable".to_string() }),
                },
                SettingItem {
                    key: "scrollback_limit",
                    label: "Scrollback Memory Limit (KiB)",
                    description: "Maximum buffer size retained for terminal scrollback history",
                    value: SettingValue::Number {
                        value: config.terminal.scrollback_limit_kib as i64,
                        min: 256,
                        max: 16384,
                        step: 512,
                    },
                },
            ]
        }
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
                key: "plugin_sandbox",
                label: "Plugin Security Sandbox",
                description: "Isolated WebAssembly (WASI) runtime with capability grants and fuel limits",
                value: SettingValue::ReadOnly("Active (wasmi Memory Isolation Enforced)".to_string()),
            },
        ],
        SettingsSection::Advanced => vec![
            SettingItem {
                key: "schema_version",
                label: "Configuration Schema Version",
                description: "Active configuration schema version with automatic migration support",
                value: SettingValue::ReadOnly(format!("Schema Version {} (Current)", config.schema_version)),
            },
            SettingItem {
                key: "export_config",
                label: "Export Settings",
                description: "Export current settings to ~/.config/termexplorer/config_export.toml",
                value: SettingValue::ReadOnly("Press [e] to Export".to_string()),
            },
            SettingItem {
                key: "import_config",
                label: "Import Settings",
                description: "Import settings from ~/.config/termexplorer/config_export.toml",
                value: SettingValue::ReadOnly("Press [i] to Import".to_string()),
            },
            SettingItem {
                key: "reset_all",
                label: "Reset All Settings to Defaults",
                description: "Reset all preferences to pristine default values (preserves user files)",
                value: SettingValue::ReadOnly("Press [R] to Reset All".to_string()),
            },
        ],
    }
}
