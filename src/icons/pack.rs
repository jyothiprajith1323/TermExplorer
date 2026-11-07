//! Icon pack definitions — Catppuccin, Nerd Font, Unicode, ASCII, None.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IconPack {
    #[default]
    Catppuccin,
    #[serde(rename = "nerd_font", alias = "nerdfont")]
    NerdFont,
    Unicode,
    Ascii,
    None,
}

impl IconPack {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Catppuccin => "catppuccin",
            Self::NerdFont => "nerdfont",
            Self::Unicode => "unicode",
            Self::Ascii => "ascii",
            Self::None => "none",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Catppuccin => "Catppuccin (Nerd Font + Palette Colors)",
            Self::NerdFont => "Nerd Font (Monochrome / Accent)",
            Self::Unicode => "Unicode (Standard Emojis & Symbols)",
            Self::Ascii => "ASCII (Pure Text Tags)",
            Self::None => "None (Disabled)",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().trim() {
            "catppuccin" | "cat" => Self::Catppuccin,
            "nerdfont" | "nerd_font" | "nerd" => Self::NerdFont,
            "unicode" | "emoji" => Self::Unicode,
            "ascii" | "text" | "plain" => Self::Ascii,
            "none" | "off" | "disabled" => Self::None,
            _ => Self::Catppuccin,
        }
    }

    pub fn all() -> &'static [IconPack] {
        &[
            IconPack::Catppuccin,
            IconPack::NerdFont,
            IconPack::Unicode,
            IconPack::Ascii,
            IconPack::None,
        ]
    }
}
