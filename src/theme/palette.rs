//! Official Catppuccin Palette — 26 canonical colors for all 4 flavors.
//! Source of truth: https://catppuccin.com/palette/

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CatppuccinFlavor {
    Mocha,
    Macchiato,
    Frappe,
    Latte,
}

impl CatppuccinFlavor {
    pub const ALL: [CatppuccinFlavor; 4] = [
        CatppuccinFlavor::Mocha,
        CatppuccinFlavor::Macchiato,
        CatppuccinFlavor::Frappe,
        CatppuccinFlavor::Latte,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Mocha => "Catppuccin Mocha",
            Self::Macchiato => "Catppuccin Macchiato",
            Self::Frappe => "Catppuccin Frappé",
            Self::Latte => "Catppuccin Latte",
        }
    }

    pub fn is_dark(&self) -> bool {
        !matches!(self, Self::Latte)
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Mocha => "Soothing dark palette, primary default",
            Self::Macchiato => "Medium-contrast dark palette",
            Self::Frappe => "Muted, balanced dark palette",
            Self::Latte => "Crisp, daylight low-contrast light palette",
        }
    }

    pub fn palette(&self) -> CatppuccinPalette {
        match self {
            Self::Mocha => CatppuccinPalette::mocha(),
            Self::Macchiato => CatppuccinPalette::macchiato(),
            Self::Frappe => CatppuccinPalette::frappe(),
            Self::Latte => CatppuccinPalette::latte(),
        }
    }
}

/// Helper to parse "#rrggbb" or "#rgb" hex string into Ratatui `Color::Rgb`.
pub fn parse_hex(hex: &str) -> Option<Color> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() == 3 {
        let r = u8::from_str_radix(&s[0..1], 16).ok()?;
        let g = u8::from_str_radix(&s[1..2], 16).ok()?;
        let b = u8::from_str_radix(&s[2..3], 16).ok()?;
        return Some(Color::Rgb(r * 17, g * 17, b * 17));
    }
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

/// The 26 canonical Catppuccin colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatppuccinPalette {
    pub rosewater: Color,
    pub flamingo: Color,
    pub pink: Color,
    pub mauve: Color,
    pub red: Color,
    pub maroon: Color,
    pub peach: Color,
    pub yellow: Color,
    pub green: Color,
    pub teal: Color,
    pub sky: Color,
    pub sapphire: Color,
    pub blue: Color,
    pub lavender: Color,
    pub text: Color,
    pub subtext1: Color,
    pub subtext0: Color,
    pub overlay2: Color,
    pub overlay1: Color,
    pub overlay0: Color,
    pub surface2: Color,
    pub surface1: Color,
    pub surface0: Color,
    pub base: Color,
    pub mantle: Color,
    pub crust: Color,
}

impl CatppuccinPalette {
    /// Official Catppuccin Mocha (Dark default).
    pub fn mocha() -> Self {
        Self {
            rosewater: Color::Rgb(0xf5, 0xe0, 0xdc),
            flamingo: Color::Rgb(0xf2, 0xcd, 0xcd),
            pink: Color::Rgb(0xf5, 0xc2, 0xe7),
            mauve: Color::Rgb(0xcb, 0xa6, 0xf7),
            red: Color::Rgb(0xf3, 0x8b, 0xa8),
            maroon: Color::Rgb(0xeb, 0xa0, 0xac),
            peach: Color::Rgb(0xfa, 0xb3, 0x87),
            yellow: Color::Rgb(0xf9, 0xe2, 0xaf),
            green: Color::Rgb(0xa6, 0xe3, 0xa1),
            teal: Color::Rgb(0x94, 0xe2, 0xd5),
            sky: Color::Rgb(0x89, 0xdc, 0xeb),
            sapphire: Color::Rgb(0x74, 0xc7, 0xec),
            blue: Color::Rgb(0x89, 0xb4, 0xfa),
            lavender: Color::Rgb(0xb4, 0xbe, 0xfe),
            text: Color::Rgb(0xcd, 0xd6, 0xf4),
            subtext1: Color::Rgb(0xba, 0xc2, 0xde),
            subtext0: Color::Rgb(0xa6, 0xad, 0xc8),
            overlay2: Color::Rgb(0x93, 0x99, 0xb2),
            overlay1: Color::Rgb(0x7f, 0x84, 0x9c),
            overlay0: Color::Rgb(0x6c, 0x70, 0x86),
            surface2: Color::Rgb(0x58, 0x5b, 0x70),
            surface1: Color::Rgb(0x45, 0x47, 0x5a),
            surface0: Color::Rgb(0x31, 0x32, 0x44),
            base: Color::Rgb(0x1e, 0x1e, 0x2e),
            mantle: Color::Rgb(0x18, 0x18, 0x25),
            crust: Color::Rgb(0x11, 0x11, 0x1b),
        }
    }

    /// Official Catppuccin Macchiato (Medium-contrast dark).
    pub fn macchiato() -> Self {
        Self {
            rosewater: Color::Rgb(0xf4, 0xdb, 0xd6),
            flamingo: Color::Rgb(0xf0, 0xc6, 0xc6),
            pink: Color::Rgb(0xf5, 0xbd, 0xe6),
            mauve: Color::Rgb(0xc6, 0xa0, 0xf6),
            red: Color::Rgb(0xed, 0x87, 0x96),
            maroon: Color::Rgb(0xee, 0x99, 0xa0),
            peach: Color::Rgb(0xf5, 0xa9, 0x7f),
            yellow: Color::Rgb(0xee, 0xd4, 0x9f),
            green: Color::Rgb(0xa6, 0xda, 0x95),
            teal: Color::Rgb(0x8b, 0xd5, 0xca),
            sky: Color::Rgb(0x91, 0xd7, 0xe3),
            sapphire: Color::Rgb(0x7d, 0xc4, 0xe4),
            blue: Color::Rgb(0x8a, 0xad, 0xf4),
            lavender: Color::Rgb(0xb7, 0xbd, 0xf8),
            text: Color::Rgb(0xca, 0xd3, 0xf5),
            subtext1: Color::Rgb(0xb8, 0xc0, 0xe0),
            subtext0: Color::Rgb(0xa5, 0xad, 0xcb),
            overlay2: Color::Rgb(0x93, 0x9a, 0xb7),
            overlay1: Color::Rgb(0x80, 0x87, 0xa2),
            overlay0: Color::Rgb(0x6e, 0x73, 0x8d),
            surface2: Color::Rgb(0x5b, 0x60, 0x78),
            surface1: Color::Rgb(0x49, 0x4d, 0x64),
            surface0: Color::Rgb(0x36, 0x3a, 0x4f),
            base: Color::Rgb(0x24, 0x27, 0x3a),
            mantle: Color::Rgb(0x1e, 0x20, 0x30),
            crust: Color::Rgb(0x18, 0x19, 0x26),
        }
    }

    /// Official Catppuccin Frappé (Muted dark).
    pub fn frappe() -> Self {
        Self {
            rosewater: Color::Rgb(0xf2, 0xd5, 0xcf),
            flamingo: Color::Rgb(0xee, 0xbe, 0xbe),
            pink: Color::Rgb(0xf4, 0xb8, 0xe4),
            mauve: Color::Rgb(0xca, 0x9e, 0xe6),
            red: Color::Rgb(0xe7, 0x82, 0x84),
            maroon: Color::Rgb(0xea, 0x99, 0x9c),
            peach: Color::Rgb(0xef, 0x9f, 0x76),
            yellow: Color::Rgb(0xe5, 0xc8, 0x90),
            green: Color::Rgb(0xa6, 0xd1, 0x89),
            teal: Color::Rgb(0x81, 0xc8, 0xbe),
            sky: Color::Rgb(0x99, 0xd1, 0xdb),
            sapphire: Color::Rgb(0x85, 0xc1, 0xdc),
            blue: Color::Rgb(0x8c, 0xaa, 0xee),
            lavender: Color::Rgb(0xba, 0xbb, 0xf1),
            text: Color::Rgb(0xc6, 0xd0, 0xf5),
            subtext1: Color::Rgb(0xb5, 0xbf, 0xe2),
            subtext0: Color::Rgb(0xa5, 0xad, 0xce),
            overlay2: Color::Rgb(0x94, 0x9c, 0xbb),
            overlay1: Color::Rgb(0x83, 0x8b, 0xa7),
            overlay0: Color::Rgb(0x73, 0x79, 0x94),
            surface2: Color::Rgb(0x62, 0x68, 0x80),
            surface1: Color::Rgb(0x51, 0x57, 0x6d),
            surface0: Color::Rgb(0x41, 0x45, 0x59),
            base: Color::Rgb(0x30, 0x34, 0x46),
            mantle: Color::Rgb(0x29, 0x2c, 0x3c),
            crust: Color::Rgb(0x23, 0x26, 0x34),
        }
    }

    /// Official Catppuccin Latte (Light).
    pub fn latte() -> Self {
        Self {
            rosewater: Color::Rgb(0xdc, 0x8a, 0x78),
            flamingo: Color::Rgb(0xdd, 0x78, 0x78),
            pink: Color::Rgb(0xea, 0x76, 0xcb),
            mauve: Color::Rgb(0x88, 0x39, 0xef),
            red: Color::Rgb(0xd2, 0x0f, 0x39),
            maroon: Color::Rgb(0xe6, 0x45, 0x53),
            peach: Color::Rgb(0xfe, 0x64, 0x0b),
            yellow: Color::Rgb(0xdf, 0x8e, 0x1d),
            green: Color::Rgb(0x40, 0xa0, 0x2b),
            teal: Color::Rgb(0x17, 0x92, 0x99),
            sky: Color::Rgb(0x04, 0xa5, 0xe5),
            sapphire: Color::Rgb(0x20, 0x9f, 0xb5),
            blue: Color::Rgb(0x1e, 0x66, 0xf5),
            lavender: Color::Rgb(0x72, 0x87, 0xfd),
            text: Color::Rgb(0x4c, 0x4f, 0x69),
            subtext1: Color::Rgb(0x5c, 0x5f, 0x77),
            subtext0: Color::Rgb(0x6c, 0x6f, 0x85),
            overlay2: Color::Rgb(0x7c, 0x7f, 0x93),
            overlay1: Color::Rgb(0x8c, 0x8f, 0xa1),
            overlay0: Color::Rgb(0x9c, 0xa0, 0xb0),
            surface2: Color::Rgb(0xac, 0xb0, 0xbe),
            surface1: Color::Rgb(0xbc, 0xc0, 0xcc),
            surface0: Color::Rgb(0xcc, 0xd0, 0xda),
            base: Color::Rgb(0xef, 0xf1, 0xf5),
            mantle: Color::Rgb(0xe6, 0xe9, 0xef),
            crust: Color::Rgb(0xdc, 0xe0, 0xe8),
        }
    }
}
