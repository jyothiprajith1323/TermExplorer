//! Process monitoring.

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_kb: u64,
}

pub fn collect(sys: &mut System, limit: usize) -> Vec<ProcessInfo> {
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );

    let mut procs: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, p)| ProcessInfo {
            pid: pid.as_u32(),
            name: p.name().to_string_lossy().to_string(),
            cpu_usage: p.cpu_usage(),
            memory_kb: p.memory() / 1024,
        })
        .collect();

    // Sort by CPU descending.
    procs.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    procs.truncate(limit);
    procs
}
