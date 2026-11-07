//! Monitor module — aggregates system resource data.

pub mod cpu;
pub mod disk;
pub mod memory;
pub mod processes;

use sysinfo::System;

/// Aggregated snapshot of all monitored resources.
#[derive(Debug, Clone, Default)]
pub struct MonitorSnapshot {
    pub cpu: cpu::CpuInfo,
    pub mem: memory::MemInfo,
    pub disks: Vec<disk::DiskInfo>,
    pub processes: Vec<processes::ProcessInfo>,
}

pub struct Monitor {
    sys: System,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
        }
    }

    pub fn refresh(&mut self) -> MonitorSnapshot {
        let cpu = cpu::collect(&mut self.sys);
        let mem = memory::collect(&mut self.sys);
        let disks = disk::collect();
        let procs = processes::collect(&mut self.sys, 20);
        MonitorSnapshot {
            cpu,
            mem,
            disks,
            processes: procs,
        }
    }
}

impl Default for Monitor {
    fn default() -> Self {
        Self::new()
    }
}
