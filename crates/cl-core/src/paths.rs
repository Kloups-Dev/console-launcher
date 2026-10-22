use std::path::PathBuf;

/// Folders used by the launcher. Everything lives under one data directory
/// so that the whole installation can be backed up or made portable.
#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
}

impl Paths {
    /// Standard per-user location:
    /// - Windows: `%APPDATA%\ConsoleLauncher\console-launcher\data`
    /// - Linux: `$XDG_DATA_HOME/console-launcher` (usually `~/.local/share/console-launcher`)
    ///
    /// The `CONSOLE_LAUNCHER_HOME` environment variable overrides it
    /// (portable mode, tests).
    pub fn discover() -> Paths {
        if let Some(dir) = std::env::var_os("CONSOLE_LAUNCHER_HOME") {
            return Paths {
                root: PathBuf::from(dir),
            };
        }
        let root = directories::ProjectDirs::from("org", "ConsoleLauncher", "console-launcher")
            .map(|p| p.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("console-launcher-data"));
        Paths { root }
    }

    pub fn at(root: impl Into<PathBuf>) -> Paths {
        Paths { root: root.into() }
    }

    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.toml")
    }
    /// Per-game profiles and test results.
    pub fn profiles_dir(&self) -> PathBuf {
        self.root.join("profiles")
    }
    /// Emulators installed and updated by the launcher.
    pub fn emulators_dir(&self) -> PathBuf {
        self.root.join("emulators")
    }
    /// Frame-time captures from real-condition tests.
    pub fn captures_dir(&self) -> PathBuf {
        self.root.join("captures")
    }
    /// User themes (built-in templates are copied here on first start).
    pub fn themes_dir(&self) -> PathBuf {
        self.root.join("themes")
    }
    /// Files generated for emulators (override configs, ownership ledgers).
    pub fn generated_dir(&self) -> PathBuf {
        self.root.join("generated")
    }
}
