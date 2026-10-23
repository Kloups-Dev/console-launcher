//! Shared building blocks for Console Launcher.
//!
//! This crate holds the data model used by every other crate (systems,
//! games, hardware tiers, quality profiles), the user settings file and a
//! small INI editor that only touches the keys the launcher owns.

pub mod config;
pub mod ini;
pub mod model;
pub mod paths;
pub mod watch;

pub use model::*;
