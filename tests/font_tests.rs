//! Tests for font detection, emulator configuration guides, and install script.

use termexplorer::fonts::{
    get_terminal_guides, install_script_content, user_fonts_dir, FontStatus,
};

#[test]
fn test_font_status_detection() {
    let status = FontStatus::detect();
    let summary = status.status_summary();
    assert!(!summary.is_empty());
    // Should never crash regardless of system state
}

#[test]
fn test_terminal_guides_content() {
    let guides = get_terminal_guides();
    assert!(guides.len() >= 4);

    let names: Vec<&str> = guides.iter().map(|g| g.name).collect();
    assert!(names.contains(&"Kitty"));
    assert!(names.contains(&"Alacritty"));
    assert!(names.contains(&"WezTerm"));
    assert!(names.contains(&"GNOME Terminal"));

    for guide in guides {
        assert!(!guide.config_path.is_empty());
        assert!(!guide.snippet.is_empty());
        assert!(!guide.instructions.is_empty());
        assert!(guide.snippet.contains("JetBrainsMono"));
    }
}

#[test]
fn test_install_script_validity() {
    let script = install_script_content();
    assert!(script.starts_with("#!/usr/bin/env bash"));
    assert!(script.contains("JetBrainsMono"));
    assert!(script.contains("fc-cache"));

    let dir = user_fonts_dir();
    assert!(dir.to_string_lossy().contains(".local/share/fonts"));
}
