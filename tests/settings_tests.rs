use crossterm::event::{KeyCode, KeyModifiers};
use tempfile::tempdir;
use termexplorer::config::Config;
use termexplorer::settings::{
    get_section_items, BindingScope, KeyActionBinding, KeybindingManager, ProfileKind,
    SettingsPersistence, SettingsSection,
};
use termexplorer::theme::Theme;

#[test]
fn test_all_14_settings_sections() {
    let cfg = Config::default();
    assert_eq!(SettingsSection::ALL.len(), 14);

    for sec in SettingsSection::ALL {
        let items = get_section_items(*sec, &cfg);
        assert!(!items.is_empty(), "Section {:?} should have items", sec);
        assert!(!sec.title().is_empty());
        assert!(!sec.icon().is_empty());
    }
}

#[test]
fn test_12_themes_palette_and_transparency() {
    let theme_names = Theme::list_themes();
    assert!(
        theme_names.len() >= 12,
        "Should contain at least 12 themes (now 14 with full Catppuccin suite)"
    );

    for &(name, _desc) in theme_names {
        // Transparent enabled
        let t_trans = Theme::from_name(name, true);
        assert_eq!(t_trans.bg, None);

        // Opaque / default
        let t_opaque = Theme::from_name(name, false);
        assert!(!t_opaque.name.is_empty());
    }
}

#[test]
fn test_keybinding_conflict_detection() {
    let mut bindings = KeybindingManager::default_bindings();
    let report = KeybindingManager::check_conflicts(&bindings);
    assert!(
        !report.has_conflicts,
        "Default bindings should be conflict-free"
    );

    // Introduce artificial duplicate conflict
    bindings.push(KeyActionBinding {
        id: "duplicate_test",
        name: "Conflicting Action",
        scope: BindingScope::Global,
        primary_key: "F1 / ?".into(),
        description: "Conflicting duplicate shortcut",
    });

    let conflict_report = KeybindingManager::check_conflicts(&bindings);
    assert!(conflict_report.has_conflicts);
    assert!(!conflict_report.details.is_empty());
}

#[test]
fn test_key_event_formatting() {
    let key_str = KeybindingManager::format_key_event(
        KeyCode::Char('p'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert_eq!(key_str, "Ctrl+Shift+P");

    let f10_str = KeybindingManager::format_key_event(KeyCode::F(10), KeyModifiers::NONE);
    assert_eq!(f10_str, "F10");
}

#[test]
fn test_profiles_application() {
    let mut cfg = Config::default();

    // 1. Developer Profile
    ProfileKind::Developer.apply_to_config(&mut cfg);
    assert!(cfg.show_sidebar);
    assert!(cfg.show_terminal);

    // 2. Minimal Profile
    ProfileKind::Minimal.apply_to_config(&mut cfg);
    assert!(!cfg.show_sidebar);
    assert!(!cfg.show_terminal);

    // 3. Terminal Power User Profile
    ProfileKind::TerminalPowerUser.apply_to_config(&mut cfg);
    assert!(!cfg.show_sidebar);
    assert!(cfg.show_terminal);
    assert_eq!(cfg.terminal_height, 24);

    // 4. Monitoring Profile
    ProfileKind::Monitoring.apply_to_config(&mut cfg);
    assert!(cfg.show_monitor);
}

#[test]
fn test_settings_validation_and_backup() {
    let mut cfg = Config {
        sidebar_width: 999, // Out of bounds
        preview_width: 3,   // Out of bounds
        ..Config::default()
    };

    let warnings = SettingsPersistence::validate(&mut cfg);
    assert!(!warnings.is_empty());
    assert_eq!(cfg.sidebar_width, 50);
    assert_eq!(cfg.preview_width, 15);

    let dir = tempdir().unwrap();
    let export_path = dir.path().join("exported_config.toml");
    SettingsPersistence::export_to_file(&cfg, &export_path).expect("Export config");
    assert!(export_path.exists());

    let imported = SettingsPersistence::import_from_file(&export_path).expect("Import config");
    assert_eq!(imported.sidebar_width, 50);
}
