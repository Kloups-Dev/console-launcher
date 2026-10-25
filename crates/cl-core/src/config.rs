//! User settings (`config.toml`).

use crate::SystemId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid settings in {path}: {source}")]
    Parse {
        path: PathBuf,
        // Boxed: `toml::de::Error` is large and would bloat every `Result`.
        source: Box<toml::de::Error>,
    },
    #[error("cannot serialise settings: {0}")]
    Serialize(#[from] toml::ser::Error),
}

/// How an emulator is provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmulatorMode {
    /// Installed by the user. The launcher never edits its files; settings
    /// are passed only through command-line overrides when the emulator
    /// supports them.
    #[default]
    External,
    /// Installed and updated by the launcher in its data folder.
    Managed,
}

/// Update policy for a managed emulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdatePolicy {
    /// Install new stable versions automatically (previous one is kept).
    #[default]
    Auto,
    /// Report new versions, install only when the user accepts.
    Notify,
    /// Never check.
    Off,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EmulatorConfig {
    pub mode: EmulatorMode,
    /// Executable for external installs. Auto-detected when empty.
    pub path: Option<PathBuf>,
    pub updates: UpdatePolicy,
    /// Version locked by the user. Updates are skipped while it is set.
    pub pinned: Option<String>,
    /// Settings keys the user fixed by hand; the tuner never changes them.
    pub locked_keys: Vec<String>,
}

/// An emulator the launcher has no built-in profile for, added by the user.
/// It is launched directly and gets the system-level optimisations; its own
/// settings are tuned only when a profile file describes them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomEmulator {
    pub name: String,
    pub path: PathBuf,
    /// Arguments; `{rom}` is replaced by the game file. Empty means `["{rom}"]`.
    pub args: Vec<String>,
    pub systems: Vec<SystemId>,
}

/// A game file added directly by the user. This complements organised ROM
/// folders for people who keep games in arbitrary locations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedGame {
    pub path: PathBuf,
    pub system: SystemId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PerfConfig {
    /// Automatic optimisation on/off.
    pub auto_tune: bool,
    /// System-level optimisations applied to every emulator: higher process
    /// priority, and on Windows the high-performance GPU preference.
    pub system_tweaks: bool,
    /// Measurement length in seconds.
    pub capture_seconds: u32,
    /// Seconds to wait after launch before measuring (loading screens).
    pub capture_delay_seconds: u32,
    /// Windows: PresentMon executable used for measurements.
    pub presentmon_path: Option<PathBuf>,
}

impl Default for PerfConfig {
    fn default() -> Self {
        PerfConfig {
            auto_tune: true,
            system_tweaks: true,
            capture_seconds: 60,
            capture_delay_seconds: 20,
            presentmon_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Folders containing games, organised one sub-folder per system
    /// (`ps2`, `snes`, …). Files directly in the folder are matched by extension.
    pub rom_dirs: Vec<PathBuf>,
    /// Individual game files, with the console chosen by the user when the
    /// file extension is shared by several systems (`.iso`, `.rvz`, ...).
    pub imported_games: Vec<ImportedGame>,
    /// Folders searched for emulator executables (for example the
    /// `emulators` folder of RetroBat), in addition to the usual places.
    pub emulator_dirs: Vec<PathBuf>,
    /// Folder where the user keeps BIOS / firmware files they extracted
    /// from their own hardware. Never distributed by the launcher.
    pub bios_dir: Option<PathBuf>,
    /// Active theme folder name.
    pub theme: String,
    pub language: String,
    pub emulators: BTreeMap<String, EmulatorConfig>,
    /// Which emulator handles which system, when the default is not wanted.
    pub system_emulator: BTreeMap<SystemId, String>,
    /// Emulators added by the user, keyed by an id (`custom-…`).
    pub custom_emulators: BTreeMap<String, CustomEmulator>,
    /// Visual controller mappings, keyed by source device and target system.
    /// Emulator-specific application is deliberately handled by adapters.
    pub controller_mappings: BTreeMap<String, BTreeMap<String, f64>>,
    /// Emulator-native mapping overrides. Keys are adapter-specific config
    /// paths and values are validated strings passed only for the session.
    pub emulator_mappings: BTreeMap<String, BTreeMap<String, String>>,
    pub perf: PerfConfig,
    pub retroarch: RetroArchConfig,
}

/// User-facing choices; conservative audio/video defaults stay in the adapter.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RetroArchConfig {
    pub smooth_image: bool,
    /// Optional save-state resume. In-game saves are always independent.
    pub resume: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            rom_dirs: Vec::new(),
            imported_games: Vec::new(),
            emulator_dirs: Vec::new(),
            bios_dir: None,
            theme: "horizon".into(),
            language: "fr".into(),
            emulators: BTreeMap::new(),
            system_emulator: BTreeMap::new(),
            custom_emulators: BTreeMap::new(),
            controller_mappings: BTreeMap::new(),
            emulator_mappings: BTreeMap::new(),
            perf: PerfConfig::default(),
            retroarch: RetroArchConfig::default(),
        }
    }
}

impl AppConfig {
    /// Loads the file, or returns defaults when it does not exist yet.
    pub fn load(path: &Path) -> Result<AppConfig, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|source| ConfigError::Parse {
                path: path.into(),
                source: Box::new(source),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
            Err(source) => Err(ConfigError::Io {
                path: path.into(),
                source,
            }),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let text = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| ConfigError::Io {
                path: parent.into(),
                source,
            })?;
        }
        // Write to a temporary file then rename, so a crash never leaves a half-written file.
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, text).map_err(|source| ConfigError::Io {
            path: tmp.clone(),
            source,
        })?;
        std::fs::rename(&tmp, path).map_err(|source| ConfigError::Io {
            path: path.into(),
            source,
        })
    }

    pub fn emulator(&self, id: &str) -> EmulatorConfig {
        self.emulators.get(id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = AppConfig::load(&dir.path().join("config.toml")).unwrap();
        assert_eq!(cfg, AppConfig::default());
    }

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut cfg = AppConfig::default();
        cfg.rom_dirs.push("/games".into());
        cfg.imported_games.push(ImportedGame {
            path: "/elsewhere/Game.rvz".into(),
            system: SystemId::GameCube,
        });
        cfg.system_emulator
            .insert(SystemId::Psx, "retroarch".into());
        cfg.emulators.insert(
            "pcsx2".into(),
            EmulatorConfig {
                updates: UpdatePolicy::Off,
                pinned: Some("2.2.0".into()),
                ..Default::default()
            },
        );
        cfg.save(&path).unwrap();
        assert_eq!(AppConfig::load(&path).unwrap(), cfg);
    }

    #[test]
    fn partial_file_keeps_defaults() {
        let cfg: AppConfig = toml::from_str("theme = \"grid\"\n").unwrap();
        assert_eq!(cfg.theme, "grid");
        assert!(cfg.perf.auto_tune);
    }
}
