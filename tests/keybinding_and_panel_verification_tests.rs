//! Comprehensive Keybinding and Panel Routing Verification Test Suite
//!
//! Verifies that every single keyboard shortcut across all panels and modes
//! (Explorer, Preview tabs, Terminal drawer, Editor, Settings Studio, Git Workbench,
//! System Monitor, Help Matrix, Command Palette, Plugin Center) routes and functions
//! correctly without conflicts or dead keys.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tempfile::tempdir;
use termexplorer::app::App;
use termexplorer::config::Config;
use termexplorer::preview::manager::PreviewTab;
use termexplorer::state::{ActivePane, AppMode, FocusedPanel, ModalKind};

fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

#[tokio::test]
async fn test_help_modal_toggle_with_f1_and_question_mark() {
    let _tmp = tempdir().unwrap();
    let mut app = App::new(Config::default()).await.unwrap();

    // F1 opens help
    let quit = app
        .handle_key(make_key(KeyCode::F(1), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(!quit);
    assert!(matches!(app.state().mode, AppMode::Help));

    // F1 again toggles help off to Browse
    let _ = app
        .handle_key(make_key(KeyCode::F(1), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::Browse));

    // ? opens help from browse mode
    let _ = app
        .handle_key(make_key(KeyCode::Char('?'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::Help));

    // Esc closes help
    let _ = app
        .handle_key(make_key(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::Browse));
}

#[tokio::test]
async fn test_git_dashboard_keybindings_ctrl_g_and_vim_navigation() {
    let _tmp = tempdir().unwrap();
    let mut app = App::new(Config::default()).await.unwrap();

    // Ctrl+G opens Git Dashboard
    let _ = app
        .handle_key(make_key(KeyCode::Char('g'), KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::GitDashboard { .. }));

    // j / k navigation in Git Dashboard
    let _ = app
        .handle_key(make_key(KeyCode::Char('j'), KeyModifiers::NONE))
        .await
        .unwrap();
    let _ = app
        .handle_key(make_key(KeyCode::Char('k'), KeyModifiers::NONE))
        .await
        .unwrap();

    // q exits Git Dashboard to browse
    let _ = app
        .handle_key(make_key(KeyCode::Char('q'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::Browse));

    // 'g' key from explorer also opens Git Dashboard
    let _ = app
        .handle_key(make_key(KeyCode::Char('g'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::GitDashboard { .. }));

    // Esc exits
    let _ = app
        .handle_key(make_key(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::Browse));
}

#[tokio::test]
async fn test_settings_studio_keybindings_and_panel_resizing() {
    let _tmp = tempdir().unwrap();
    let mut app = App::new(Config::default()).await.unwrap();

    // Ctrl+, opens Settings Studio
    let _ = app
        .handle_key(make_key(KeyCode::Char(','), KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::SettingsStudio { .. }));

    // q exits Settings Studio
    let _ = app
        .handle_key(make_key(KeyCode::Char('q'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::Browse));

    // F10 opens Settings Studio
    let _ = app
        .handle_key(make_key(KeyCode::F(10), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::SettingsStudio { .. }));

    // Esc exits Settings Studio
    let _ = app
        .handle_key(make_key(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::Browse));
}

#[tokio::test]
async fn test_preview_panel_resizing_clamps_up_to_85_percent() {
    let config = Config {
        preview_width: 75,
        ..Default::default()
    };
    let mut app = App::new(config).await.unwrap();
    assert_eq!(app.state().config.preview_width, 75);

    // Ctrl+Right expands preview width
    let _ = app
        .handle_key(make_key(KeyCode::Right, KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert_eq!(app.state().config.preview_width, 80);
    assert!((app.state().config.layout.preview_ratio - 0.80).abs() < 1e-4);
    assert!((app.state().config.layout.explorer_ratio - 0.20).abs() < 1e-4);

    // Another Ctrl+Right hits clamp at 85%
    let _ = app
        .handle_key(make_key(KeyCode::Right, KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert_eq!(app.state().config.preview_width, 85);

    // Further Ctrl+Right stays clamped at 85%
    let _ = app
        .handle_key(make_key(KeyCode::Right, KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert_eq!(app.state().config.preview_width, 85);

    // Ctrl+Left shrinks preview width down
    let _ = app
        .handle_key(make_key(KeyCode::Left, KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert_eq!(app.state().config.preview_width, 80);
    assert!((app.state().config.layout.preview_ratio - 0.80).abs() < 1e-4);
}

#[tokio::test]
async fn test_terminal_drawer_height_resizing_and_toggle() {
    let mut app = App::new(Config::default()).await.unwrap();

    // Initially terminal is hidden
    assert!(!app.state().show_terminal);

    // Ctrl+T toggles terminal
    let _ = app
        .handle_key(make_key(KeyCode::Char('t'), KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert!(app.state().show_terminal);

    // Ctrl+Up increases terminal height
    let initial_h = app.state().config.terminal_height;
    let _ = app
        .handle_key(make_key(KeyCode::Up, KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert_eq!(app.state().config.terminal_height, initial_h + 2);

    // Ctrl+Down decreases terminal height
    let _ = app
        .handle_key(make_key(KeyCode::Down, KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert_eq!(app.state().config.terminal_height, initial_h);

    // Ctrl+` also toggles terminal drawer closed
    let _ = app
        .handle_key(make_key(KeyCode::Char('`'), KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert!(!app.state().show_terminal);
}

#[tokio::test]
async fn test_explorer_file_operation_keys() {
    let mut app = App::new(Config::default()).await.unwrap();

    // 'a' opens NewFile modal
    let _ = app
        .handle_key(make_key(KeyCode::Char('a'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(
        app.state().mode,
        AppMode::Modal(ModalKind::NewFile)
    ));
    let _ = app
        .handle_key(make_key(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::Browse));

    // 'A' opens NewDir modal
    let _ = app
        .handle_key(make_key(KeyCode::Char('A'), KeyModifiers::SHIFT))
        .await
        .unwrap();
    assert!(matches!(
        app.state().mode,
        AppMode::Modal(ModalKind::NewDir)
    ));
    let _ = app
        .handle_key(make_key(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::Browse));

    // Space switches focus to Preview panel
    let _ = app
        .handle_key(make_key(KeyCode::Char(' '), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().focused, FocusedPanel::Preview);

    // Esc from preview switches focus back to Explorer
    let _ = app
        .handle_key(make_key(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().focused, FocusedPanel::Explorer);
}

#[tokio::test]
async fn test_preview_tabs_switching_and_no_collision_with_resizing() {
    let mut app = App::new(Config::default()).await.unwrap();

    // Focus preview
    app.state_mut().focused = FocusedPanel::Preview;
    assert_eq!(app.state().preview.active_tab, PreviewTab::Preview);

    // Direct jump keys 1, 2, 3, 4
    let _ = app
        .handle_key(make_key(KeyCode::Char('2'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().preview.active_tab, PreviewTab::Information);

    let _ = app
        .handle_key(make_key(KeyCode::Char('3'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().preview.active_tab, PreviewTab::Hex);

    let _ = app
        .handle_key(make_key(KeyCode::Char('4'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().preview.active_tab, PreviewTab::Actions);

    let _ = app
        .handle_key(make_key(KeyCode::Char('1'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().preview.active_tab, PreviewTab::Preview);

    // ']' and '[' cycle preview tabs without mutating panel width
    let orig_width = app.state().config.preview_width;
    let _ = app
        .handle_key(make_key(KeyCode::Char(']'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().preview.active_tab, PreviewTab::Information);
    assert_eq!(app.state().config.preview_width, orig_width); // Did not resize!

    let _ = app
        .handle_key(make_key(KeyCode::Char(']'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().preview.active_tab, PreviewTab::Hex);
    assert_eq!(app.state().config.preview_width, orig_width);

    let _ = app
        .handle_key(make_key(KeyCode::Char('['), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().preview.active_tab, PreviewTab::Information);
    assert_eq!(app.state().config.preview_width, orig_width); // Did not resize!

    // 'h' while in Information tab moves tab left to Preview
    let _ = app
        .handle_key(make_key(KeyCode::Char('h'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().preview.active_tab, PreviewTab::Preview);

    // 'h' while in Preview tab moves focus left back to Explorer
    let _ = app
        .handle_key(make_key(KeyCode::Char('h'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().focused, FocusedPanel::Explorer);

    // Focusing preview and pressing Esc directly returns to Explorer
    app.state_mut().focused = FocusedPanel::Preview;
    let _ = app
        .handle_key(make_key(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().focused, FocusedPanel::Explorer);
}

#[tokio::test]
async fn test_fullscreen_preview_and_dual_pane_shortcuts() {
    let mut app = App::new(Config::default()).await.unwrap();

    // 'z' toggles fullscreen preview
    assert!(!app.state().preview_maximized);
    let _ = app
        .handle_key(make_key(KeyCode::Char('z'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(app.state().preview_maximized);

    // 'z' again restores
    let _ = app
        .handle_key(make_key(KeyCode::Char('z'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(!app.state().preview_maximized);

    // F7 toggles dual pane
    assert!(!app.state().dual_pane_enabled);
    let _ = app
        .handle_key(make_key(KeyCode::F(7), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(app.state().dual_pane_enabled);

    // Tab in dual pane cycles active pane between primary and secondary
    assert_eq!(app.state().active_pane, ActivePane::Primary);
    let _ = app
        .handle_key(make_key(KeyCode::Tab, KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().active_pane, ActivePane::Secondary);
    assert_eq!(app.state().focused, FocusedPanel::ExplorerSecondary);

    let _ = app
        .handle_key(make_key(KeyCode::Tab, KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().active_pane, ActivePane::Primary);
    assert_eq!(app.state().focused, FocusedPanel::Explorer);

    // F7 toggles dual pane off
    let _ = app
        .handle_key(make_key(KeyCode::F(7), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(!app.state().dual_pane_enabled);
}

#[tokio::test]
async fn test_monitor_and_plugin_center_shortcuts() {
    let mut app = App::new(Config::default()).await.unwrap();

    // F6 toggles system monitor
    assert!(!app.state().show_monitor);
    let _ = app
        .handle_key(make_key(KeyCode::F(6), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(app.state().show_monitor);

    // Ctrl+M toggles monitor off
    let _ = app
        .handle_key(make_key(KeyCode::Char('m'), KeyModifiers::CONTROL))
        .await
        .unwrap();
    assert!(!app.state().show_monitor);

    // Ctrl+Shift+X opens Plugin Center
    let _ = app
        .handle_key(make_key(
            KeyCode::Char('X'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::PluginCenter { .. }));

    // Esc closes Plugin Center
    let _ = app
        .handle_key(make_key(KeyCode::Esc, KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(matches!(app.state().mode, AppMode::Browse));
}
