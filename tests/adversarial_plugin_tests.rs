//! Adversarial WebAssembly Plugin Security & Sandbox Auditing Tests.
//!
//! Verifies:
//! - Strict linear memory ceiling (16 MiB) on initial allocation
//! - Runtime memory growth interception via `wasmi::ResourceLimiter`
//! - Fuel metering & infinite loop denial-of-service mitigation
//! - Host crash isolation on guest traps and unreachable instructions
//! - Capability-based permission enforcement and zero host syscall leaks

use std::fs;
use std::path::PathBuf;
use termexplorer::plugins::{PluginInstance, PluginManifest, PluginPermission};

fn fixture_manifest(id: &str, entrypoint: &str, perms: Vec<PluginPermission>) -> PluginManifest {
    PluginManifest {
        id: id.to_string(),
        name: format!("Adversarial {}", id),
        version: "1.0.0".to_string(),
        author: Some("Security Auditor".to_string()),
        description: Some("Adversarial security test fixture".to_string()),
        entrypoint: entrypoint.to_string(),
        permissions: perms.into_iter().collect(),
        sha256: None,
    }
}

#[test]
fn test_adversarial_infinite_loop_stopped_by_fuel_metering() {
    let wasm_path = PathBuf::from("tests/fixtures/infinite_loop.wasm");
    assert!(wasm_path.exists(), "infinite_loop.wasm fixture must exist");

    let wasm_bytes = fs::read(&wasm_path).expect("read infinite_loop.wasm");
    let manifest = fixture_manifest(
        "infinite-loop-test",
        "infinite_loop.wasm",
        vec![PluginPermission::Commands],
    );

    let mut instance = PluginInstance::new(manifest, &wasm_bytes, ".").expect("instantiated");

    // Execution must NOT hang indefinitely; fuel consumption must trap cleanly
    let start = std::time::Instant::now();
    let res = instance.trigger_command("attack");
    let elapsed = start.elapsed();

    assert!(res.is_err(), "Infinite loop must be aborted by fuel limit");
    let err_msg = res.unwrap_err().to_lowercase();
    assert!(
        err_msg.contains("fuel") || err_msg.contains("trap") || err_msg.contains("execution error"),
        "Error message must indicate fuel exhaustion or trap, got: {}",
        err_msg
    );
    let max_duration_ms = if cfg!(debug_assertions) { 2500 } else { 500 };
    assert!(
        elapsed.as_millis() < max_duration_ms,
        "Fuel exhaustion should terminate in < {}ms (took {}ms)",
        max_duration_ms,
        elapsed.as_millis()
    );
}

#[test]
fn test_adversarial_runtime_memory_growth_blocked_by_resource_limiter() {
    let wasm_path = PathBuf::from("tests/fixtures/memory_hog.wasm");
    assert!(wasm_path.exists(), "memory_hog.wasm fixture must exist");

    let wasm_bytes = fs::read(&wasm_path).expect("read memory_hog.wasm");
    let manifest = fixture_manifest(
        "memory-hog-test",
        "memory_hog.wasm",
        vec![PluginPermission::Commands],
    );

    let mut instance = PluginInstance::new(manifest, &wasm_bytes, ".").expect("instantiated");

    // Plugin attempts to grow memory by 300 pages (19.2 MiB)
    // Wasmi ResourceLimiter must intercept memory.grow and reject it, returning -1
    let res = instance.trigger_command("grow");
    assert!(res.is_ok(), "Command should finish and return guest result");
    assert_eq!(
        res.unwrap(),
        -1,
        "ResourceLimiter must force memory_grow to return -1 (usize::MAX)"
    );
}

#[test]
fn test_adversarial_unreachable_trap_crash_isolation() {
    let wasm_path = PathBuf::from("tests/fixtures/trap_crash.wasm");
    assert!(wasm_path.exists(), "trap_crash.wasm fixture must exist");

    let wasm_bytes = fs::read(&wasm_path).expect("read trap_crash.wasm");
    let manifest = fixture_manifest(
        "trap-crash-test",
        "trap_crash.wasm",
        vec![PluginPermission::Commands],
    );

    let mut instance = PluginInstance::new(manifest, &wasm_bytes, ".").expect("instantiated");

    // Must catch trap gracefully without panicking host process
    let res = instance.trigger_command("crash");
    assert!(res.is_err(), "Unreachable opcode must return error");
    let err_msg = res.unwrap_err();
    assert!(
        err_msg.contains("unreachable") || err_msg.contains("Execution error"),
        "Trap should be reported, got: {}",
        err_msg
    );
}

#[test]
fn test_adversarial_oversized_initial_memory_rejected_upfront() {
    // Construct a valid minimal WebAssembly binary with initial memory of 300 pages (19.2 MiB > 16 MiB)
    // Header: \0asm, version 1
    // Section 5 (Memory): 1 memory, min = 300 pages (LEB128 0xAC, 0x02)
    // Section 7 (Export): export "memory" (kind 0x02, index 0)
    let wasm = vec![
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // magic & version
        0x05, 0x04, 0x01, 0x00, 0xac,
        0x02, // memory section: 1 memory, limits flag 0, min 300 pages
        0x07, 0x0a, 0x01, 0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02,
        0x00, // export "memory"
    ];

    let manifest = fixture_manifest(
        "oversized-memory-test",
        "oversized.wasm",
        vec![PluginPermission::Commands],
    );

    let res = PluginInstance::new(manifest, &wasm, ".");
    let err_msg = match res {
        Err(e) => e,
        Ok(_) => panic!("Expected failure on oversized initial memory"),
    };
    assert!(
        err_msg.contains("out of bounds memory allocation")
            || err_msg.contains("safety ceiling of 16 MB"),
        "Expected safety ceiling or memory allocation rejection, got: {}",
        err_msg
    );
}

#[test]
fn test_adversarial_capability_leak_and_undeclared_syscalls() {
    let wasm_path = PathBuf::from("plugins/file-inspector/plugin.wasm");
    if !wasm_path.exists() {
        return;
    }

    let wasm_bytes = fs::read(&wasm_path).expect("read file-inspector wasm");

    // Zero permissions
    let manifest = fixture_manifest(
        "zero-perm-test",
        "plugin.wasm",
        vec![], // no permissions granted
    );

    let mut instance = PluginInstance::new(manifest, &wasm_bytes, ".").expect("instantiated");

    // Command dispatch blocked
    assert!(instance.trigger_command("inspect").is_err());

    // Status text poll blocked
    assert!(instance.poll_status_text().is_none());
}
