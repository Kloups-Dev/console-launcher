//! Emulator adapters.
//!
//! An adapter knows how to find one emulator, which systems it runs, the
//! "quality ladder" of settings it offers for each system, and how to build
//! the command that starts a game directly (no emulator menu).
//!
//! How settings reach the emulator, by order of preference:
//! - **Session overrides**, when the emulator has them (RetroArch
//!   `--appendconfig`, Dolphin `--config`, mGBA `-C`): the user's files are
//!   not written at all.
//! - **Editing the emulator's settings file** otherwise (PCSX2, DuckStation,
//!   PPSSPP), through [`edit_ini`]: only the keys the launcher manages are
//!   changed, the original file is backed up once, and any value the user
//!   changed by hand is kept.
//!
//! In both cases [`EmulatorAdapter::current_values`] lets the engine notice
//! when the user changes a managed setting inside the emulator; from then on
//! that key is the user's (see `cl_core::watch`).
//!
//! Emulators without a built-in adapter are described by profile files
//! ([`profile`]) or added by the user as custom emulators.

mod dolphin;
mod pcsx2;
pub mod profile;
mod retroarch;

use cl_core::config::{EmulatorConfig, EmulatorMode};
use cl_core::ini::{apply_owned, ApplyReport, IniDoc, Ledger};
use cl_core::{Game, GraphicsApi, QualityStep, SystemId};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

pub use dolphin::Dolphin;
pub use pcsx2::Pcsx2;
pub use profile::{Profile, ProfileAdapter};
pub use retroarch::RetroArch;

#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("invalid emulator profile {file}: {message}")]
    Profile { file: String, message: String },
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
    /// Folder where the adapter may write override files, ledgers and backups.
    pub generated_dir: &'a Path,
    /// Keys that belong to the user (locked in the settings, or changed by
    /// hand in the emulator): never changed.
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
    /// System-level optimisations, decided by the engine for every emulator.
    pub tweaks: SystemTweaks,
}

/// Optimisations that need no knowledge of the emulator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct SystemTweaks {
    /// Start the emulator with a higher process priority (Windows).
    pub high_priority: bool,
    /// Ask Windows to run the emulator on the high-performance GPU, unless
    /// the user already chose a GPU for it in the Windows settings.
    pub prefer_discrete_gpu: bool,
}

pub trait EmulatorAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    /// SPDX licence of the emulator itself, for the credits screen.
    fn license(&self) -> &str;
    fn homepage(&self) -> &str;
    fn systems(&self) -> &[SystemId];
    /// Graphics back-ends, best first for the current OS.
    fn apis(&self) -> Vec<GraphicsApi>;
    /// Executable names to look for on this OS.
    fn exe_names(&self) -> Vec<String>;
    /// Usual install locations to probe when no path is configured.
    fn common_paths(&self) -> Vec<PathBuf> {
        Vec::new()
    }
    /// Quality ladder for a system, lightest first. Never empty.
    fn quality_ladder(&self, system: SystemId) -> Vec<QualityStep>;
    fn prepare(&self, ctx: &LaunchContext) -> Result<LaunchPlan, AdapterError>;

    /// True when the ladder raises rendering resolution (GPU-bound),
    /// false when it trades CPU time for latency.
    fn gpu_bound_ladder(&self) -> bool {
        true
    }

    /// Whether the launcher knows the emulator's own settings. When false,
    /// only the system-level optimisations apply.
    fn tunes_settings(&self) -> bool {
        true
    }

    /// Extra arguments for measured sessions, so that every console
    /// refresh is presented (otherwise a 30 fps game and a game running at
    /// half speed look the same in a capture).
    fn measurement_args(&self) -> Vec<String> {
        Vec::new()
    }

    /// Arguments that open the emulator without a game, so the user can
    /// set it up (BIOS, controllers).
    fn setup_args(&self) -> Vec<String> {
        Vec::new()
    }

    /// Keys the launcher manages for a system: those of the ladder and any
    /// key the adapter sets on every launch.
    fn managed_keys(&self, system: SystemId) -> Vec<String> {
        ladder_keys(&self.quality_ladder(system))
    }

    /// Values of the managed keys currently stored in the emulator's own
    /// settings, or `None` when the settings file is unknown or missing.
    fn current_values(
        &self,
        _exe: &Path,
        _mode: EmulatorMode,
        _generated_dir: &Path,
    ) -> Option<BTreeMap<String, String>> {
        None
    }

    /// Finds the executable: configured path, managed install folder
    /// (given by the updater), emulator folders, PATH, then usual locations.
    fn find_executable(
        &self,
        cfg: &EmulatorConfig,
        managed_install: Option<&Path>,
        index: &ExeIndex,
    ) -> Option<PathBuf> {
        let names = self.exe_names();
        if let Some(p) = &cfg.path {
            if p.is_file() {
                return Some(p.clone());
            }
        }
        if cfg.mode == EmulatorMode::Managed {
            if let Some(found) = managed_install.and_then(|dir| find_in_tree(dir, &names)) {
                return Some(found);
            }
        }
        index
            .find(&names)
            .or_else(|| find_in_path(&names))
            .or_else(|| self.common_paths().into_iter().find(|p| p.is_file()))
    }
}

/// Union of the keys used by a ladder.
pub fn ladder_keys(ladder: &[QualityStep]) -> Vec<String> {
    let set: BTreeSet<String> = ladder
        .iter()
        .flat_map(|s| s.settings.keys().cloned())
        .collect();
    set.into_iter().collect()
}

fn find_in_path(names: &[String]) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |n| dir.join(n)))
        .find(|p| p.is_file())
}

fn file_name_lower(p: &Path) -> Option<String> {
    p.file_name().map(|n| n.to_string_lossy().to_lowercase())
}

/// Looks for one of `names` (case-insensitive) in `dir` and up to two levels
/// below (release archives often contain a top folder).
pub fn find_in_tree(dir: &Path, names: &[String]) -> Option<PathBuf> {
    let wanted: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
    fn walk(dir: &Path, wanted: &[String], depth: u8) -> Option<PathBuf> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .ok()?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect();
        entries.sort();
        for w in wanted {
            if let Some(f) = entries
                .iter()
                .find(|p| p.is_file() && file_name_lower(p).as_deref() == Some(w.as_str()))
            {
                return Some(f.clone());
            }
        }
        if depth == 0 {
            return None;
        }
        entries
            .iter()
            .filter(|p| p.is_dir())
            .find_map(|d| walk(d, wanted, depth - 1))
    }
    walk(dir, &wanted, 2)
}

/// Executables found in the emulator folders (for example RetroBat's
/// `emulators` folder), built once and shared by every adapter.
#[derive(Debug, Clone, Default)]
pub struct ExeIndex {
    by_name: HashMap<String, PathBuf>,
}

impl ExeIndex {
    /// Indexes the files named like one of `wanted` in `dirs`, up to three
    /// levels deep. The first folder wins when a name appears twice.
    pub fn build(dirs: &[PathBuf], wanted: &[String]) -> ExeIndex {
        let wanted: BTreeSet<String> = wanted.iter().map(|n| n.to_lowercase()).collect();
        let mut by_name = HashMap::new();
        fn walk(
            dir: &Path,
            depth: u8,
            wanted: &BTreeSet<String>,
            out: &mut HashMap<String, PathBuf>,
        ) {
            let Ok(rd) = std::fs::read_dir(dir) else {
                return;
            };
            let mut entries: Vec<PathBuf> = rd.filter_map(Result::ok).map(|e| e.path()).collect();
            entries.sort();
            for p in &entries {
                if p.is_file() {
                    if let Some(n) = file_name_lower(p) {
                        if wanted.contains(&n) {
                            out.entry(n).or_insert_with(|| p.clone());
                        }
                    }
                }
            }
            if depth > 0 {
                for p in entries.iter().filter(|p| p.is_dir()) {
                    walk(p, depth - 1, wanted, out);
                }
            }
        }
        for d in dirs {
            walk(d, 3, &wanted, &mut by_name);
        }
        ExeIndex { by_name }
    }

    pub fn find(&self, names: &[String]) -> Option<PathBuf> {
        names
            .iter()
            .find_map(|n| self.by_name.get(&n.to_lowercase()))
            .filter(|p| p.is_file())
            .cloned()
    }
}

/// All built-in adapters: the three written in Rust and the bundled profiles.
pub fn registry() -> Vec<Box<dyn EmulatorAdapter>> {
    let mut list: Vec<Box<dyn EmulatorAdapter>> =
        vec![Box::new(RetroArch), Box::new(Dolphin), Box::new(Pcsx2)];
    for p in profile::builtin_profiles() {
        list.push(Box::new(ProfileAdapter::new(p)));
    }
    list
}

/// Emulators to try for a system, preferred first. The engine takes the
/// first one installed, unless the user chose one.
pub fn preferred_emulators(system: SystemId) -> &'static [&'static str] {
    use SystemId::*;
    match system {
        Gb | Gbc | Gba => &["mgba", "retroarch"],
        Nds => &["melonds", "retroarch"],
        Psx => &["duckstation", "retroarch"],
        Psp => &["ppsspp", "retroarch"],
        GameCube | Wii => &["dolphin"],
        Ps2 => &["pcsx2"],
        WiiU => &["cemu"],
        Nes | Snes | N64 | MasterSystem | MegaDrive => &["retroarch"],
    }
}

/// Default emulator per system, when none is installed yet.
pub fn default_emulator(system: SystemId) -> &'static str {
    preferred_emulators(system)[0]
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

/// First existing file among path templates (see `cl_core::paths::expand`).
pub fn first_existing(templates: &[String], exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent();
    templates
        .iter()
        .filter_map(|t| cl_core::paths::expand(t, dir))
        .find(|p| p.is_file())
}

/// Reads `Section/Key` values from an INI file. Keys without a section
/// (`Key` alone) are looked up before the first section header.
pub fn read_ini_values(file: &Path, keys: &[String]) -> Option<BTreeMap<String, String>> {
    let doc = IniDoc::parse(&std::fs::read_to_string(file).ok()?);
    Some(
        keys.iter()
            .filter_map(|k| {
                let (section, key) = k.rsplit_once('/').unwrap_or(("", k));
                doc.get(section, key).map(|v| (k.clone(), v))
            })
            .collect(),
    )
}

/// Changes the managed keys of an emulator's INI file.
///
/// - The original file is copied once to `<generated>/backups/<emulator>/`
///   before the first change.
/// - A ledger remembers what the launcher wrote; values the user changed
///   since are kept (see [`apply_owned`]).
/// - `locked` keys are never written.
pub fn edit_ini(
    file: &Path,
    emulator: &str,
    changes: &BTreeMap<String, String>,
    locked: &[String],
    generated_dir: &Path,
) -> Result<ApplyReport, AdapterError> {
    let ledger_path = generated_dir.join(format!("{emulator}-ledger.json"));
    let mut ledger: Ledger = std::fs::read_to_string(&ledger_path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    let text = std::fs::read_to_string(file)?;
    let mut doc = IniDoc::parse(&text);
    let report = apply_owned(&mut doc, changes, &mut ledger, locked);
    if !report.applied.is_empty() {
        let backup_dir = generated_dir.join("backups").join(emulator);
        std::fs::create_dir_all(&backup_dir)?;
        let name = file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "settings".into());
        let backup = backup_dir.join(format!("{name}.orig"));
        if !backup.exists() {
            std::fs::write(&backup, &text)?;
        }
        std::fs::write(file, doc.render())?;
    }
    std::fs::create_dir_all(generated_dir)?;
    std::fs::write(
        &ledger_path,
        serde_json::to_string_pretty(&ledger).unwrap_or_default(),
    )?;
    Ok(report)
}

/// Forgets what the launcher wrote for an emulator, so the next launch
/// takes every managed key back.
pub fn forget_ledger(emulator: &str, generated_dir: &Path) {
    let _ = std::fs::remove_file(generated_dir.join(format!("{emulator}-ledger.json")));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_finds_executables_case_insensitively() {
        let dir = tempfile::tempdir().unwrap();
        let emus = dir.path().join("emulators");
        std::fs::create_dir_all(emus.join("dolphin-emu")).unwrap();
        std::fs::create_dir_all(emus.join("retroarch/cores")).unwrap();
        std::fs::write(emus.join("dolphin-emu/Dolphin.exe"), b"").unwrap();
        std::fs::write(emus.join("retroarch/retroarch.exe"), b"").unwrap();
        let idx = ExeIndex::build(
            std::slice::from_ref(&emus),
            &["dolphin.exe".into(), "retroarch.exe".into()],
        );
        assert_eq!(
            idx.find(&["Dolphin.exe".into()]).unwrap(),
            emus.join("dolphin-emu/Dolphin.exe")
        );
        assert!(idx.find(&["pcsx2-qt.exe".into()]).is_none());
        assert_eq!(
            find_in_tree(&emus, &["RETROARCH.EXE".into()]).unwrap(),
            emus.join("retroarch/retroarch.exe")
        );
    }

    #[test]
    fn edit_ini_backs_up_once_and_keeps_user_values() {
        let dir = tempfile::tempdir().unwrap();
        let ini = dir.path().join("settings.ini");
        std::fs::write(&ini, "[GPU]\nResolutionScale = 1\n").unwrap();
        let gen = dir.path().join("gen");
        let ch = |v: &str| BTreeMap::from([("GPU/ResolutionScale".to_string(), v.to_string())]);
        let r = edit_ini(&ini, "emu", &ch("3"), &[], &gen).unwrap();
        assert_eq!(r.applied, vec!["GPU/ResolutionScale"]);
        let backup = gen.join("backups/emu/settings.ini.orig");
        assert!(std::fs::read_to_string(&backup)
            .unwrap()
            .contains("ResolutionScale = 1"));
        std::fs::write(&ini, "[GPU]\nResolutionScale = 5\n").unwrap();
        let r = edit_ini(&ini, "emu", &ch("2"), &[], &gen).unwrap();
        assert_eq!(r.kept_user_value, vec!["GPU/ResolutionScale"]);
        // The backup still holds the very first version.
        assert!(std::fs::read_to_string(&backup)
            .unwrap()
            .contains("ResolutionScale = 1"));
        forget_ledger("emu", &gen);
        let r = edit_ini(&ini, "emu", &ch("2"), &[], &gen).unwrap();
        assert_eq!(r.applied, vec!["GPU/ResolutionScale"]);
    }
}
