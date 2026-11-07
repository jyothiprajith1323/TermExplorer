use crate::theme::palette::CatppuccinPalette;
use crate::theme::tokens::ThemeTokens;
use crate::theme::Theme;

pub fn theme(transparent: bool) -> Theme {
    let p = CatppuccinPalette::macchiato();
    let tokens = ThemeTokens::from_palette(&p, transparent);
    Theme {
        name: "Catppuccin Macchiato",
        bg: tokens.bg_primary,
        fg: tokens.text_primary,
        accent: p.blue,
        selection_bg: p.surface1,
        selection_fg: p.text,
        border: p.surface0,
        border_focused: p.blue,
        status_fg: p.subtext0,
        error: p.red,
        warning: p.yellow,
        success: p.green,
        dir_color: p.blue,
        file_color: p.text,
        symlink_color: p.teal,
        hidden_color: p.overlay0,
        size_color: p.subtext0,
        date_color: p.overlay2,
        header_fg: p.text,
        header_bg: if transparent { None } else { Some(p.mantle) },
        tokens: Some(tokens),
    }
}
