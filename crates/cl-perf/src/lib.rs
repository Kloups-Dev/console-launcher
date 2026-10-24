//! Smoothness: estimate, measure, adjust.
//!
//! 1. **Simulation** ([`estimate`]): before the first launch, the hardware
//!    report and the CPU micro-benchmark give a starting rung on the
//!    adapter's quality ladder.
//! 2. **Real conditions** ([`frametime`]): during a session a frame-time
//!    capture (MangoHud on Linux, PresentMon on Windows) is analysed.
//! 3. **Adjustment** ([`tuner`]): the result moves the game one rung down
//!    when it stutters, or one rung up to explore while it stays smooth,
//!    until the best smooth rung is found.

pub mod estimate;
pub mod frametime;
pub mod store;
pub mod tuner;
