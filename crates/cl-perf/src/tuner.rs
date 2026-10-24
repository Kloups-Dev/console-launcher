//! Per-game adjustment from measured sessions.
//!
//! Stability-first policy: keep a certified rung, descend after a failure,
//! and never raise quality automatically. Higher rungs remain available as
//! an explicit user choice.

use crate::stability::CertificationVerdict;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuneState {
    /// Rung to use at the next launch.
    pub current: usize,
    pub max: usize,
    pub best_smooth: Option<usize>,
    pub lowest_failing: Option<usize>,
    pub done: bool,
    /// True only when frame pacing, emulation speed and audio all passed.
    #[serde(default)]
    pub certified: bool,
    /// True when even the lightest rung stutters: the machine is too slow
    /// for this game and the interface should say so.
    pub below_requirements: bool,
}

impl TuneState {
    pub fn new(start: usize, ladder_len: usize) -> TuneState {
        let max = ladder_len.saturating_sub(1);
        TuneState {
            current: start.min(max),
            max,
            best_smooth: None,
            lowest_failing: None,
            done: false,
            certified: false,
            below_requirements: false,
        }
    }

    /// Records the verdict of a session played at `self.current`.
    pub fn record(&mut self, verdict: CertificationVerdict) {
        let cur = self.current;
        match verdict {
            CertificationVerdict::Stable => {
                self.best_smooth = Some(self.best_smooth.map_or(cur, |b| b.max(cur)));
                self.below_requirements = false;
                self.certified = true;
                self.done = true;
            }
            CertificationVerdict::Unstable => {
                self.lowest_failing = Some(self.lowest_failing.map_or(cur, |f| f.min(cur)));
                self.certified = false;
                if cur > 0 {
                    self.current = cur - 1;
                    self.done = false;
                } else {
                    self.done = true;
                    self.below_requirements = true;
                }
            }
            CertificationVerdict::Insufficient => {
                self.certified = false;
                self.done = false;
            }
        }
    }

    /// The user changed the ladder position by hand: stop searching.
    pub fn set_by_user(&mut self, step: usize) {
        self.current = step.min(self.max);
        self.done = true;
        self.certified = false;
    }

    /// Hardware or emulator version changed: search again from the current rung.
    pub fn reset_search(&mut self) {
        self.best_smooth = None;
        self.lowest_failing = None;
        self.done = false;
        self.certified = false;
        self.below_requirements = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use CertificationVerdict::*;

    #[test]
    fn stable_never_climbs() {
        let mut t = TuneState::new(1, 5);
        t.record(Stable);
        assert_eq!((t.current, t.done, t.certified), (1, true, true));
    }

    #[test]
    fn descends_when_estimate_was_too_optimistic() {
        let mut t = TuneState::new(4, 5);
        t.record(Unstable);
        assert_eq!(t.current, 3);
        t.record(Unstable);
        t.record(Stable);
        assert_eq!((t.current, t.done, t.certified), (2, true, true));
    }

    #[test]
    fn insufficient_evidence_keeps_the_same_step_open() {
        let mut t = TuneState::new(2, 5);
        t.record(Insufficient);
        assert_eq!((t.current, t.done, t.certified), (2, false, false));
    }

    #[test]
    fn flags_machines_below_requirements() {
        let mut t = TuneState::new(0, 5);
        t.record(Unstable);
        assert!(t.done && t.below_requirements);
        assert_eq!(t.current, 0);
    }

    #[test]
    fn manual_choice_is_not_a_fake_certification() {
        let mut t = TuneState::new(0, 5);
        t.set_by_user(3);
        assert_eq!((t.current, t.done, t.certified), (3, true, false));
    }
}
