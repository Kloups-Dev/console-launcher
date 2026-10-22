//! Hardware detection.
//!
//! - CPU and RAM through `sysinfo`.
//! - GPUs through the Vulkan loader (`ash`, loaded at run time, so the
//!   program still starts on a machine without Vulkan).
//! - A short single-thread micro-benchmark that runs a tiny bytecode
//!   interpreter, the kind of work CPU emulation spends most of its time on.

mod bench;
mod gpu;

pub use bench::{run_cpu_bench, BenchResult};
pub use gpu::{GpuInfo, GpuKind};

use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub brand: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub max_mhz: u64,
    pub avx2: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareReport {
    pub os: String,
    pub cpu: CpuInfo,
    pub ram_mb: u64,
    pub gpus: Vec<GpuInfo>,
    pub bench: Option<BenchResult>,
}

impl HardwareReport {
    /// The GPU the emulators will most likely use: discrete first, then the
    /// one with the most dedicated memory.
    pub fn main_gpu(&self) -> Option<&GpuInfo> {
        self.gpus.iter().max_by_key(|g| (g.kind.rank(), g.vram_mb))
    }
}

fn has_avx2() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        std::arch::is_x86_feature_detected!("avx2")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

pub fn detect_cpu() -> (CpuInfo, u64) {
    let mut sys = System::new();
    sys.refresh_cpu_all();
    sys.refresh_memory();
    let cpus = sys.cpus();
    let cpu = CpuInfo {
        brand: cpus
            .first()
            .map(|c| c.brand().trim().to_owned())
            .unwrap_or_default(),
        physical_cores: System::physical_core_count().unwrap_or(cpus.len()),
        logical_cores: cpus.len(),
        max_mhz: cpus.iter().map(|c| c.frequency()).max().unwrap_or(0),
        avx2: has_avx2(),
    };
    (cpu, sys.total_memory() / (1024 * 1024))
}

/// Full detection. `with_bench` runs the micro-benchmark (about one second).
pub fn probe(with_bench: bool) -> HardwareReport {
    let (cpu, ram_mb) = detect_cpu();
    let gpus = gpu::detect_gpus().unwrap_or_else(|e| {
        log::warn!("GPU detection through Vulkan failed: {e}");
        Vec::new()
    });
    HardwareReport {
        os: format!(
            "{} {}",
            std::env::consts::OS,
            System::os_version().unwrap_or_default()
        ),
        cpu,
        ram_mb,
        gpus,
        bench: with_bench.then(run_cpu_bench),
    }
}
