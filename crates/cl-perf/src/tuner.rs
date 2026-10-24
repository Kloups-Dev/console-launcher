//! Per-game adjustment from measured sessions.
//!
//! Search: start from the estimate. After each measured session:
//! - smooth → remember it as good, then try the next rung up
//!   (unless that rung already failed);
//! - stutters → remember the failure, go back to the last good rung,
//!   or one rung down if none is known.
//!
//! The search ends when a smooth rung sits right below a failing rung (or
//! at the top of the ladder). It never needs more sessions than the
//! ladder has rungs.

use crate::frametime::Verdict;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuneState {
    /// Rung to use at the next launch.
    pub current: usize,
    pub max: usize,
    pub best_smooth: Option<usize>,
    pub lowest_failing: Option<usize>,
    pub done: bool,
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
            below_requirements: false,
        }
    }

    /// Records the verdict of a session played at `self.current`.
    pub fn record(&mut self, verdict: Verdict) {
        let cur = self.current;
        match verdict {
            Verdict::Smooth => {
                self.best_smooth = Some(self.best_smooth.map_or(cur, |b| b.max(cur)));
                self.below_requirements = false;
                let ceiling = self.lowest_failing.unwrap_or(self.max + 1);
                if cur + 1 < ceiling && cur < self.max {
                    self.current = cur + 1;
                    self.done = false;
                } else {
                    self.done = true;
                }
            }
            Verdict::Stutters => {
                self.lowest_failing = Some(self.lowest_failing.map_or(cur, |f| f.min(cur)));
                if let Some(b) = self.best_smooth.filter(|b| *b < cur) {
                    self.current = b;
                    self.done = true;
                } else if cur > 0 {
                    self.current = cur - 1;
                    self.done = false;
                } else {
                    self.done = true;
                    self.below_requirements = true;
                }
            }
        }
    }

    /// The user changed the ladder position by hand: stop searching.
    pub fn set_by_user(&mut self, step: usize) {
        self.current = step.min(self.max);
        self.done = true;
    }

    /// Hardware or emulator version changed: search again from the current rung.
    pub fn reset_search(&mut self) {
        self.best_smooth = None;
        self.lowest_failing = None;
        self.done = false;
        self.below_requirements = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Verdict::*;

    #[test]
    fn climbs_until_first_failure() {
        let mut t = TuneState::new(1, 5);
        t.record(Smooth); // 1 ok → try 2
        assert_eq!((t.current, t.done), (2, false));
        t.record(Smooth); // 2 ok → try 3
        t.record(Stutters); // 3 fails → back to 2
        assert_eq!((t.current, t.done), (2, true));
    }

    #[test]
    fn descends_when_estimate_was_too_optimistic() {
        let mut t = TuneState::new(4, 5);
        t.record(Stutters);
        assert_eq!(t.current, 3);
        t.record(Stutters);
        t.record(Smooth); // 2 ok, 3 known to fail → done
        assert_eq!((t.current, t.done), (2, true));
    }

    #[test]
    fn stops_at_top_of_ladder() {
        let mut t = TuneState::new(4, 5);
        t.record(Smooth);
        assert_eq!((t.current, t.done), (4, true));
    }

    #[test]
    fn flags_machines_below_requirements() {
        let mut t = TuneState::new(0, 5);
        t.record(Stutters);
        assert!(t.done && t.below_requirements);
        assert_eq!(t.current, 0);
    }

    #[test]
    fn never_exceeds_ladder_length_in_sessions() {
        for start in 0..5 {
            for fail_from in 0..=5 {
                let mut t = TuneState::new(start, 5);
                let mut sessions = 0;
                while !t.done {
                    let v = if t.current >= fail_from {
                        Stutters
                    } else {
                        Smooth
                    };
                    t.record(v);
                    sessions += 1;
                    assert!(sessions <= 5, "start {start} fail_from {fail_from}");
                }
                let expected = fail_from.saturating_sub(1).min(4);
                assert_eq!(t.current, expected, "start {start} fail_from {fail_from}");
            }
        }
    }
}
