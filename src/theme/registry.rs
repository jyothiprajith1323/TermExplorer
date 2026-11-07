//! Theme registry, loader, and custom theme TOML parser.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::palette::{parse_hex, CatppuccinPalette};
use super::tokens::ThemeTokens;
use super::Theme;

/// Serializable format for a custom theme definition in TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomThemeDef {
    pub theme: ThemeMeta,
    pub colors: Option<CustomColorsDef>,
    pub palette: Option<CustomPaletteDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMeta {
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub dark: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomColorsDef {
    pub bg: Option<String>,
    pub fg: Option<String>,
    pub accent: Option<String>,
    pub selection_bg: Option<String>,
    pub selection_fg: Option<String>,
    pub border: Option<String>,
    pub border_focused: Option<String>,
    pub status_fg: Option<String>,
    pub error: Option<String>,
    pub warning: Option<String>,
    pub success: Option<String>,
    pub dir_color: Option<String>,
    pub file_color: Option<String>,
    pub symlink_color: Option<String>,
    pub hidden_color: Option<String>,
    pub size_color: Option<String>,
    pub date_color: Option<String>,
    pub header_fg: Option<String>,
    pub header_bg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomPaletteDef {
    pub rosewater: Option<String>,
    pub flamingo: Option<String>,
    pub pink: Option<String>,
    pub mauve: Option<String>,
    pub red: Option<String>,
    pub maroon: Option<String>,
    pub peach: Option<String>,
    pub yellow: Option<String>,
    pub green: Option<String>,
    pub teal: Option<String>,
    pub sky: Option<String>,
    pub sapphire: Option<String>,
    pub blue: Option<String>,
    pub lavender: Option<String>,
    pub text: Option<String>,
    pub subtext1: Option<String>,
    pub subtext0: Option<String>,
    pub overlay2: Option<String>,
    pub overlay1: Option<String>,
    pub overlay0: Option<String>,
    pub surface2: Option<String>,
    pub surface1: Option<String>,
    pub surface0: Option<String>,
    pub base: Option<String>,
    pub mantle: Option<String>,
    pub crust: Option<String>,
}

impl CustomThemeDef {
    /// Convert to a runtime `Theme`.
    pub fn to_theme(&self, transparent: bool) -> Result<Theme, String> {
        let name_leak: &'static str = Box::leak(self.theme.name.clone().into_boxed_str());
        let is_dark = self.theme.dark.unwrap_or(true);

        // If a full or partial Catppuccin palette is provided, base off mocha or latte
        let base_palette = if is_dark {
            CatppuccinPalette::mocha()
        } else {
            CatppuccinPalette::latte()
        };

        let mut pal = base_palette;
        if let Some(cp) = &self.palette {
            if let Some(c) = cp.rosewater.as_deref().and_then(parse_hex) {
                pal.rosewater = c;
            }
            if let Some(c) = cp.flamingo.as_deref().and_then(parse_hex) {
                pal.flamingo = c;
            }
            if let Some(c) = cp.pink.as_deref().and_then(parse_hex) {
                pal.pink = c;
            }
            if let Some(c) = cp.mauve.as_deref().and_then(parse_hex) {
                pal.mauve = c;
            }
            if let Some(c) = cp.red.as_deref().and_then(parse_hex) {
                pal.red = c;
            }
            if let Some(c) = cp.maroon.as_deref().and_then(parse_hex) {
                pal.maroon = c;
            }
            if let Some(c) = cp.peach.as_deref().and_then(parse_hex) {
                pal.peach = c;
            }
            if let Some(c) = cp.yellow.as_deref().and_then(parse_hex) {
                pal.yellow = c;
            }
            if let Some(c) = cp.green.as_deref().and_then(parse_hex) {
                pal.green = c;
            }
            if let Some(c) = cp.teal.as_deref().and_then(parse_hex) {
                pal.teal = c;
            }
            if let Some(c) = cp.sky.as_deref().and_then(parse_hex) {
                pal.sky = c;
            }
            if let Some(c) = cp.sapphire.as_deref().and_then(parse_hex) {
                pal.sapphire = c;
            }
            if let Some(c) = cp.blue.as_deref().and_then(parse_hex) {
                pal.blue = c;
            }
            if let Some(c) = cp.lavender.as_deref().and_then(parse_hex) {
                pal.lavender = c;
            }
            if let Some(c) = cp.text.as_deref().and_then(parse_hex) {
                pal.text = c;
            }
            if let Some(c) = cp.subtext1.as_deref().and_then(parse_hex) {
                pal.subtext1 = c;
            }
            if let Some(c) = cp.subtext0.as_deref().and_then(parse_hex) {
                pal.subtext0 = c;
            }
            if let Some(c) = cp.overlay2.as_deref().and_then(parse_hex) {
                pal.overlay2 = c;
            }
            if let Some(c) = cp.overlay1.as_deref().and_then(parse_hex) {
                pal.overlay1 = c;
            }
            if let Some(c) = cp.overlay0.as_deref().and_then(parse_hex) {
                pal.overlay0 = c;
            }
            if let Some(c) = cp.surface2.as_deref().and_then(parse_hex) {
                pal.surface2 = c;
            }
            if let Some(c) = cp.surface1.as_deref().and_then(parse_hex) {
                pal.surface1 = c;
            }
            if let Some(c) = cp.surface0.as_deref().and_then(parse_hex) {
                pal.surface0 = c;
            }
            if let Some(c) = cp.base.as_deref().and_then(parse_hex) {
                pal.base = c;
            }
            if let Some(c) = cp.mantle.as_deref().and_then(parse_hex) {
                pal.mantle = c;
            }
            if let Some(c) = cp.crust.as_deref().and_then(parse_hex) {
                pal.crust = c;
            }
        }

        let tokens = ThemeTokens::from_palette(&pal, transparent);

        // Fallback or explicit colors
        let colors = self.colors.clone().unwrap_or_default();
        let bg = if transparent {
            None
        } else {
            colors
                .bg
                .as_deref()
                .and_then(parse_hex)
                .or(tokens.bg_primary)
        };
        let fg = colors
            .fg
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.text_primary);
        let accent = colors
            .accent
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.accent_primary);
        let selection_bg = colors
            .selection_bg
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.selection_bg);
        let selection_fg = colors
            .selection_fg
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.selection_fg);
        let border = colors
            .border
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.border_normal);
        let border_focused = colors
            .border_focused
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.border_focused);
        let status_fg = colors
            .status_fg
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.status_info);
        let error = colors
            .error
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.status_error);
        let warning = colors
            .warning
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.status_warning);
        let success = colors
            .success
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.status_success);
        let dir_color = colors
            .dir_color
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.fs_directory);
        let file_color = colors
            .file_color
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.fs_file);
        let symlink_color = colors
            .symlink_color
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.fs_symlink);
        let hidden_color = colors
            .hidden_color
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.fs_hidden);
        let size_color = colors
            .size_color
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.text_muted);
        let date_color = colors
            .date_color
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.text_muted);
        let header_fg = colors
            .header_fg
            .as_deref()
            .and_then(parse_hex)
            .unwrap_or(tokens.text_primary);
        let header_bg = if transparent {
            None
        } else {
            colors
                .header_bg
                .as_deref()
                .and_then(parse_hex)
                .or(Some(tokens.bg_secondary))
        };

        Ok(Theme {
            name: name_leak,
            bg,
            fg,
            accent,
            selection_bg,
            selection_fg,
            border,
            border_focused,
            status_fg,
            error,
            warning,
            success,
            dir_color,
            file_color,
            symlink_color,
            hidden_color,
            size_color,
            date_color,
            header_fg,
            header_bg,
            tokens: Some(tokens),
        })
    }
}

/// Theme Registry managing built-in and user-defined themes.
#[derive(Debug, Default)]
pub struct ThemeRegistry {
    custom_themes: HashMap<String, CustomThemeDef>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        Self {
            custom_themes: HashMap::new(),
        }
    }

    /// Load custom themes from the given directory (e.g. `~/.config/termexplorer/themes/`).
    pub fn load_from_dir(&mut self, dir: &Path) {
        if !dir.exists() || !dir.is_dir() {
            return;
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(def) = toml::from_str::<CustomThemeDef>(&content) {
                            self.custom_themes
                                .insert(def.theme.name.to_lowercase(), def);
                        }
                    }
                }
            }
        }
    }

    /// Get user themes directory path.
    pub fn user_themes_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".config")
                .join("termexplorer")
                .join("themes")
        } else {
            PathBuf::from(".config/termexplorer/themes")
        }
    }

    /// List all custom theme names.
    pub fn custom_theme_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .custom_themes
            .values()
            .map(|d| d.theme.name.clone())
            .collect();
        names.sort();
        names
    }

    /// Get a theme by name, checking custom themes first, then built-in presets.
    pub fn get_theme(&self, name: &str, transparent: bool) -> Option<Theme> {
        if let Some(def) = self.custom_themes.get(&name.to_lowercase()) {
            if let Ok(theme) = def.to_theme(transparent) {
                return Some(theme);
            }
        }
        None
    }

    /// Export a theme to a TOML file.
    pub fn export_theme(def: &CustomThemeDef, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let serialized = toml::to_string_pretty(def).map_err(|e| e.to_string())?;
        fs::write(path, serialized).map_err(|e| e.to_string())?;
        Ok(())
    }
}
