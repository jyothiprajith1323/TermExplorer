//! Comprehensive unit and integration test suite for Sprint 6B:
//! Configuration architecture, schema migrations, first-run detection, settings, and terminal environment.

use tempfile::tempdir;
use termexplorer::config::first_run::{FirstRunManager, FirstRunStatus};
use termexplorer::config::migration::{ConfigMigrator, CURRENT_SCHEMA_VERSION};
use termexplorer::config::Config;
use termexplorer::settings::keybindings::KeybindingManager;
use termexplorer::settings::{get_section_items, SettingsSection};
use termexplorer::shell::TerminalCapabilities;

// ── 1. Configuration Unit Tests ──────────────────────────────────────────────

#[test]
fn test_config_defaults() {
    let config = Config::default();

    assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
    assert!(!config.general.first_run_completed);
    assert!(config.general.confirm_delete);
    assert!(config.general.restore_session);
    assert!(config.general.watch_filesystem);

    // Layout defaults: 25% Explorer / 75% Preview / 15% Terminal drawer
    assert_eq!(config.layout.explorer_ratio, 0.25);
    assert_eq!(config.layout.preview_ratio, 0.75);
    assert_eq!(config.layout.terminal_height_percent, 15);
    assert!(!config.layout.show_terminal);
    assert!(config.layout.show_preview);
    assert!(!config.layout.show_sidebar);

    // Theme default: Catppuccin Mocha
    assert_eq!(config.theme.name, "Catppuccin Mocha");
    assert!(config.theme.transparent_background);

    // Editor defaults
    assert_eq!(config.editor.tab_width, 4);
    assert!(config.editor.auto_pair);
    assert!(config.editor.line_numbers);
    assert!(config.editor.syntax_highlight);
    assert!(config.editor.word_wrap);
    assert!(config.editor.show_completion);
}

#[test]
fn test_config_serialization_roundtrip() {
    let mut config = Config::default();
    config.theme.name = "Dracula".to_string();
    config.editor.tab_width = 2;
    config.layout.explorer_ratio = 0.30;
    config.layout.preview_ratio = 0.70;

    let serialized = toml::to_string_pretty(&config).expect("Serialization failed");
    let deserialized: Config = toml::from_str(&serialized).expect("Deserialization failed");

    assert_eq!(deserialized.schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(deserialized.theme.name, "Dracula");
    assert_eq!(deserialized.editor.tab_width, 2);
    assert_eq!(deserialized.layout.explorer_ratio, 0.30);
    assert_eq!(deserialized.layout.preview_ratio, 0.70);
}

#[test]
fn test_config_validation_and_clamping() {
    let mut config = Config::default();

    // Out-of-bounds layout ratios
    config.layout.explorer_ratio = 0.05; // Below min 0.15
    config.layout.preview_ratio = 0.95; // Above max 0.85
    config.layout.terminal_height_percent = 75; // Above max 40
    config.editor.tab_width = 7; // Invalid tab width (must be 2, 4, or 8)

    config.validate_and_clamp();

    assert_eq!(config.layout.explorer_ratio, 0.15);
    assert_eq!(config.layout.preview_ratio, 0.85);
    assert_eq!(config.layout.terminal_height_percent, 40);
    assert_eq!(config.editor.tab_width, 4); // Clamped/reset to 4
}

// ── 2. Schema Migration Tests ────────────────────────────────────────────────

#[test]
fn test_migration_from_unversioned_v1() {
    let v1_toml = r##"
theme = { name = "Nord", accent = "#88c0d0", transparent_background = false }
editor = { tab_width = 2, auto_pair = true, line_numbers = true }
show_hidden = true
icon_pack = "catppuccin"
"##;

    let dir = tempdir().expect("Failed to create tempdir");
    let (migrated, report) =
        ConfigMigrator::migrate(v1_toml, Some(dir.path())).expect("Migration failed");

    assert_eq!(report.initial_version, 1);
    assert_eq!(report.final_version, 2);
    assert!(report.migrated);

    // Verify user settings from v1 were strictly preserved
    assert_eq!(migrated.schema_version, 2);
    assert_eq!(migrated.theme.name, "Nord");
    assert_eq!(migrated.editor.tab_width, 2);
    assert!(migrated.show_hidden);

    // Existing user must have first_run_completed set to true to avoid unwanted interruptions
    assert!(migrated.general.first_run_completed);

    // Verify defaults applied for new tables
    assert!(migrated.general.confirm_delete);
    assert!(migrated.developer.git_status_enabled);
}

#[test]
fn test_migration_idempotence() {
    let config = Config::default();
    let toml_str = toml::to_string_pretty(&config).unwrap();

    let (first_run, report1) = ConfigMigrator::migrate(&toml_str, None).unwrap();
    assert!(!report1.migrated);
    assert_eq!(first_run.schema_version, 2);

    let second_toml = toml::to_string_pretty(&first_run).unwrap();
    let (second_run, report2) = ConfigMigrator::migrate(&second_toml, None).unwrap();
    assert!(!report2.migrated);
    assert_eq!(first_run, second_run);
}

#[test]
fn test_future_schema_rejection() {
    let future_toml = r#"
schema_version = 99
theme = { name = "FutureTheme" }
"#;

    let result = ConfigMigrator::migrate(future_toml, None);
    assert!(
        result.is_err(),
        "Future schema versions must be safely rejected"
    );
}

#[test]
fn test_config_atomic_export_and_import() {
    let dir = tempdir().unwrap();
    let export_file = dir.path().join("exported_config.toml");

    let mut config = Config::default();
    config.theme.name = "Tokyo Night".to_string();
    config.editor.tab_width = 8;
    config.layout.explorer_ratio = 0.20;
    config.layout.preview_ratio = 0.80;

    config.export_to_file(&export_file).expect("Export failed");
    assert!(export_file.exists());

    let imported = Config::import_from_file(&export_file).expect("Import failed");
    assert_eq!(imported.theme.name, "Tokyo Night");
    assert_eq!(imported.editor.tab_width, 8);
    assert_eq!(imported.layout.explorer_ratio, 0.20);
    assert_eq!(imported.layout.preview_ratio, 0.80);
}

// ── 3. First-Run Detection Tests ─────────────────────────────────────────────

#[test]
fn test_first_run_status_detection() {
    let dir = tempdir().unwrap();
    let state_file = dir.path().join("first_run.json");
    let non_existent_config = dir.path().join("does_not_exist.toml");

    // Case 1: Fresh first launch when neither config nor state exists
    let status = FirstRunManager::detect_status_with_paths(&non_existent_config, Some(&state_file));
    assert_eq!(status, FirstRunStatus::FreshFirstLaunch);

    // Case 2: Existing legacy config file
    let legacy_config = dir.path().join("legacy.toml");
    std::fs::write(&legacy_config, "theme = { name = \"Dracula\" }\n").unwrap();
    let status = FirstRunManager::detect_status_with_paths(&legacy_config, Some(&state_file));
    assert_eq!(status, FirstRunStatus::ExistingConfigLegacy);

    // Case 3: Corrupted config file
    let corrupted_config = dir.path().join("corrupted.toml");
    std::fs::write(&corrupted_config, "[[[invalid syntax {{{{").unwrap();
    let status = FirstRunManager::detect_status_with_paths(&corrupted_config, Some(&state_file));
    assert_eq!(status, FirstRunStatus::ExistingConfigCorrupted);

    // Case 4: Valid completed state
    let state = termexplorer::config::FirstRunState {
        completed: true,
        completed_at: Some("2026-11-14T12:00:00Z".to_string()),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        launch_count: 5,
    };
    FirstRunManager::save_state_to_path(&state, &state_file).unwrap();
    let status = FirstRunManager::detect_status_with_paths(&non_existent_config, Some(&state_file));
    assert_eq!(status, FirstRunStatus::ExistingConfigValid);
}

// ── 4. Settings Categories & Items Tests ─────────────────────────────────────

#[test]
fn test_all_14_settings_sections() {
    let config = Config::default();
    let sections = SettingsSection::ALL;

    assert_eq!(sections.len(), 14);

    for &sec in sections {
        let items = get_section_items(sec, &config);
        assert!(
            !items.is_empty(),
            "Section {:?} must contain configurable items",
            sec
        );

        for item in items {
            assert!(
                !item.key.is_empty(),
                "Item key cannot be empty in section {:?}",
                sec
            );
            assert!(
                !item.label.is_empty(),
                "Item label cannot be empty in section {:?}",
                sec
            );
            assert!(
                !item.description.is_empty(),
                "Item description cannot be empty in section {:?}",
                sec
            );
        }
    }
}

// ── 5. Keybindings & Conflict Detection Tests ────────────────────────────────

#[test]
fn test_keybinding_registry_and_no_conflicts() {
    let defaults = KeybindingManager::default_bindings();

    assert!(
        defaults.len() >= 20,
        "Must register comprehensive keyboard shortcuts"
    );

    // Verify 0 conflicts in default bindings
    let report = KeybindingManager::check_conflicts(&defaults);
    assert!(
        !report.has_conflicts,
        "Default bindings must have 0 conflicts. Found: {:?}",
        report.details
    );
}

#[test]
fn test_keybinding_search() {
    let defaults = KeybindingManager::default_bindings();

    let git_matches = KeybindingManager::search_bindings(&defaults, "git");
    assert!(
        !git_matches.is_empty(),
        "Search for 'git' should return Git bindings"
    );

    let f10_matches = KeybindingManager::search_bindings(&defaults, "F10");
    assert!(
        !f10_matches.is_empty(),
        "Search for 'F10' should return Settings Studio"
    );

    let non_existent = KeybindingManager::search_bindings(&defaults, "xyznonexistent999");
    assert!(non_existent.is_empty());
}

// ── 6. Terminal Environment Detection Tests ──────────────────────────────────

#[test]
fn test_terminal_capabilities_detection() {
    let caps = TerminalCapabilities::detect();

    assert!(!caps.term.is_empty(), "Detected TERM cannot be empty");
    assert!(caps.columns > 0, "Terminal columns must be positive");
    assert!(caps.rows > 0, "Terminal rows must be positive");

    // Shell detection
    assert!(!caps.default_shell.name.is_empty());
    assert!(!caps.default_shell.path.as_os_str().is_empty());

    // Diagnostic summary
    let summary = caps.diagnostic_summary();
    assert!(summary.contains("TERM="));
    assert!(summary.contains("PTY="));
}
