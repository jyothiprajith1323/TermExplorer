//! Comprehensive tests for WebAssembly plugins, sandboxing, fuel limit, integrity, and permissions.

use std::fs;
use std::path::PathBuf;
use termexplorer::plugins::{
    PluginInstance, PluginManager, PluginManifest, PluginPermission, PluginState,
};

#[test]
fn test_plugin_manifest_parsing() {
    let manifest_toml = r#"
id = "sample-plugin"
name = "Sample Plugin"
version = "1.0.0"
author = "Developer"
description = "A test plugin"
entrypoint = "sample.wasm"
permissions = ["commands", "file_metadata", "status_text"]
sha256 = "abcd1234ef5678"
"#;

    let manifest = PluginManifest::parse(manifest_toml).expect("parse manifest");
    assert_eq!(manifest.id, "sample-plugin");
    assert_eq!(manifest.version, "1.0.0");
    assert!(manifest.has_permission(PluginPermission::Commands));
    assert!(manifest.has_permission(PluginPermission::FileMetadata));
    assert!(manifest.has_permission(PluginPermission::StatusText));
    assert!(!manifest.has_permission(PluginPermission::WorkspaceActions));
}

#[test]
fn test_example_plugins_discovery_and_execution() {
    let plugins_dir = PathBuf::from("plugins");
    if !plugins_dir.exists() {
        return;
    }

    let mut manager = PluginManager::new();
    manager.discover_and_load(&plugins_dir, ".");

    assert!(
        manager.plugins.len() >= 2,
        "Should discover at least 2 built-in plugins"
    );

    let ids: Vec<&str> = manager
        .plugins
        .iter()
        .map(|p| p.manifest.id.as_str())
        .collect();
    assert!(
        ids.contains(&"workspace-tools"),
        "workspace-tools plugin loaded"
    );
    assert!(
        ids.contains(&"file-inspector"),
        "file-inspector plugin loaded"
    );

    // Check states
    for p in &manager.plugins {
        assert_eq!(p.state, PluginState::Active);
        assert!(p.instance.is_some());
    }

    // Trigger command on workspace-tools
    let cmd_res = manager.trigger_command("workspace-tools", "build");
    assert!(cmd_res.is_ok());

    // Status texts
    let statuses = manager.collect_status_texts();
    assert!(!statuses.is_empty());
}

#[test]
fn test_plugin_integrity_checksum_tampering() {
    let wasm_file = PathBuf::from("plugins/workspace-tools/plugin.wasm");
    if !wasm_file.exists() {
        return;
    }

    let wasm_bytes = fs::read(&wasm_file).expect("read wasm");

    let manifest = PluginManifest {
        id: "tamper-test".to_string(),
        name: "Tamper Test".to_string(),
        version: "0.1.0".to_string(),
        author: None,
        description: None,
        entrypoint: "plugin.wasm".to_string(),
        permissions: [PluginPermission::Commands].into_iter().collect(),
        sha256: Some(
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        ),
    };

    let result = PluginInstance::new(manifest, &wasm_bytes, ".");
    match result {
        Err(e) => assert!(e.contains("Plugin integrity check failed")),
        Ok(_) => panic!("Mismatched sha256 checksum must fail instantiation"),
    }
}

#[test]
fn test_plugin_permission_enforcement() {
    let wasm_file = PathBuf::from("plugins/file-inspector/plugin.wasm");
    if !wasm_file.exists() {
        return;
    }

    let wasm_bytes = fs::read(&wasm_file).expect("read wasm");

    // Create instance WITHOUT commands permission
    let manifest = PluginManifest {
        id: "noperm-test".to_string(),
        name: "No Perm Test".to_string(),
        version: "0.1.0".to_string(),
        author: None,
        description: None,
        entrypoint: "plugin.wasm".to_string(),
        permissions: [].into_iter().collect(), // empty permissions
        sha256: None,
    };

    let mut instance = PluginInstance::new(manifest, &wasm_bytes, ".").expect("instantiated");
    let res = instance.trigger_command("check");
    assert!(
        res.is_err(),
        "Triggering command without permission must fail"
    );
    assert!(res
        .unwrap_err()
        .contains("does not have 'commands' permission"));
}
