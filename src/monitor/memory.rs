//! Memory monitoring.

use sysinfo::System;

#[derive(Debug, Clone, Default)]
pub struct MemInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

impl MemInfo {
    pub fn used_pct(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.used as f64 / self.total as f64 * 100.0
        }
    }

    pub fn swap_pct(&self) -> f64 {
        if self.swap_total == 0 {
            0.0
        } else {
            self.swap_used as f64 / self.swap_total as f64 * 100.0
        }
    }
}

pub fn collect(sys: &mut System) -> MemInfo {
    sys.refresh_memory();
    MemInfo {
        total: sys.total_memory(),
        used: sys.used_memory(),
        available: sys.available_memory(),
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
    }
}
