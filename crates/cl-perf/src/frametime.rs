//! Frame-time capture analysis (real-condition test).
//!
//! Accepted CSV formats:
//! - MangoHud logs (Linux): a system-info header, then a table with a
//!   `frametime` column (milliseconds).
//! - PresentMon (Windows): `MsBetweenPresents` (1.x and 2.x with
//!   `--v1_metrics`) or `FrameTime` (2.x) column.
//! - Any CSV with one of those columns, or a single column of numbers.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("cannot read capture: {0}")]
    Io(#[from] std::io::Error),
    #[error("no frame-time column found")]
    NoColumn,
    #[error("capture too short ({0} frames)")]
    TooShort(usize),
    #[error("invalid target refresh rate")]
    InvalidTarget,
}

const COLUMNS: [&str; 4] = [
    "frametime",
    "msbetweenpresents",
    "frametime(ms)",
    "msbetweendisplaychange",
];
/// Frames ignored at the start (loading, shader warm-up of the first second).
const SKIP_FRAMES: usize = 60;
const MIN_FRAMES: usize = 300;

pub fn parse_csv(text: &str) -> Result<Vec<f64>, CaptureError> {
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    // Find the header line holding a known column.
    let header = lines.iter().enumerate().find_map(|(i, l)| {
        let cols: Vec<String> = l
            .split(',')
            .map(|c| c.trim().to_ascii_lowercase())
            .collect();
        COLUMNS
            .iter()
            .find_map(|name| cols.iter().position(|c| c == name))
            .map(|col| (i, col))
    });
    let values: Vec<f64> = match header {
        Some((row, col)) => lines[row + 1..]
            .iter()
            .filter_map(|l| l.split(',').nth(col))
            .filter_map(|v| v.trim().parse::<f64>().ok())
            .collect(),
        None => {
            let v: Vec<f64> = lines.iter().filter_map(|l| l.parse::<f64>().ok()).collect();
            if v.is_empty() {
                return Err(CaptureError::NoColumn);
            }
            v
        }
    };
    let mut values: Vec<f64> = values
        .into_iter()
        .filter(|v| v.is_finite() && *v > 0.0)
        .collect();
    // Some tools log microseconds: a median above one second means µs.
    if median(&values) > 1000.0 {
        values.iter_mut().for_each(|v| *v /= 1000.0);
    }
    Ok(values)
}

pub fn parse_file(path: &Path) -> Result<Vec<f64>, CaptureError> {
    parse_csv(&std::fs::read_to_string(path)?)
}

fn median(v: &[f64]) -> f64 {
    percentile(v, 50.0)
}

fn percentile(v: &[f64], p: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.total_cmp(b));
    let idx = ((p / 100.0) * (s.len() - 1) as f64).round() as usize;
    s[idx.min(s.len() - 1)]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameStats {
    pub frames: usize,
    pub seconds: f64,
    pub avg_fps: f64,
    /// FPS matching the 99th percentile frame time ("1 % low").
    pub low1_fps: f64,
    /// Output refresh of the emulated console (60 NTSC or 50 PAL), deduced
    /// from the median. Measurements are taken with every refresh presented
    /// (see `cl-session`), so a game running at half speed shows up as
    /// stutter instead of being mistaken for a 30 fps game.
    pub target_fps: f64,
    /// Frames lasting more than 1.5 × the target frame time.
    pub stutters: usize,
    pub stutter_pct: f64,
}

pub fn analyse(frametimes_ms: &[f64]) -> Result<FrameStats, CaptureError> {
    analyse_inner(frametimes_ms, None)
}

pub fn analyse_with_target(frametimes_ms: &[f64], target: f64) -> Result<FrameStats, CaptureError> {
    if !target.is_finite() || target <= 0.0 {
        return Err(CaptureError::InvalidTarget);
    }
    analyse_inner(frametimes_ms, Some(target))
}

fn analyse_inner(frametimes_ms: &[f64], target: Option<f64>) -> Result<FrameStats, CaptureError> {
    let ft = if frametimes_ms.len() > SKIP_FRAMES + MIN_FRAMES {
        &frametimes_ms[SKIP_FRAMES..]
    } else {
        frametimes_ms
    };
    if ft.len() < MIN_FRAMES {
        return Err(CaptureError::TooShort(ft.len()));
    }
    let total_ms: f64 = ft.iter().sum();
    let median_fps = 1000.0 / median(ft);
    let target_fps = target.unwrap_or_else(|| {
        [60.0, 50.0]
            .into_iter()
            .min_by(|a: &f64, b: &f64| (a - median_fps).abs().total_cmp(&(b - median_fps).abs()))
            .unwrap_or(60.0)
    });
    let budget = 1000.0 / target_fps * 1.5;
    let stutters = ft.iter().filter(|&&t| t > budget).count();
    Ok(FrameStats {
        frames: ft.len(),
        seconds: total_ms / 1000.0,
        avg_fps: ft.len() as f64 / (total_ms / 1000.0),
        low1_fps: 1000.0 / percentile(ft, 99.0),
        target_fps,
        stutters,
        stutter_pct: stutters as f64 * 100.0 / ft.len() as f64,
    })
}

/// Smoothness verdict for one session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Smooth,
    Stutters,
}

/// Pass criteria. Initial values, to confirm with testers.
pub fn verdict(s: &FrameStats) -> Verdict {
    let ok = s.avg_fps >= s.target_fps * 0.97
        && s.low1_fps >= s.target_fps * 0.85
        && s.stutter_pct < 1.0;
    if ok {
        Verdict::Smooth
    } else {
        Verdict::Stutters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(n: usize, ms: f64) -> Vec<f64> {
        vec![ms; n]
    }

    #[test]
    fn parses_mangohud_log() {
        let mut text =
            String::from("os,cpu,gpu,ram\nLinux,Ryzen,RX,16\nfps,frametime,cpu_load,gpu_load\n");
        for _ in 0..10 {
            text.push_str("60,16.67,30,40\n");
        }
        let v = parse_csv(&text).unwrap();
        assert_eq!(v.len(), 10);
        assert!((v[0] - 16.67).abs() < 1e-9);
    }

    #[test]
    fn parses_presentmon_and_microseconds() {
        let text = "Application,ProcessID,MsBetweenPresents\napp.exe,1,16.6\napp.exe,1,16.7\n";
        assert_eq!(parse_csv(text).unwrap(), vec![16.6, 16.7]);
        let us = "16666\n16700\n";
        let v = parse_csv(us).unwrap();
        assert!((v[0] - 16.666).abs() < 1e-6);
    }

    #[test]
    fn smooth_60fps_passes() {
        let s = analyse(&series(3_600, 16.67)).unwrap();
        assert_eq!(s.target_fps, 60.0);
        assert_eq!(verdict(&s), Verdict::Smooth);
    }

    #[test]
    fn stutters_are_detected() {
        let mut v = series(3_600, 16.67);
        for i in (0..v.len()).step_by(50) {
            v[i] = 50.0; // 2 % of frames hitch
        }
        let s = analyse(&v).unwrap();
        assert!(s.stutter_pct > 1.0);
        assert_eq!(verdict(&s), Verdict::Stutters);
    }

    #[test]
    fn half_speed_is_not_mistaken_for_a_30fps_game() {
        let s = analyse(&series(2_000, 33.3)).unwrap();
        assert_eq!(s.target_fps, 50.0);
        assert_eq!(verdict(&s), Verdict::Stutters);
    }

    #[test]
    fn pal_game_is_judged_against_fifty() {
        let s = analyse(&series(2_000, 20.0)).unwrap();
        assert_eq!(s.target_fps, 50.0);
        assert_eq!(verdict(&s), Verdict::Smooth);
    }

    #[test]
    fn too_short_capture_is_rejected() {
        assert!(matches!(
            analyse(&series(100, 16.0)),
            Err(CaptureError::TooShort(_))
        ));
    }

    #[test]
    fn known_ntsc_rate_cannot_be_reclassified_as_pal_when_running_slow() {
        let s = analyse_with_target(&series(2000, 20.0), 60.0).unwrap();
        assert_eq!(s.target_fps, 60.0);
        assert_eq!(verdict(&s), Verdict::Stutters);
        let s = analyse_with_target(&series(2000, 1000.0 / 59.7275), 59.7275).unwrap();
        assert_eq!(verdict(&s), Verdict::Smooth);
        assert!(analyse_with_target(&series(2000, 16.67), f64::NAN).is_err());
    }
}
