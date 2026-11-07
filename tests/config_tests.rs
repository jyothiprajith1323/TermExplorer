use std::path::PathBuf;
use termexplorer::config::{Config, PinnedFolder};

#[test]
fn test_default_config() {
    let cfg = Config::default();
    assert!(cfg.show_sidebar);
    assert!(cfg.show_preview);
    assert!(!cfg.show_terminal);
    assert!(!cfg.show_monitor);
    assert_eq!(cfg.sidebar_width, 20);
    assert_eq!(cfg.preview_width, 35);
    assert_eq!(cfg.editor.tab_width, 4);
    assert!(cfg.editor.auto_pair);
    assert!(cfg.theme.transparent_background);
}

#[test]
fn test_pin_and_unpin_folder() {
    let mut cfg = Config::default();
    assert!(cfg.pinned_folders.is_empty());

    let p1 = PathBuf::from("/home/user/projects");
    let p2 = PathBuf::from("/home/user/downloads");

    cfg.pin_folder("Projects".to_string(), p1.clone());
    assert_eq!(cfg.pinned_folders.len(), 1);
    assert_eq!(
        cfg.pinned_folders[0],
        PinnedFolder {
            name: "Projects".to_string(),
            path: p1.clone(),
        }
    );

    // Pinning the same folder twice should be a no-op
    cfg.pin_folder("Projects Dup".to_string(), p1.clone());
    assert_eq!(cfg.pinned_folders.len(), 1);

    cfg.pin_folder("Downloads".to_string(), p2);
    assert_eq!(cfg.pinned_folders.len(), 2);

    cfg.unpin_folder(0);
    assert_eq!(cfg.pinned_folders.len(), 1);
    assert_eq!(cfg.pinned_folders[0].name, "Downloads");
}

#[test]
fn test_config_serialization_roundtrip() {
    let mut cfg = Config::default();
    cfg.pin_folder("Workspace".to_string(), PathBuf::from("/workspace"));
    cfg.show_terminal = true;
    cfg.show_hidden = true;

    let toml_str = toml::to_string(&cfg).unwrap();
    let loaded: Config = toml::from_str(&toml_str).unwrap();

    assert_eq!(loaded.pinned_folders.len(), 1);
    assert_eq!(loaded.pinned_folders[0].name, "Workspace");
    assert!(loaded.show_terminal);
    assert!(loaded.show_hidden);
    assert_eq!(loaded.sidebar_width, cfg.sidebar_width);
}
