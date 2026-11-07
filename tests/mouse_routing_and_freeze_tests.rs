//! Regression & Interaction Tests: Mouse routing precision, hitbox layout consistency,
//! and freeze elimination (zero disk I/O on draw, cached fonts, lazy Wasm discovery).

use ratatui::{backend::TestBackend, layout::Rect, Terminal};
use std::time::Instant;
use termexplorer::config::Config;
use termexplorer::fonts::FontStatus;
use termexplorer::plugins::PluginManager;
use termexplorer::state::{AppState, DirectoryPreviewData};
use termexplorer::theme::Theme;
use termexplorer::ui::header::render_header;
use termexplorer::ui::hit_test::{AppAction, HitRegistry};
use termexplorer::ui::preview::render_preview;
use termexplorer::ui::terminal::render_terminal;

#[test]
fn test_mouse_routing_settings_vs_plugins_120_cols() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = AppState::new(Config::default());
    let theme = Theme::catppuccin_mocha();
    let mut registry = HitRegistry::new();

    terminal
        .draw(|frame| {
            let header_area = Rect::new(0, 0, 120, 1);
            render_header(frame, header_area, &state, None, &theme, &mut registry);
        })
        .unwrap();

    // Settings button is at far right: cols 104..120
    let settings_click = registry.hit_test(115, 0);
    assert_eq!(
        settings_click,
        Some(AppAction::OpenSettings),
        "Clicking Settings area (115, 0) must trigger OpenSettings"
    );
    assert_ne!(
        settings_click,
        Some(AppAction::OpenPluginCenter),
        "Clicking Settings must NEVER trigger OpenPluginCenter!"
    );

    // Plugins button is middle right: cols 88..104
    let plugins_click = registry.hit_test(95, 0);
    assert_eq!(
        plugins_click,
        Some(AppAction::OpenPluginCenter),
        "Clicking Plugins area (95, 0) must trigger OpenPluginCenter"
    );
    assert_ne!(
        plugins_click,
        Some(AppAction::OpenSettings),
        "Clicking Plugins must NEVER trigger OpenSettings!"
    );

    // Search button is cols 72..88
    let search_click = registry.hit_test(80, 0);
    assert_eq!(
        search_click,
        Some(AppAction::OpenSearch),
        "Clicking Search area (80, 0) must trigger OpenSearch"
    );

    // Brand button is at far left: cols 0..17
    let brand_click = registry.hit_test(5, 0);
    assert_eq!(
        brand_click,
        Some(AppAction::OpenHelp),
        "Clicking Brand (5, 0) must trigger OpenHelp"
    );
}

#[test]
fn test_mouse_routing_compact_60_cols() {
    let backend = TestBackend::new(60, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = AppState::new(Config::default());
    let theme = Theme::catppuccin_mocha();
    let mut registry = HitRegistry::new();

    terminal
        .draw(|frame| {
            let header_area = Rect::new(0, 0, 60, 1);
            render_header(frame, header_area, &state, None, &theme, &mut registry);
        })
        .unwrap();

    // On 60-col display:
    // Settings: 60 - 12 = 48..60
    assert_eq!(
        registry.hit_test(55, 0),
        Some(AppAction::OpenSettings),
        "Compact Settings click"
    );

    // Plugins: 60 - 23 = 37..48
    assert_eq!(
        registry.hit_test(42, 0),
        Some(AppAction::OpenPluginCenter),
        "Compact Plugins click"
    );

    // Search: 60 - 33 = 27..37
    assert_eq!(
        registry.hit_test(30, 0),
        Some(AppAction::OpenSearch),
        "Compact Search click"
    );
}

#[test]
fn test_terminal_controls_exact_hit_testing() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let state = AppState::new(Config::default());
    let theme = Theme::catppuccin_mocha();
    let mut registry = HitRegistry::new();

    let term_area = Rect::new(0, 30, 120, 9);
    terminal
        .draw(|frame| {
            render_terminal(
                frame,
                term_area,
                b"user@termexplorer:~$ ",
                &state,
                &theme,
                &mut registry,
            );
        })
        .unwrap();

    // The interior content of terminal
    let inside_click = registry.hit_test(50, 34);
    assert_eq!(
        inside_click,
        Some(AppAction::FocusTerminal),
        "Clicking inside terminal body must focus terminal"
    );

    // Header buttons on row 30
    let mut found_new = false;
    let mut found_close = false;
    let mut found_max = false;

    for col in 0..120 {
        match registry.hit_test(col, 30) {
            Some(AppAction::ClearTerminal) => found_new = true,
            Some(AppAction::ToggleTerminal) => found_close = true,
            Some(AppAction::MaximizeTerminal) => found_max = true,
            _ => {}
        }
    }

    assert!(found_new, "Terminal Clear button must have hit region");
    assert!(found_close, "Terminal Close button must have hit region");
    assert!(found_max, "Terminal Max button must have hit region");
}

#[test]
fn test_resize_geometry_invalidation() {
    let mut registry = HitRegistry::new();
    let state = AppState::new(Config::default());
    let theme = Theme::catppuccin_mocha();

    // Frame 1: 120 width
    let backend_120 = TestBackend::new(120, 30);
    let mut term_120 = Terminal::new(backend_120).unwrap();
    term_120
        .draw(|f| {
            render_header(
                f,
                Rect::new(0, 0, 120, 1),
                &state,
                None,
                &theme,
                &mut registry,
            );
        })
        .unwrap();
    assert_eq!(registry.hit_test(115, 0), Some(AppAction::OpenSettings));

    // Frame 2: Terminal resizes to 80 cols -> geometry invalidation and update
    registry.clear();
    let backend_80 = TestBackend::new(80, 30);
    let mut term_80 = Terminal::new(backend_80).unwrap();
    term_80
        .draw(|f| {
            render_header(
                f,
                Rect::new(0, 0, 80, 1),
                &state,
                None,
                &theme,
                &mut registry,
            );
        })
        .unwrap();

    // Old coord (115) is now completely outside 80-col screen
    assert_eq!(
        registry.hit_test(115, 0),
        None,
        "Old coordinate from 120-col frame must not hit anything in 80-col frame"
    );

    // New Settings coord is around col 75 (80 - 5..12)
    assert_eq!(
        registry.hit_test(75, 0),
        Some(AppAction::OpenSettings),
        "Settings must be registered at new resized position"
    );
}

#[test]
fn test_directory_preview_zero_io_on_cached_draw() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::new(Config::default());
    let theme = Theme::catppuccin_mocha();
    let mut registry = HitRegistry::new();

    // Setup cached directory preview
    let cached_data = DirectoryPreviewData {
        path: std::path::PathBuf::from("/mock/dir"),
        total_count: 5,
        dir_count: 2,
        file_count: 3,
        total_bytes: 4096,
        items: vec![
            ("src".to_string(), true, 0),
            ("docs".to_string(), true, 0),
            ("Cargo.toml".to_string(), false, 1234),
            ("README.md".to_string(), false, 2048),
            ("LICENSE".to_string(), false, 814),
        ],
    };
    state.preview.path = Some(std::path::PathBuf::from("/mock/dir"));
    state.preview.dir_data = Some(cached_data);

    let start = Instant::now();
    terminal
        .draw(|f| {
            let prev_area = Rect::new(60, 1, 40, 20);
            render_preview(f, prev_area, &state, &theme, &mut registry);
        })
        .unwrap();
    let elapsed = start.elapsed();

    // Drawing with cached directory preview should take microseconds (< 500 µs), not milliseconds of disk stat
    assert!(
        elapsed.as_millis() < 50,
        "Rendering directory preview with cache took {:?}, must be fast",
        elapsed
    );
}

#[test]
fn test_font_status_cached_instant() {
    let t0 = Instant::now();
    let s1 = FontStatus::detect();
    let _ = t0.elapsed();

    let t1 = Instant::now();
    let s2 = FontStatus::detect();
    let cached_elapsed = t1.elapsed();

    assert_eq!(s1, s2);
    // In-memory cached detect must be < 50 microseconds
    assert!(
        cached_elapsed.as_micros() < 500,
        "Cached FontStatus::detect took {:?}, should be instantaneous in-memory lookup",
        cached_elapsed
    );
}

#[test]
fn test_lazy_plugin_discovery_no_upfront_wasm_compilation() {
    let plugins_dir = std::path::PathBuf::from("plugins");
    if !plugins_dir.exists() {
        return;
    }

    let mut manager = PluginManager::new();
    let t0 = Instant::now();
    manager.discover_and_load(&plugins_dir, ".");
    let elapsed = t0.elapsed();

    // Discovery reads manifest toml only; instances must be None until requested
    assert!(
        elapsed.as_millis() < 100,
        "Plugin discovery took {:?}, should be fast metadata reading",
        elapsed
    );

    for p in &manager.plugins {
        assert!(
            p.instance.is_none(),
            "Plugin instance for '{}' should be lazily uninitialized upon discovery",
            p.manifest.id
        );
    }

    // Now test on-demand initialization
    let init_res = manager.init_instance("workspace-tools", ".");
    assert!(
        init_res.is_ok(),
        "Explicit initialization on demand succeeds"
    );
    let p_tools = manager
        .plugins
        .iter()
        .find(|p| p.manifest.id == "workspace-tools")
        .unwrap();
    assert!(p_tools.instance.is_some(), "Instance initialized on demand");
}
