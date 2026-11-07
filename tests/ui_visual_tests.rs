//! Comprehensive UI Visual Verification and Screen Rendering Tests.
//! Tests responsive layouts (80x24, 100x30, 120x40, 160x50), visual hierarchy,
//! panel proportions, and captures rendered TUI screens.

use ratatui::{backend::TestBackend, buffer::Buffer, layout::Rect, Terminal};
use termexplorer::config::Config;
use termexplorer::monitor::Monitor;
use termexplorer::state::AppState;
use termexplorer::theme::Theme;
use termexplorer::ui::{
    command_palette::render_command_palette,
    file_explorer::render_file_explorer,
    git_panel::render_git_panel,
    header::render_header,
    layout::{compute_layout, LayoutConfig},
    preview::render_preview,
    settings::render_settings_studio,
    sidebar::render_sidebar,
    status_bar::render_status_bar,
    system_monitor::render_system_monitor,
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

#[test]
fn test_render_workstation_main_layout_120x40() {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut state = AppState::new(Config::default());
    state.workspace.name = "IronKV".to_string();
    state.show_sidebar = true;
    state.show_preview = true;
    let theme = Theme::default_dark();
    let snap = Monitor::new().refresh();

    terminal
        .draw(|frame| {
            let area = frame.area();
            let layout_cfg = LayoutConfig {
                show_sidebar: state.show_sidebar,
                show_preview: state.show_preview,
                show_bottom: false,
                dual_pane: false,
                sidebar_pct: 22,
                preview_pct: 38,
                bottom_height: 10,
            };
            let areas = compute_layout(area, &layout_cfg);

            render_header(frame, areas.header, &state, Some(&snap), &theme);
            if let Some(sb) = areas.sidebar {
                render_sidebar(frame, sb, &state, &theme);
            }
            render_file_explorer(frame, areas.explorer, &state, &theme);
            if let Some(pv) = areas.preview {
                render_preview(frame, pv, &state, &theme);
            }
            render_status_bar(frame, areas.status_bar, &state, &theme);
        })
        .unwrap();

    let screen = buffer_to_string(terminal.backend().buffer());

    std::fs::create_dir_all("screenshots").unwrap();
    std::fs::write("screenshots/main_workstation_120x40.txt", &screen).unwrap();

    assert!(screen.contains("TERMEXPLORER"));
    assert!(screen.contains("NORMAL"));
    assert!(screen.contains("EXPLORER"));
    assert!(screen.contains("WORKSPACE"));
}

#[test]
fn test_responsive_layout_adaptive_breakpoints() {
    let layout_cfg = LayoutConfig {
        show_sidebar: true,
        show_preview: true,
        show_bottom: false,
        dual_pane: false,
        sidebar_pct: 22,
        preview_pct: 35,
        bottom_height: 10,
    };

    // 1. Narrow terminal (80x24) -> sidebar auto-collapses
    let areas_80 = compute_layout(Rect::new(0, 0, 80, 24), &layout_cfg);
    assert!(
        areas_80.sidebar.is_none(),
        "Sidebar should auto-collapse under 90 cols"
    );
    assert!(
        areas_80.preview.is_some(),
        "Preview stays active at 80 cols"
    );

    // 2. Very narrow terminal (60x20) -> both sidebar and preview collapse
    let areas_60 = compute_layout(Rect::new(0, 0, 60, 20), &layout_cfg);
    assert!(areas_60.sidebar.is_none());
    assert!(
        areas_60.preview.is_none(),
        "Preview should auto-collapse under 65 cols"
    );
    assert_eq!(
        areas_60.explorer.width, 60,
        "Explorer should take 100% width on small screens"
    );

    // 3. Wide workstation terminal (160x50) -> full multi-column glory
    let areas_160 = compute_layout(Rect::new(0, 0, 160, 50), &layout_cfg);
    assert!(areas_160.sidebar.is_some());
    assert!(areas_160.preview.is_some());
    assert!(areas_160.explorer.width > 50);
}

#[test]
fn test_render_command_palette_modal() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::default_dark();
    let items = termexplorer::commands::CommandRegistry::search("");

    terminal
        .draw(|frame| {
            render_command_palette(frame, frame.area(), "git", &items, 0, &theme);
        })
        .unwrap();

    let screen = buffer_to_string(terminal.backend().buffer());
    let _ = std::fs::create_dir_all("screenshots");
    std::fs::write("screenshots/command_palette_modal.txt", &screen).unwrap();
    assert!(screen.contains("COMMAND PALETTE"));
}

#[test]
fn test_render_settings_studio_modal() {
    let backend = TestBackend::new(110, 35);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = AppState::new(Config::default());
    let theme = Theme::default_dark();

    terminal
        .draw(|frame| {
            render_settings_studio(frame, frame.area(), &state, &theme, 0, 0, false);
        })
        .unwrap();

    let screen = buffer_to_string(terminal.backend().buffer());
    let _ = std::fs::create_dir_all("screenshots");
    std::fs::write("screenshots/settings_studio_modal.txt", &screen).unwrap();
    assert!(screen.contains("SETTINGS STUDIO"));
    assert!(screen.contains("Categories"));
}

#[test]
fn test_render_system_telemetry_gauges() {
    let backend = TestBackend::new(120, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::default_dark();
    let snap = Monitor::new().refresh();

    terminal
        .draw(|frame| {
            render_system_monitor(frame, frame.area(), &snap, &theme);
        })
        .unwrap();

    let screen = buffer_to_string(terminal.backend().buffer());
    let _ = std::fs::create_dir_all("screenshots");
    std::fs::write("screenshots/system_telemetry.txt", &screen).unwrap();
    assert!(screen.contains("SYSTEM TELEMETRY"));
    assert!(screen.contains("CPU:"));
    assert!(screen.contains("RAM:"));
    assert!(screen.contains("DISKS"));
}

#[test]
fn test_render_git_workbench() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::default_dark();
    let mut git_status = termexplorer::git::GitRepoStatus::empty();
    git_status.is_repo = true;
    git_status.branch = "main".to_string();
    git_status.is_dirty = true;
    git_status
        .staged_files
        .push(termexplorer::git::GitFileEntry {
            path: "src/main.rs".to_string(),
            status: termexplorer::git::GitFileStatus::Modified,
            raw_code: "M".to_string(),
        });

    terminal
        .draw(|frame| {
            render_git_panel(frame, frame.area(), &git_status, 0, &theme);
        })
        .unwrap();

    let screen = buffer_to_string(terminal.backend().buffer());
    let _ = std::fs::create_dir_all("screenshots");
    std::fs::write("screenshots/git_workbench.txt", &screen).unwrap();
    assert!(screen.contains("GIT WORKBENCH"));
    assert!(screen.contains("main"));
}
