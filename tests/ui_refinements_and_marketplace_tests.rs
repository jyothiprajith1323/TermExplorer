use ratatui::layout::Rect;
use tempfile::tempdir;
use termexplorer::config::Config;
use termexplorer::theme::Theme;
use termexplorer::ui::header::{header_click_action, HeaderAction};
use termexplorer::ui::layout::{compute_layout, LayoutConfig};
use termexplorer::ui::plugin_center::{CATEGORIES, MARKETPLACE_CATALOG};

#[test]
fn test_compact_terminal_proportions_and_height() {
    let cfg = Config::default();
    assert_eq!(
        cfg.terminal_height, 8,
        "Default terminal height must be compact (8 rows total, ~6 rows content)"
    );

    // On standard 120x40 workstation display:
    let area_40 = Rect::new(0, 0, 120, 40);
    let layout_cfg = LayoutConfig {
        show_sidebar: false,
        show_preview: true,
        show_bottom: true,
        dual_pane: false,
        sidebar_pct: 20,
        preview_pct: 30,
        bottom_height: cfg.terminal_height,
        preview_maximized: false,
        terminal_maximized: false,
    };

    let areas = compute_layout(area_40, &layout_cfg);
    let term_rect = areas.bottom_panel.expect("Bottom terminal must be shown");

    // Terminal height: 8 rows out of 40 = exactly 20%
    assert_eq!(term_rect.height, 8);
    let workspace_pct = (areas.explorer.height as f32 / area_40.height as f32) * 100.0;
    assert!(
        (75.0..=80.0).contains(&workspace_pct),
        "Workspace must receive ~75-80% height (actual: {:.1}%)",
        workspace_pct
    );

    // Maximized terminal takes ~80% height
    let mut max_cfg = layout_cfg.clone();
    max_cfg.terminal_maximized = true;
    let max_areas = compute_layout(area_40, &max_cfg);
    let max_term = max_areas.bottom_panel.expect("Maximized terminal");
    assert!(
        max_term.height >= 30,
        "Maximized terminal should receive ~80% height (actual: {})",
        max_term.height
    );
}

#[test]
fn test_file_explorer_preview_70_30_split() {
    let area = Rect::new(0, 0, 100, 40);
    let layout_cfg = LayoutConfig {
        show_sidebar: false,
        show_preview: true,
        show_bottom: true,
        dual_pane: false,
        sidebar_pct: 20,
        preview_pct: 30,
        bottom_height: 8,
        preview_maximized: false,
        terminal_maximized: false,
    };

    let areas = compute_layout(area, &layout_cfg);
    let prev = areas
        .preview
        .expect("Preview should be visible at 100 cols");

    assert_eq!(areas.explorer.width, 70, "Explorer should be 70% width");
    assert_eq!(prev.width, 30, "Preview should be 30% width");

    // On narrow terminals (< 70 cols), preview collapses to preserve explorer readability
    let narrow = Rect::new(0, 0, 60, 30);
    let narrow_areas = compute_layout(narrow, &layout_cfg);
    assert!(
        narrow_areas.preview.is_none(),
        "Preview should collapse on narrow screens"
    );
    assert_eq!(
        narrow_areas.explorer.width, 60,
        "Explorer gets 100% on narrow screen"
    );
}

#[test]
fn test_marketplace_catalog_and_categories() {
    assert!(
        MARKETPLACE_CATALOG.len() >= 6,
        "Marketplace should have at least 6 curated extensions"
    );
    assert_eq!(
        CATEGORIES,
        &[
            "All",
            "Developer Tools",
            "Inspection",
            "Productivity",
            "Telemetry",
            "UI & Themes"
        ]
    );

    // Verify key bundled extensions
    let has_workspace_tools = MARKETPLACE_CATALOG
        .iter()
        .any(|p| p.id == "workspace-tools" && !p.permissions.is_empty());
    let has_file_inspector = MARKETPLACE_CATALOG
        .iter()
        .any(|p| p.id == "file-inspector" && !p.permissions.is_empty());
    let has_git_supercharged = MARKETPLACE_CATALOG
        .iter()
        .any(|p| p.id == "git-supercharged");

    assert!(has_workspace_tools, "Workspace Tools must be in catalog");
    assert!(has_file_inspector, "File Inspector must be in catalog");
    assert!(has_git_supercharged, "Git Supercharged must be in catalog");

    for p in MARKETPLACE_CATALOG {
        assert!(!p.id.is_empty());
        assert!(!p.name.is_empty());
        assert!(!p.version.is_empty());
        assert!(!p.description.is_empty());
        assert!(!p.category.is_empty());
        assert!(!p.sha256.is_empty());
    }
}

#[test]
fn test_catppuccin_mocha_defaults_and_config_migration() {
    let cfg = Config::default();
    assert_eq!(cfg.theme.name, "Catppuccin Mocha");
    assert_eq!(cfg.theme.accent, "#89b4fa");
    assert_eq!(cfg.icon_pack, "catppuccin");
    assert!(cfg.theme.transparent_background);

    // Verify theme resolution
    let t = Theme::from_name(&cfg.theme.name, cfg.theme.transparent_background);
    assert_eq!(t.name, "Catppuccin Mocha");
    assert_eq!(t.bg, None, "Transparency should preserve None for bg");

    // Test migration from legacy tall terminal and default_dark theme
    let temp = tempdir().unwrap();
    let legacy_toml = r##"
show_sidebar = false
show_preview = true
show_terminal = true
terminal_height = 16
theme = { name = "default_dark", accent = "#61AFEF", transparent_background = true }
icon_pack = ""
"##;
    let legacy_path = temp.path().join("config.toml");
    std::fs::write(&legacy_path, legacy_toml).unwrap();

    let content = std::fs::read_to_string(&legacy_path).unwrap();
    let mut loaded: Config = toml::from_str(&content).unwrap();

    // Emulate migration logic
    let mut migrated = false;
    let current_theme = loaded.theme.name.to_lowercase();
    if current_theme == "default dark"
        || current_theme == "default_dark"
        || current_theme.is_empty()
    {
        loaded.theme.name = "Catppuccin Mocha".to_string();
        migrated = true;
    }
    if loaded.icon_pack.is_empty() {
        loaded.icon_pack = "catppuccin".to_string();
        migrated = true;
    }
    if loaded.terminal_height > 10 {
        loaded.terminal_height = 8;
        migrated = true;
    }

    assert!(migrated);
    assert_eq!(loaded.theme.name, "Catppuccin Mocha");
    assert_eq!(loaded.icon_pack, "catppuccin");
    assert_eq!(loaded.terminal_height, 8);
}

#[test]
fn test_header_click_actions_detection() {
    // Wide workstation display (120 cols)
    let width = 120;

    // Left brand click
    assert_eq!(header_click_action(5, width), Some(HeaderAction::Brand));

    // Right buttons: Settings is furthest right, Plugins is middle, Search is left
    // At width 120:
    // Settings: >= 120 - 16 = 104
    // Plugins: >= 120 - 32 = 88 and < 104
    // Search: >= 120 - 48 = 72 and < 88
    assert_eq!(
        header_click_action(115, width),
        Some(HeaderAction::Settings)
    );
    assert_eq!(header_click_action(95, width), Some(HeaderAction::Plugins));
    assert_eq!(header_click_action(80, width), Some(HeaderAction::Search));

    // Compact display (60 cols)
    let compact_width = 60;
    // Settings: >= 48
    // Plugins: >= 37 and < 48
    // Search: >= 27 and < 37
    assert_eq!(
        header_click_action(55, compact_width),
        Some(HeaderAction::Settings)
    );
    assert_eq!(
        header_click_action(42, compact_width),
        Some(HeaderAction::Plugins)
    );
    assert_eq!(
        header_click_action(30, compact_width),
        Some(HeaderAction::Search)
    );
}
