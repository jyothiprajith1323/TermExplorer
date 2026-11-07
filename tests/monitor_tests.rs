use termexplorer::monitor::Monitor;

#[test]
fn test_monitor_refresh_and_data_collection() {
    let mut monitor = Monitor::new();
    let snap = monitor.refresh();

    // CPU data collected
    assert!(snap.cpu.global_usage >= 0.0);
    assert!(
        !snap.cpu.per_core.is_empty(),
        "Per-core CPU should not be empty"
    );

    // Memory data collected
    assert!(snap.mem.total > 0, "Total RAM should be greater than 0");
    assert!(snap.mem.used <= snap.mem.total);
    assert!(snap.mem.used_pct() >= 0.0 && snap.mem.used_pct() <= 100.0);

    // Processes list collected
    assert!(
        !snap.processes.is_empty(),
        "Processes list should not be empty"
    );
    let p = &snap.processes[0];
    assert!(p.pid > 0);
    assert!(!p.name.is_empty());
}
