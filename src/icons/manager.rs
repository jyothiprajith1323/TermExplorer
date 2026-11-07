//! Icon manager and custom overrides registry.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use super::glyphs::{FILE_SPECIAL, FILE_SYMLINK};
use super::mappings::{resolve_file_icon, resolve_folder_icon};
use super::pack::IconPack;
use crate::theme::palette::{parse_hex, CatppuccinPalette};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomIconRule {
    pub glyph: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomIconsConfig {
    #[serde(default)]
    pub pack: Option<String>,
    #[serde(default)]
    pub extensions: HashMap<String, CustomIconRule>,
    #[serde(default)]
    pub filenames: HashMap<String, CustomIconRule>,
    #[serde(default)]
    pub folders: HashMap<String, CustomIconRule>,
}

#[derive(Debug, Clone)]
pub struct IconManager {
    pub pack: IconPack,
    custom_extensions: HashMap<String, (String, Option<Color>)>,
    custom_filenames: HashMap<String, (String, Option<Color>)>,
    custom_folders: HashMap<String, (String, Option<Color>)>,
}

impl Default for IconManager {
    fn default() -> Self {
        Self::new(IconPack::Catppuccin)
    }
}

impl IconManager {
    pub fn new(pack: IconPack) -> Self {
        Self {
            pack,
            custom_extensions: HashMap::new(),
            custom_filenames: HashMap::new(),
            custom_folders: HashMap::new(),
        }
    }

    /// Load custom overrides from a config file (e.g. `~/.config/termexplorer/icons.toml`).
    pub fn load_from_file(&mut self, path: &Path) {
        if !path.exists() {
            return;
        }

        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(cfg) = toml::from_str::<CustomIconsConfig>(&content) {
                if let Some(p) = cfg.pack {
                    self.pack = IconPack::parse(&p);
                }

                for (ext, rule) in cfg.extensions {
                    let col = rule.color.as_deref().and_then(parse_hex);
                    self.custom_extensions
                        .insert(ext.to_lowercase(), (rule.glyph, col));
                }

                for (file, rule) in cfg.filenames {
                    let col = rule.color.as_deref().and_then(parse_hex);
                    self.custom_filenames
                        .insert(file.to_lowercase(), (rule.glyph, col));
                }

                for (folder, rule) in cfg.folders {
                    let col = rule.color.as_deref().and_then(parse_hex);
                    self.custom_folders
                        .insert(folder.to_lowercase(), (rule.glyph, col));
                }
            }
        }
    }

    pub fn default_config_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".config")
                .join("termexplorer")
                .join("icons.toml")
        } else {
            PathBuf::from(".config/termexplorer/icons.toml")
        }
    }

    pub fn folder_icon(
        &self,
        name: &str,
        is_open: bool,
        palette: &CatppuccinPalette,
    ) -> (String, Color) {
        if self.pack == IconPack::None {
            return (String::new(), palette.blue);
        }

        let lower = name.to_ascii_lowercase();
        if let Some((glyph, col)) = self.custom_folders.get(&lower) {
            return (format!("{} ", glyph.trim()), col.unwrap_or(palette.blue));
        }

        resolve_folder_icon(name, is_open, self.pack, palette)
    }

    pub fn file_icon(
        &self,
        filename: &str,
        extension: Option<&str>,
        palette: &CatppuccinPalette,
    ) -> (String, Color) {
        if self.pack == IconPack::None {
            return (String::new(), palette.text);
        }

        let lower_name = filename.to_ascii_lowercase();
        if let Some((glyph, col)) = self.custom_filenames.get(&lower_name) {
            return (format!("{} ", glyph.trim()), col.unwrap_or(palette.text));
        }

        if let Some(ext) = extension {
            let lower_ext = ext.to_ascii_lowercase();
            if let Some((glyph, col)) = self.custom_extensions.get(&lower_ext) {
                return (format!("{} ", glyph.trim()), col.unwrap_or(palette.text));
            }
        }

        resolve_file_icon(filename, extension, self.pack, palette)
    }

    pub fn symlink_icon(&self, broken: bool, palette: &CatppuccinPalette) -> (String, Color) {
        if self.pack == IconPack::None {
            return (
                String::new(),
                if broken { palette.red } else { palette.teal },
            );
        }

        let symbol = match self.pack {
            IconPack::Catppuccin | IconPack::NerdFont => FILE_SYMLINK.nerd,
            IconPack::Unicode => FILE_SYMLINK.unicode,
            IconPack::Ascii => FILE_SYMLINK.ascii,
            IconPack::None => "",
        };

        let color = if broken { palette.red } else { palette.teal };

        (format!("{} ", symbol.trim()), color)
    }

    pub fn special_icon(&self, palette: &CatppuccinPalette) -> (String, Color) {
        if self.pack == IconPack::None {
            return (String::new(), palette.yellow);
        }

        let symbol = match self.pack {
            IconPack::Catppuccin | IconPack::NerdFont => FILE_SPECIAL.nerd,
            IconPack::Unicode => FILE_SPECIAL.unicode,
            IconPack::Ascii => FILE_SPECIAL.ascii,
            IconPack::None => "",
        };

        (format!("{} ", symbol.trim()), palette.yellow)
    }
}
