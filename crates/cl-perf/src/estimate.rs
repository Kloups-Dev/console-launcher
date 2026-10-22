//! Starting point from hardware data ("simulation" test).
//!
//! The thresholds below are **initial estimates**. They have not been
//! measured yet; docs/prototype/PERFORMANCE.*.md describes how to calibrate
//! them from real captures. The tuner corrects any wrong guess after the
//! first measured session.

use cl_core::{GraphicsApi, SystemId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GpuClass {
    /// No usable GPU detected (software renderer, VM…).
    None,
    Integrated,
    Discrete,
}

/// Hardware facts the estimator needs (filled from `cl-hwprobe`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HwSummary {
    /// Micro-benchmark score, 1000 = reference machine.
    pub cpu_score: u32,
    pub gpu_class: GpuClass,
    pub vram_mb: u64,
    pub has_vulkan: bool,
    pub ram_mb: u64,
}

/// What the ladder trades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LadderKind {
    /// Rungs raise the rendering resolution: limited by the GPU.
    Resolution,
    /// Rungs lower input latency (run-ahead): limited by the CPU.
    Latency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estimate {
    pub step: usize,
    pub api: GraphicsApi,
    /// CPU score divided by the score needed for full speed.
    pub cpu_headroom: f32,
    pub warnings: Vec<String>,
}

/// CPU score assumed necessary for full speed, by system cost (1–10).
fn required_cpu(cost: u8) -> f32 {
    match cost {
        0..=1 => 150.0,
        2 => 250.0,
        3 => 350.0,
        4 => 450.0,
        5 | 6 => 600.0,
        7 => 750.0,
        _ => 900.0,
    }
}

/// GPU budget in "resolution rungs" (0 = native only).
fn gpu_budget(hw: &HwSummary) -> usize {
    match hw.gpu_class {
        GpuClass::None => 0,
        GpuClass::Integrated => 1,
        GpuClass::Discrete => match hw.vram_mb {
            0..=3_071 => 1,
            3_072..=5_119 => 2,
            5_120..=9_215 => 3,
            _ => 4,
        },
    }
}

pub fn pick_api(available: &[GraphicsApi], hw: &HwSummary) -> GraphicsApi {
    available
        .iter()
        .copied()
        .find(|a| *a != GraphicsApi::Vulkan || hw.has_vulkan)
        .unwrap_or(GraphicsApi::OpenGl)
}

pub fn estimate(
    hw: &HwSummary,
    system: SystemId,
    kind: LadderKind,
    ladder_len: usize,
    apis: &[GraphicsApi],
) -> Estimate {
    let cost = system.info().cost;
    let headroom = hw.cpu_score as f32 / required_cpu(cost);
    let max = ladder_len.saturating_sub(1);
    let mut warnings = Vec::new();
    if headroom < 1.0 {
        warnings.push(format!(
            "The processor may be too slow for {} at full speed (estimate: {:.0} %).",
            system.info().name,
            headroom * 100.0
        ));
    }
    if hw.gpu_class == GpuClass::None {
        warnings.push("No hardware graphics acceleration detected.".into());
    }
    let step = if headroom < 1.0 {
        0
    } else {
        match kind {
            LadderKind::Latency => match headroom {
                h if h >= 4.0 => 3,
                h if h >= 2.5 => 2,
                h if h >= 1.3 => 1,
                _ => 0,
            },
            LadderKind::Resolution => {
                // Heavy systems cost more GPU per rung.
                let budget = gpu_budget(hw).saturating_sub(usize::from(cost >= 8));
                // A CPU close to the limit leaves no margin for the extra
                // work some upscaled effects add on the CPU side.
                if headroom < 1.2 {
                    budget.min(1)
                } else {
                    budget
                }
            }
        }
    }
    .min(max);
    Estimate {
        step,
        api: pick_api(apis, hw),
        cpu_headroom: headroom,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hw(cpu: u32, class: GpuClass, vram: u64) -> HwSummary {
        HwSummary {
            cpu_score: cpu,
            gpu_class: class,
            vram_mb: vram,
            has_vulkan: true,
            ram_mb: 16_000,
        }
    }
    const APIS: [GraphicsApi; 2] = [GraphicsApi::Vulkan, GraphicsApi::OpenGl];

    #[test]
    fn strong_pc_gets_high_resolution() {
        let e = estimate(
            &hw(1500, GpuClass::Discrete, 12_288),
            SystemId::Ps2,
            LadderKind::Resolution,
            5,
            &APIS,
        );
        assert_eq!(e.step, 4);
        assert!(e.warnings.is_empty());
    }

    #[test]
    fn weak_cpu_falls_back_to_native_with_warning() {
        let e = estimate(
            &hw(400, GpuClass::Discrete, 12_288),
            SystemId::Wii,
            LadderKind::Resolution,
            5,
            &APIS,
        );
        assert_eq!(e.step, 0);
        assert_eq!(e.warnings.len(), 1);
    }

    #[test]
    fn integrated_gpu_stays_low() {
        let e = estimate(
            &hw(1200, GpuClass::Integrated, 2_048),
            SystemId::GameCube,
            LadderKind::Resolution,
            5,
            &APIS,
        );
        assert_eq!(e.step, 1);
    }

    #[test]
    fn latency_ladder_follows_cpu() {
        let e = estimate(
            &hw(700, GpuClass::Integrated, 0),
            SystemId::Snes,
            LadderKind::Latency,
            4,
            &APIS,
        );
        assert_eq!(e.step, 3);
        let e = estimate(
            &hw(250, GpuClass::Integrated, 0),
            SystemId::Snes,
            LadderKind::Latency,
            4,
            &APIS,
        );
        assert_eq!(e.step, 1);
    }

    #[test]
    fn no_vulkan_picks_next_api() {
        let mut h = hw(1000, GpuClass::Discrete, 8_000);
        h.has_vulkan = false;
        assert_eq!(pick_api(&APIS, &h), GraphicsApi::OpenGl);
    }
}
