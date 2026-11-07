//! Disk monitoring.

use sysinfo::Disks;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount_point: String,
    pub name: String,
    pub total: u64,
    pub available: u64,
    pub file_system: String,
}

impl DiskInfo {
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    pub fn used_pct(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.used() as f64 / self.total as f64 * 100.0
        }
    }
}

pub fn collect() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    disks
        .iter()
        .map(|d| DiskInfo {
            mount_point: d.mount_point().to_string_lossy().to_string(),
            name: d.name().to_string_lossy().to_string(),
            total: d.total_space(),
            available: d.available_space(),
            file_system: d.file_system().to_string_lossy().to_string(),
        })
        .collect()
}
