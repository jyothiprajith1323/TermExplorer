//! Comprehensive tests for Catppuccin themes, 26-color canonical palette, tokens, and custom TOML loader.

use ratatui::style::Color;
use std::fs;
use tempfile::tempdir;
use termexplorer::theme::{
    palette::parse_hex, CatppuccinFlavor, CatppuccinPalette, Theme, ThemeRegistry, ThemeTokens,
};

#[test]
fn test_canonical_catppuccin_mocha_palette() {
    let p = CatppuccinPalette::mocha();
    // Verify canonical hex values for Mocha
    assert_eq!(p.rosewater, Color::Rgb(245, 224, 220)); // #f5e0dc
    assert_eq!(p.flamingo, Color::Rgb(242, 205, 205)); // #f2cdcd
    assert_eq!(p.pink, Color::Rgb(245, 194, 231)); // #f5c2e7
    assert_eq!(p.mauve, Color::Rgb(203, 166, 247)); // #cba6f7
    assert_eq!(p.red, Color::Rgb(243, 139, 168)); // #f38ba8
    assert_eq!(p.maroon, Color::Rgb(235, 160, 172)); // #eba0ac
    assert_eq!(p.peach, Color::Rgb(250, 179, 135)); // #fab387
    assert_eq!(p.yellow, Color::Rgb(249, 226, 175)); // #f9e2af
    assert_eq!(p.green, Color::Rgb(166, 227, 161)); // #a6e3a1
    assert_eq!(p.teal, Color::Rgb(148, 226, 213)); // #94e2d5
    assert_eq!(p.sky, Color::Rgb(137, 220, 235)); // #89dceb
    assert_eq!(p.sapphire, Color::Rgb(116, 199, 236)); // #74c7ec
    assert_eq!(p.blue, Color::Rgb(137, 180, 250)); // #89b4fa
    assert_eq!(p.lavender, Color::Rgb(180, 190, 254)); // #b4befe
    assert_eq!(p.text, Color::Rgb(205, 214, 244)); // #cdd6f4
    assert_eq!(p.subtext1, Color::Rgb(186, 194, 222)); // #bac2de
    assert_eq!(p.subtext0, Color::Rgb(166, 173, 200)); // #a6adc8
    assert_eq!(p.overlay2, Color::Rgb(147, 153, 178)); // #9399b2
    assert_eq!(p.overlay1, Color::Rgb(127, 132, 156)); // #7f849c
    assert_eq!(p.overlay0, Color::Rgb(108, 112, 134)); // #6c7086
    assert_eq!(p.surface2, Color::Rgb(88, 91, 112)); // #585b70
    assert_eq!(p.surface1, Color::Rgb(69, 71, 90)); // #45475a
    assert_eq!(p.surface0, Color::Rgb(49, 50, 68)); // #313244
    assert_eq!(p.base, Color::Rgb(30, 30, 46)); // #1e1e2e
    assert_eq!(p.mantle, Color::Rgb(24, 24, 37)); // #181825
    assert_eq!(p.crust, Color::Rgb(17, 17, 27)); // #11111b
}

#[test]
fn test_all_four_catppuccin_flavors() {
    let flavors = [
        CatppuccinFlavor::Mocha,
        CatppuccinFlavor::Macchiato,
        CatppuccinFlavor::Frappe,
        CatppuccinFlavor::Latte,
    ];

    for flavor in flavors {
        let palette = flavor.palette();
        assert!(!flavor.name().is_empty());
        let tokens = ThemeTokens::from_palette(&palette, false);
        assert_eq!(tokens.bg_primary, Some(palette.base));
        assert_eq!(tokens.text_primary, palette.text);
        assert_eq!(tokens.status_success, palette.green);
        assert_eq!(tokens.status_error, palette.red);

        // Transparent mode
        let tokens_trans = ThemeTokens::from_palette(&palette, true);
        assert_eq!(tokens_trans.bg_primary, None);
    }
}

#[test]
fn test_theme_instantiation_and_transparency() {
    let mocha = Theme::catppuccin_mocha();
    assert_eq!(mocha.name, "Catppuccin Mocha");
    assert!(mocha.tokens.is_some());

    let lat = Theme::catppuccin_latte();
    assert_eq!(lat.name, "Catppuccin Latte");
    assert!(lat.tokens.is_some());

    let f = Theme::from_name("Catppuccin Frappé", true);
    assert_eq!(f.bg, None);
    assert_eq!(f.header_bg, None);
}

#[test]
fn test_hex_parsing() {
    assert_eq!(parse_hex("#1e1e2e"), Some(Color::Rgb(30, 30, 46)));
    assert_eq!(parse_hex("1e1e2e"), Some(Color::Rgb(30, 30, 46)));
    assert_eq!(parse_hex("#fff"), Some(Color::Rgb(255, 255, 255)));
    assert_eq!(parse_hex("invalid"), None);
}

#[test]
fn test_custom_theme_toml_loading() {
    let dir = tempdir().expect("create tempdir");
    let theme_file = dir.path().join("cyberpunk.toml");

    let toml_content = r##"
[theme]
name = "Cyberpunk Neon"
author = "Runner"
description = "High-voltage neon palette"
dark = true

[colors]
bg = "#0d0221"
fg = "#00f0ff"
accent = "#ff003c"
border = "#261447"
border_focused = "#fcee0a"
"##;

    fs::write(&theme_file, toml_content).expect("write theme");

    let mut registry = ThemeRegistry::new();
    registry.load_from_dir(dir.path());

    let names = registry.custom_theme_names();
    assert_eq!(names.len(), 1);
    assert_eq!(names[0], "Cyberpunk Neon");

    let theme = registry
        .get_theme("Cyberpunk Neon", false)
        .expect("theme found");
    assert_eq!(theme.name, "Cyberpunk Neon");
    assert_eq!(theme.bg, Some(Color::Rgb(13, 2, 33)));
    assert_eq!(theme.fg, Color::Rgb(0, 240, 255));
    assert_eq!(theme.accent, Color::Rgb(255, 0, 60));
}
