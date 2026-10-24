//! Smoothness: estimate, measure, adjust.
//!
//! 1. **Simulation** ([`estimate`]): before the first launch, the hardware
//!    report and the CPU micro-benchmark give a starting rung on the
//!    adapter's quality ladder.
//! 2. **Real conditions** ([`frametime`]): during a session a frame-time
//!    capture (MangoHud on Linux, PresentMon on Windows) is analysed.
//! 3. **Certification** ([`stability`]): frame pacing, actual emulation speed
//!    and audio health must all pass before a profile is called stable.
//! 4. **Adjustment** ([`tuner`]): failures move the game one rung down. A
//!    successful conservative rung is kept; quality is never raised silently.

pub mod estimate;
pub mod frametime;
pub mod stability;
pub mod store;
pub mod tuner;
