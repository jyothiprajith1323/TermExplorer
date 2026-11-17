//! Automated Regression Test Suite for Layout Ratios and Dynamic Settings Integration.
//!
//! Validates:
//! 1. Ratio validation (default 25/75, min 15/85, max 50/50, sum to 100%, safe clamping)
//! 2. Synchronized settings adjustments (Left/Right, complementary updates, save & reload)
//! 3. Workspace rendering at the five required ratios (25/75, 35/65, 40/60, 50/50, 15/85)
//! 4. Responsive terminal resizing and independent terminal drawer height
//! 5. Dual-pane mode preservation
//! 6. Config persistence across simulated application restarts
//! 7. Backward-compatible migration from legacy configurations

use ratatui::{backend::TestBackend, buffer::Buffer, layout::Rect, Terminal};
use tempfile::tempdir;
use termexplorer::config::{Config, LayoutSettings};
use termexplorer::monitor::Monitor;
use termexplorer::settings::persistence::SettingsPersistence;
use termexplorer::state::AppState;
use termexplorer::theme::Theme;
use termexplorer::ui::{
    file_explorer::render_file_explorer,
    header::render_header,
    hit_test::HitRegistry,
    layout::{compute_layout, LayoutConfig},
    preview::render_preview,
    status_bar::render_status_bar,
};

fn buffer_to_string(buffer: &Buffer) -> String {
    let mut s = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            s.push_str(buffer[(x, y)].symbol());
        }
        s.push('\n');
    }
    s
}

// ── 1. Ratio Validation & Boundary Tests ──────────────────────────────────────

#[test]
fn test_layout_ratio_defaults_and_sum_to_100() {
    let mut cfg = Config::default();
    assert_eq!(cfg.layout.explorer_percent(), 25);
    assert_eq!(cfg.layout.preview_percent(), 75);
    assert_eq!(cfg.layout.terminal_height_percent, 15);
    assert_eq!(
        cfg.layout.explorer_percent() + cfg.layout.preview_percent(),
        100
    );

    cfg.validate_and_clamp();
    assert!((cfg.layout.explorer_ratio - 0.25).abs() < 1e-4);
    assert!((cfg.layout.preview_ratio - 0.75).abs() < 1e-4);
    assert!((cfg.layout.explorer_ratio + cfg.layout.preview_ratio - 1.0).abs() < 1e-4);
}

#[test]
fn test_layout_ratio_clamping_limits() {
    let mut cfg = Config::default();

    // 1. Below minimum explorer ratio (e.g. 5% -> clamp to 15% explorer, 85% preview)
    cfg.layout.explorer_ratio = 0.05;
    cfg.layout.preview_ratio = 0.95;
    cfg.validate_and_clamp();
    assert_eq!(cfg.layout.explorer_percent(), 15);
    assert_eq!(cfg.layout.preview_percent(), 85);

    // 2. Above maximum explorer ratio (e.g. 70% -> clamp to 50% explorer, 50% preview)
    cfg.layout.explorer_ratio = 0.70;
    cfg.layout.preview_ratio = 0.30;
    cfg.validate_and_clamp();
    assert_eq!(cfg.layout.explorer_percent(), 50);
    assert_eq!(cfg.layout.preview_percent(), 50);

    // 3. Zero / Negative / NaN / Out-of-bounds handling
    cfg.layout.explorer_ratio = -0.50;
    cfg.layout.preview_ratio = 2.0;
    cfg.validate_and_clamp();
    assert_eq!(cfg.layout.explorer_percent(), 15);
    assert_eq!(cfg.layout.preview_percent(), 85);
    assert_eq!(
        cfg.layout.explorer_percent() + cfg.layout.preview_percent(),
        100
    );

    // 4. Zero explorer ratio clamps to minimum 15%
    cfg.layout.explorer_ratio = 0.0;
    cfg.validate_and_clamp();
    assert_eq!(cfg.layout.explorer_percent(), 15);
    assert_eq!(cfg.layout.preview_percent(), 85);
}

#[test]
fn test_settings_persistence_validation() {
    let mut cfg = Config::default();
    cfg.layout.explorer_ratio = 0.02; // invalid
    cfg.layout.terminal_height_percent = 90; // invalid

    let warnings = SettingsPersistence::validate(&mut cfg);
    assert!(!warnings.is_empty());
    assert_eq!(cfg.layout.explorer_percent(), 15);
    assert_eq!(cfg.layout.preview_percent(), 85);
    assert_eq!(cfg.layout.terminal_height_percent, 40);
}

// ── 2. Settings Interaction & Synchronization Tests ──────────────────────────

#[test]
fn test_layout_settings_helper_synchronization() {
    let mut layout = LayoutSettings::default();
    assert_eq!(layout.explorer_percent(), 25);
    assert_eq!(layout.preview_percent(), 75);

    // Set explorer to 35% -> preview must automatically become 65%
    layout.set_explorer_percent(35);
    assert_eq!(layout.explorer_percent(), 35);
    assert_eq!(layout.preview_percent(), 65);
    assert_eq!(layout.explorer_percent() + layout.preview_percent(), 100);

    // Set preview to 80% -> explorer must automatically become 20%
    layout.set_preview_percent(80);
    assert_eq!(layout.explorer_percent(), 20);
    assert_eq!(layout.preview_percent(), 80);
    assert_eq!(layout.explorer_percent() + layout.preview_percent(), 100);

    // Set preview to 50% -> explorer becomes 50%
    layout.set_preview_percent(50);
    assert_eq!(layout.explorer_percent(), 50);
    assert_eq!(layout.preview_percent(), 50);

    // Attempting to set explorer above 50% clamps to 50%
    layout.set_explorer_percent(60);
    assert_eq!(layout.explorer_percent(), 50);
    assert_eq!(layout.preview_percent(), 50);

    // Attempting to set explorer below 15% clamps to 15%
    layout.set_explorer_percent(5);
    assert_eq!(layout.explorer_percent(), 15);
    assert_eq!(layout.preview_percent(), 85);
}

// ── 3. Workspace Rendering at the Five Required Ratios ────────────────────────

#[test]
fn test_workspace_rendering_five_ratios_on_120x40() {
    std::fs::create_dir_all("screenshots").unwrap();

    let test_cases = [
        (
            "Default (25/75)",
            75,
            30,
            90,
            "screenshots/layout_ratio_25_75.txt",
        ),
        (
            "Alternative 1 (35/65)",
            65,
            42,
            78,
            "screenshots/layout_ratio_35_65.txt",
        ),
        (
            "Alternative 2 (40/60)",
            60,
            48,
            72,
            "screenshots/layout_ratio_40_60.txt",
        ),
        (
            "Alternative 3 (50/50)",
            50,
            60,
            60,
            "screenshots/layout_ratio_50_50.txt",
        ),
        (
            "Minimum (15/85)",
            85,
            18,
            102,
            "screenshots/layout_ratio_15_85.txt",
        ),
    ];

    let theme = Theme::catppuccin_mocha();
    let snap = Monitor::new().refresh();

    for (name, prev_pct, expected_exp_w, expected_prev_w, screenshot_path) in test_cases {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut config = Config::default();
        config.layout.set_preview_percent(prev_pct);
        config.validate_and_clamp();

        let mut state = AppState::new(config.clone());
        state.workspace.name = format!("Ratio_{}_{}", 100 - prev_pct, prev_pct);

        let layout_cfg = LayoutConfig {
            show_sidebar: false,
            show_preview: true,
            show_bottom: false,
            dual_pane: false,
            sidebar_pct: config.sidebar_width,
            preview_pct: config.layout.preview_percent(),
            bottom_height: 8,
            preview_maximized: false,
            terminal_maximized: false,
        };

        let areas = compute_layout(Rect::new(0, 0, 120, 40), &layout_cfg);

        assert_eq!(
            areas.explorer.width, expected_exp_w,
            "{} - Expected Explorer width {} cols, got {}",
            name, expected_exp_w, areas.explorer.width
        );
        assert_eq!(
            areas.preview.unwrap().width,
            expected_prev_w,
            "{} - Expected Preview width {} cols, got {}",
            name,
            expected_prev_w,
            areas.preview.unwrap().width
        );
        assert_eq!(
            areas.explorer.width + areas.preview.unwrap().width,
            120,
            "Total content width must equal 120"
        );

        let mut hit_reg = HitRegistry::new();
        terminal
            .draw(|frame| {
                render_header(
                    frame,
                    areas.header,
                    &state,
                    Some(&snap),
                    &theme,
                    &mut hit_reg,
                );
                render_file_explorer(frame, areas.explorer, &state, &theme, &mut hit_reg);
                if let Some(pv) = areas.preview {
                    render_preview(frame, pv, &state, &theme, &mut hit_reg);
                }
                render_status_bar(frame, areas.status_bar, &state, &theme);
            })
            .unwrap();

        let screen_text = buffer_to_string(terminal.backend().buffer());
        std::fs::write(screenshot_path, &screen_text).unwrap();

        assert!(screen_text.contains("TERMEXPLORER"));
        assert!(screen_text.contains("NORMAL"));
    }
}

// ── 4. Responsive Resizing & Terminal Drawer Independence ─────────────────────

#[test]
fn test_responsive_terminal_resizing_preserves_ratios() {
    let mut config = Config::default();
    config.layout.set_explorer_percent(25); // 25% Explorer, 75% Preview
    config.validate_and_clamp();

    let display_widths = [80, 100, 120, 140, 154, 160];

    for &w in &display_widths {
        let layout_cfg = LayoutConfig {
            show_sidebar: false,
            show_preview: true,
            show_bottom: false,
            dual_pane: false,
            sidebar_pct: 20,
            preview_pct: config.layout.preview_percent(),
            bottom_height: 8,
            preview_maximized: false,
            terminal_maximized: false,
        };

        let areas = compute_layout(Rect::new(0, 0, w, 40), &layout_cfg);
        let exp_w = areas.explorer.width;
        let prev_w = areas.preview.unwrap().width;

        assert_eq!(exp_w + prev_w, w);
        let actual_exp_ratio = exp_w as f32 / w as f32;
        let actual_prev_ratio = prev_w as f32 / w as f32;

        // Proportions must remain within ±1.5% of 25/75 across all screen sizes
        assert!(
            (actual_exp_ratio - 0.25).abs() < 0.015,
            "At width {}, explorer ratio was {:.3}",
            w,
            actual_exp_ratio
        );
        assert!(
            (actual_prev_ratio - 0.75).abs() < 0.015,
            "At width {}, preview ratio was {:.3}",
            w,
            actual_prev_ratio
        );
    }
}

#[test]
fn test_terminal_drawer_independence_from_horizontal_ratios() {
    let _config = Config::default();

    // 1. Terminal hidden
    let layout_no_term = LayoutConfig {
        show_sidebar: false,
        show_preview: true,
        show_bottom: false,
        dual_pane: false,
        sidebar_pct: 20,
        preview_pct: 75,
        bottom_height: 8,
        preview_maximized: false,
        terminal_maximized: false,
    };
    let areas_no_term = compute_layout(Rect::new(0, 0, 120, 40), &layout_no_term);
    assert_eq!(areas_no_term.explorer.width, 30);
    assert_eq!(areas_no_term.preview.unwrap().width, 90);
    assert!(areas_no_term.bottom_panel.is_none());

    // 2. Terminal visible (default ~15-20% height, 8 rows)
    let layout_term = LayoutConfig {
        show_sidebar: false,
        show_preview: true,
        show_bottom: true,
        dual_pane: false,
        sidebar_pct: 20,
        preview_pct: 75,
        bottom_height: 8,
        preview_maximized: false,
        terminal_maximized: false,
    };
    let areas_term = compute_layout(Rect::new(0, 0, 120, 40), &layout_term);

    // Horizontal widths must be identical regardless of whether terminal is shown
    assert_eq!(areas_term.explorer.width, 30);
    assert_eq!(areas_term.preview.unwrap().width, 90);
    assert!(areas_term.bottom_panel.is_some());
    assert_eq!(areas_term.bottom_panel.unwrap().height, 8);
}

// ── 5. Dual Pane Mode Preservation ───────────────────────────────────────────

#[test]
fn test_dual_pane_mode_preserves_preview_allocation() {
    let layout_dual = LayoutConfig {
        show_sidebar: false,
        show_preview: true,
        show_bottom: false,
        dual_pane: true,
        sidebar_pct: 20,
        preview_pct: 75,
        bottom_height: 8,
        preview_maximized: false,
        terminal_maximized: false,
    };

    let areas = compute_layout(Rect::new(0, 0, 120, 40), &layout_dual);
    // Explorer total 30 cols is split into primary (15) and secondary (15)
    assert_eq!(areas.explorer.width, 15);
    assert_eq!(areas.explorer_secondary.unwrap().width, 15);
    // Preview keeps its full 75% (90 cols)
    assert_eq!(areas.preview.unwrap().width, 90);
}

// ── 6. Persistence & Application Restart Simulation ──────────────────────────

#[test]
fn test_layout_persistence_across_sessions() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    // 1. Initial configuration: 25/75
    let mut initial = Config::default();
    assert_eq!(initial.layout.explorer_percent(), 25);
    assert_eq!(initial.layout.preview_percent(), 75);

    // 2. User changes ratio to 35/65
    initial.set_explorer_percent(35);
    assert_eq!(initial.layout.explorer_percent(), 35);
    assert_eq!(initial.layout.preview_percent(), 65);

    // 3. Save to disk
    let content = toml::to_string_pretty(&initial).unwrap();
    std::fs::write(&config_path, content).unwrap();

    // 4. Simulate application restart: load from disk
    let loaded_content = std::fs::read_to_string(&config_path).unwrap();
    let mut restored: Config = toml::from_str(&loaded_content).unwrap();
    restored.validate_and_clamp();

    assert_eq!(restored.layout.explorer_percent(), 35);
    assert_eq!(restored.layout.preview_percent(), 65);
    assert_eq!(restored.preview_width, 65);

    // 5. Change back to 25/75 and save
    restored.set_explorer_percent(25);
    let updated_content = toml::to_string_pretty(&restored).unwrap();
    std::fs::write(&config_path, updated_content).unwrap();

    // 6. Second restart: confirms 25/75 restored
    let re_read = std::fs::read_to_string(&config_path).unwrap();
    let mut final_cfg: Config = toml::from_str(&re_read).unwrap();
    final_cfg.validate_and_clamp();

    assert_eq!(final_cfg.layout.explorer_percent(), 25);
    assert_eq!(final_cfg.layout.preview_percent(), 75);
    assert_eq!(final_cfg.preview_width, 75);
}

// ── 7. Migration & Backward Compatibility ─────────────────────────────────────

#[test]
fn test_migration_legacy_preview_width_without_layout_table() {
    let legacy_toml = r##"
schema_version = 1
show_sidebar = false
show_preview = true
show_terminal = false
preview_width = 65
terminal_height = 8
"##;

    let (cfg, report) = termexplorer::config::ConfigMigrator::migrate(legacy_toml, None).unwrap();

    assert_eq!(report.final_version, 2);
    // Legacy preview_width: 65 must migrate to 65% preview / 35% explorer
    assert_eq!(cfg.layout.preview_percent(), 65);
    assert_eq!(cfg.layout.explorer_percent(), 35);
    assert_eq!(cfg.preview_width, 65);
}
