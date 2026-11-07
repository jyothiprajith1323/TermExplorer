//! Comprehensive tests for file/folder icons, Nerd Font glyphs, fallbacks, and custom overrides.

use std::fs;
use tempfile::tempdir;
use termexplorer::icons::{
    mappings::{resolve_file_icon, resolve_folder_icon},
    IconManager, IconPack,
};
use termexplorer::theme::CatppuccinPalette;

#[test]
fn test_all_five_icon_packs() {
    let p = CatppuccinPalette::mocha();

    for pack in IconPack::all() {
        let (icon, _color) = resolve_folder_icon("src", false, *pack, &p);
        if *pack == IconPack::None {
            assert!(icon.is_empty());
        } else {
            assert!(
                !icon.is_empty(),
                "Pack {:?} should have non-empty icon",
                pack
            );
        }
    }
}

#[test]
fn test_programming_languages_icon_resolution() {
    let p = CatppuccinPalette::mocha();

    let extensions = [
        ("rs", "main.rs"),
        ("py", "script.py"),
        ("js", "index.js"),
        ("ts", "app.ts"),
        ("tsx", "component.tsx"),
        ("c", "driver.c"),
        ("cpp", "engine.cpp"),
        ("go", "server.go"),
        ("html", "index.html"),
        ("css", "style.css"),
        ("toml", "config.toml"),
        ("json", "data.json"),
        ("md", "README.md"),
        ("sh", "deploy.sh"),
    ];

    for (ext, filename) in extensions {
        let (glyph, _col) = resolve_file_icon(filename, Some(ext), IconPack::Catppuccin, &p);
        assert!(!glyph.is_empty(), "Extension {} should resolve icon", ext);

        // Unicode fallback
        let (ugraph, _) = resolve_file_icon(filename, Some(ext), IconPack::Unicode, &p);
        assert!(!ugraph.is_empty());

        // ASCII fallback
        let (agraph, _) = resolve_file_icon(filename, Some(ext), IconPack::Ascii, &p);
        assert!(agraph.starts_with('[') && agraph.trim_end().ends_with(']'));
    }
}

#[test]
fn test_special_filename_matches() {
    let p = CatppuccinPalette::mocha();

    let files = [
        "Cargo.toml",
        "Cargo.lock",
        "package.json",
        "Dockerfile",
        "docker-compose.yml",
        ".gitignore",
        "Makefile",
    ];

    for file in files {
        let (glyph, _) = resolve_file_icon(file, None, IconPack::Catppuccin, &p);
        assert!(!glyph.is_empty(), "File {} should match special rule", file);
    }
}

#[test]
fn test_folder_category_mappings() {
    let p = CatppuccinPalette::mocha();

    let folders = ["src", "tests", "docs", "target", ".git", "config", "assets"];
    for folder in folders {
        let (icon, _) = resolve_folder_icon(folder, false, IconPack::Catppuccin, &p);
        assert!(!icon.is_empty());
    }
}

#[test]
fn test_custom_icon_overrides() {
    let dir = tempdir().expect("create tempdir");
    let icons_toml = dir.path().join("icons.toml");

    let content = r##"
pack = "catppuccin"

[extensions]
foo = { glyph = "󰘦", color = "#ff007f" }

[filenames]
special_secret = { glyph = "󰌆", color = "#ffd700" }

[folders]
my_vault = { glyph = "󰒃", color = "#00ffff" }
"##;

    fs::write(&icons_toml, content).expect("write icons.toml");

    let mut manager = IconManager::default();
    manager.load_from_file(&icons_toml);

    let p = CatppuccinPalette::mocha();

    // Check custom extension
    let (g1, c1) = manager.file_icon("test.foo", Some("foo"), &p);
    assert!(g1.contains("󰘦"));
    assert_eq!(c1, ratatui::style::Color::Rgb(255, 0, 127));

    // Check custom filename
    let (g2, c2) = manager.file_icon("special_secret", None, &p);
    assert!(g2.contains("󰌆"));
    assert_eq!(c2, ratatui::style::Color::Rgb(255, 215, 0));

    // Check custom folder
    let (g3, c3) = manager.folder_icon("my_vault", false, &p);
    assert!(g3.contains("󰒃"));
    assert_eq!(c3, ratatui::style::Color::Rgb(0, 255, 255));
}
