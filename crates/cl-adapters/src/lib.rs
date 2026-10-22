//! Emulator adapters.
//!
//! An adapter knows how to find one emulator, which systems it runs, the
//! "quality ladder" of settings it offers for each system, and how to build
//! the command that starts a game directly (no emulator menu).
//!
//! Rule for the user's own files:
//! - **External** installs (put there by the user) are never edited.
//!   Settings go through command-line overrides when the emulator has them
//!   (RetroArch `--appendconfig`, Dolphin `--config`). Otherwise the
//!   profile is not applied and the plan says so.
//! - **Managed** installs (installed by the launcher in its data folder)
//!   can have their INI files edited, through [`cl_core::ini::apply_owned`],
//!   which keeps every value the user changed by hand.

mod dolphin;
mod pcsx2;
mod retroarch;

use cl_core::config::{EmulatorConfig, EmulatorMode};
use cl_core::ini::ApplyReport;
use cl_core::{Game, GraphicsApi, QualityStep, SystemId};
use serde::Serialize;
use std::path::{Path, PathBuf};

pub use dolphin::Dolphin;
pub use pcsx2::Pcsx2;
pub use retroarch::RetroArch;

#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
}

/// Everything an adapter needs to prepare a launch.
#[derive(Debug, Clone)]
pub struct LaunchContext<'a> {
    pub exe: &'a Path,
    pub game: &'a Game,
    pub step: &'a QualityStep,
    pub api: GraphicsApi,
    pub mode: EmulatorMode,
    /// Folder where the adapter may write override files and ledgers.
    pub generated_dir: &'a Path,
    /// Keys the user locked; never changed.
    pub locked_keys: &'a [String],
    /// Folder of BIOS/firmware files provided by the user, if any.
    pub bios_dir: Option<&'a Path>,
}

/// The command to run and what was done to prepare it.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LaunchPlan {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    /// Process name to watch for frame capture (may differ from `program`
    /// when a wrapper is used).
    pub process_name: String,
    pub settings_report: ApplyReport,
    /// Human-readable remarks (missing core, settings not applied, …).
    pub notes: Vec<String>,
}

pub trait EmulatorAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    /// SPDX licence of the emulator itself, for the credits screen.
    fn license(&self) -> &'static str;
    fn homepage(&self) -> &'static str;
    fn systems(&self) -> &'static [SystemId];
    /// Graphics back-ends, best first for the current OS.
    fn apis(&self) -> Vec<GraphicsApi>;
    /// Executable names to look for, per OS.
    fn exe_names(&self) -> &'static [&'static str];
    /// Usual install locations to probe when no path is configured.
    fn common_paths(&self) -> Vec<PathBuf> {
        Vec::new()
    }
    /// Quality ladder for a system, lightest first.
    fn quality_ladder(&self, system: SystemId) -> Vec<QualityStep>;
    fn prepare(&self, ctx: &LaunchContext) -> Result<LaunchPlan, AdapterError>;

    /// True when the ladder raises rendering resolution (GPU-bound),
    /// false when it trades CPU time for latency.
    fn gpu_bound_ladder(&self) -> bool {
        true
    }

    /// Extra arguments for measured sessions, so that every console
    /// refresh is presented (otherwise a 30 fps game and a game running at
    /// half speed look the same in a capture).
    fn measurement_args(&self) -> Vec<String> {
        Vec::new()
    }

    /// Finds the executable: configured path, managed install folder
    /// (given by the updater), PATH, then usual locations.
    fn find_executable(
        &self,
        cfg: &EmulatorConfig,
        managed_install: Option<&Path>,
    ) -> Option<PathBuf> {
        if let Some(p) = &cfg.path {
            return p.is_file().then(|| p.clone());
        }
        if cfg.mode == EmulatorMode::Managed {
            return managed_install.and_then(|dir| find_in_tree(dir, self.exe_names()));
        }
        find_in_path(self.exe_names())
            .or_else(|| self.common_paths().into_iter().find(|p| p.is_file()))
    }
}

fn find_in_path(names: &[&str]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |n| dir.join(n)))
        .find(|p| p.is_file())
}

/// Looks for one of `names` in `dir` and up to two levels below
/// (release archives often contain a top folder).
pub fn find_in_tree(dir: &Path, names: &[&str]) -> Option<PathBuf> {
    fn walk(dir: &Path, names: &[&str], depth: u8) -> Option<PathBuf> {
        for n in names {
            let p = dir.join(n);
            if p.is_file() {
                return Some(p);
            }
        }
        if depth == 0 {
            return None;
        }
        let mut subdirs: Vec<_> = std::fs::read_dir(dir)
            .ok()?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        subdirs.sort();
        subdirs.into_iter().find_map(|d| walk(&d, names, depth - 1))
    }
    walk(dir, names, 2)
}

/// All adapters known to this version.
pub fn registry() -> Vec<Box<dyn EmulatorAdapter>> {
    vec![Box::new(RetroArch), Box::new(Dolphin), Box::new(Pcsx2)]
}

/// Default emulator per system. Users can override it in `config.toml`.
pub fn default_emulator(system: SystemId) -> &'static str {
    match system {
        SystemId::GameCube | SystemId::Wii => "dolphin",
        SystemId::Ps2 => "pcsx2",
        _ => "retroarch",
    }
}

pub fn by_id(id: &str) -> Option<Box<dyn EmulatorAdapter>> {
    registry().into_iter().find(|a| a.id() == id)
}

pub(crate) fn exe_stem(p: &Path) -> String {
    p.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub(crate) fn step(label: &str, pairs: &[(&str, &str)]) -> QualityStep {
    QualityStep {
        label: label.into(),
        settings: pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    }
}

#[cfg(test)]
pub(crate) mod testutil {
    use super::*;
    pub fn game(system: SystemId, path: &str) -> Game {
        Game {
            id: "x".into(),
            system,
            title: "T".into(),
            path: path.into(),
            cover: None,
            favorite: false,
        }
    }
}
