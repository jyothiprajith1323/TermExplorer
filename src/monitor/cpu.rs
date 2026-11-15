//! CPU monitoring.

use sysinfo::System;

#[derive(Debug, Clone, Default)]
pub struct CpuInfo {
    pub global_usage: f32,
    pub per_core: Vec<f32>,
    pub cpu_name: String,
    pub load_avg: (f64, f64, f64),
}

pub fn collect(sys: &mut System) -> CpuInfo {
    sys.refresh_cpu_all();

    let cpus = sys.cpus();
    let global = sys.global_cpu_usage();
    let per_core: Vec<f32> = cpus.iter().map(|c| c.usage()).collect();
    let name = cpus
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_default();

    let (one, five, fifteen) = match System::load_average() {
        Ok(l) => (l.one, l.five, l.fifteen),
        Err(_) => (0.0, 0.0, 0.0),
    };
    CpuInfo {
        global_usage: global,
        per_core,
        cpu_name: name,
        load_avg: (one, five, fifteen),
    }
}
