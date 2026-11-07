//! Design Tokens architecture mapping canonical palette colors to semantic UI roles.

use crate::theme::palette::CatppuccinPalette;
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeTokens {
    // ── Backgrounds ──
    pub bg_primary: Option<Color>,
    pub bg_secondary: Color,
    pub bg_surface: Color,
    pub bg_floating: Color,

    // ── Text & Content ──
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,

    // ── Accents & Selection ──
    pub accent_primary: Color,
    pub accent_secondary: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub border_normal: Color,
    pub border_focused: Color,

    // ── Semantic Status ──
    pub status_success: Color,
    pub status_warning: Color,
    pub status_error: Color,
    pub status_info: Color,

    // ── Filesystem Specific ──
    pub fs_directory: Color,
    pub fs_file: Color,
    pub fs_symlink: Color,
    pub fs_hidden: Color,
    pub fs_size: Color,
    pub fs_date: Color,

    // ── Syntax Highlighting Tokens ──
    pub syntax_keyword: Color,
    pub syntax_string: Color,
    pub syntax_function: Color,
    pub syntax_type: Color,
    pub syntax_number: Color,
    pub syntax_comment: Color,
    pub syntax_operator: Color,
}

impl ThemeTokens {
    /// Construct complete design tokens from a canonical Catppuccin palette.
    pub fn from_palette(p: &CatppuccinPalette, transparent: bool) -> Self {
        Self {
            bg_primary: if transparent { None } else { Some(p.base) },
            bg_secondary: p.mantle,
            bg_surface: p.surface0,
            bg_floating: p.crust,

            text_primary: p.text,
            text_secondary: p.subtext1,
            text_muted: p.overlay0,

            accent_primary: p.blue,
            accent_secondary: p.sapphire,
            selection_bg: p.surface1,
            selection_fg: p.text,
            border_normal: p.surface1,
            border_focused: p.blue,

            status_success: p.green,
            status_warning: p.yellow,
            status_error: p.red,
            status_info: p.sky,

            fs_directory: p.blue,
            fs_file: p.text,
            fs_symlink: p.teal,
            fs_hidden: p.overlay0,
            fs_size: p.subtext0,
            fs_date: p.overlay2,

            syntax_keyword: p.mauve,
            syntax_string: p.green,
            syntax_function: p.blue,
            syntax_type: p.yellow,
            syntax_number: p.peach,
            syntax_comment: p.overlay1,
            syntax_operator: p.sky,
        }
    }
}
