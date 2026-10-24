//! Stability certification built from independent runtime signals.
//!
//! A smooth frame-time graph alone is useful, but it cannot prove that an
//! emulator ran at full speed or that its audio buffer stayed healthy.  The
//! certification therefore stays `Insufficient` until all three signals are
//! available.  Any known failure is still reported immediately.

use crate::frametime::{verdict, FrameStats, Verdict};
use serde::{Deserialize, Serialize};

const MIN_EMULATION_SPEED_PCT: f64 = 99.5;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CertificationVerdict {
    Stable,
    Unstable,
    #[default]
    Insufficient,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StabilityAssessment {
    pub verdict: CertificationVerdict,
    #[serde(default)]
    pub frame_verdict: Option<Verdict>,
    #[serde(default)]
    pub emulation_speed_pct: Option<f64>,
    #[serde(default)]
    pub audio_underruns: Option<u32>,
    #[serde(default)]
    pub reasons: Vec<String>,
}

pub fn assess(
    frame: Option<&FrameStats>,
    emulation_speed_pct: Option<f64>,
    audio_underruns: Option<u32>,
) -> StabilityAssessment {
    let frame_verdict = frame.map(verdict);
    let mut reasons = Vec::new();
    let mut failed = false;

    match frame_verdict {
        Some(Verdict::Smooth) => {}
        Some(Verdict::Stutters) => {
            failed = true;
            reasons.push("frame pacing is outside the stability target".into());
        }
        None => reasons.push("frame pacing was not measured".into()),
    }

    match emulation_speed_pct {
        Some(speed) if speed < MIN_EMULATION_SPEED_PCT => {
            failed = true;
            reasons.push(format!(
                "emulation speed is {speed:.1}% (minimum {MIN_EMULATION_SPEED_PCT:.1}%)"
            ));
        }
        Some(_) => {}
        None => reasons.push("actual emulation speed was not measured".into()),
    }

    match audio_underruns {
        Some(count) if count > 0 => {
            failed = true;
            reasons.push(format!("audio buffer underruns detected: {count}"));
        }
        Some(_) => {}
        None => reasons.push("audio buffer health was not measured".into()),
    }

    let complete =
        frame_verdict.is_some() && emulation_speed_pct.is_some() && audio_underruns.is_some();
    let verdict = if failed {
        CertificationVerdict::Unstable
    } else if complete {
        CertificationVerdict::Stable
    } else {
        CertificationVerdict::Insufficient
    };

    StabilityAssessment {
        verdict,
        frame_verdict,
        emulation_speed_pct,
        audio_underruns,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn smooth_stats() -> FrameStats {
        FrameStats {
            frames: 3_600,
            seconds: 60.0,
            avg_fps: 60.0,
            low1_fps: 59.0,
            target_fps: 60.0,
            stutters: 0,
            stutter_pct: 0.0,
        }
    }

    #[test]
    fn frame_only_is_not_enough_to_claim_stable() {
        let result = assess(Some(&smooth_stats()), None, None);
        assert_eq!(result.verdict, CertificationVerdict::Insufficient);
        assert_eq!(result.reasons.len(), 2);
    }

    #[test]
    fn every_signal_must_be_healthy() {
        let result = assess(Some(&smooth_stats()), Some(100.0), Some(0));
        assert_eq!(result.verdict, CertificationVerdict::Stable);
        assert!(result.reasons.is_empty());
    }

    #[test]
    fn a_known_failure_wins_over_missing_signals() {
        let mut stats = smooth_stats();
        stats.stutter_pct = 2.0;
        stats.stutters = 72;
        let result = assess(Some(&stats), None, None);
        assert_eq!(result.verdict, CertificationVerdict::Unstable);
    }

    #[test]
    fn slow_emulation_or_audio_underruns_fail() {
        assert_eq!(
            assess(Some(&smooth_stats()), Some(98.0), Some(0)).verdict,
            CertificationVerdict::Unstable
        );
        assert_eq!(
            assess(Some(&smooth_stats()), Some(100.0), Some(1)).verdict,
            CertificationVerdict::Unstable
        );
    }
}
