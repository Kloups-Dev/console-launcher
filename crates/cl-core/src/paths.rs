use std::path::{Path, PathBuf};

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

/// Expands a path template used in emulator profiles. Placeholders:
/// `{exe_dir}` (folder of the emulator), `{home}`, `{documents}`,
/// `{config}` (Windows: Roaming AppData, Linux: `~/.config`),
/// `{data}` (Windows: Local AppData, Linux: `~/.local/share`),
/// `{appdata}`, `{localappdata}`, `{programfiles}` (Windows).
/// Returns `None` when a placeholder cannot be resolved on this machine.
pub fn expand(template: &str, exe_dir: Option<&Path>) -> Option<PathBuf> {
    let base = directories::BaseDirs::new();
    let user = directories::UserDirs::new();
    let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
    let mut out = template.to_owned();
    let mut sub = |name: &str, value: Option<PathBuf>| -> Option<()> {
        let tag = format!("{{{name}}}");
        if out.contains(&tag) {
            out = out.replace(&tag, &value?.to_string_lossy());
        }
        Some(())
    };
    sub("exe_dir", exe_dir.map(Path::to_path_buf))?;
    sub("home", base.as_ref().map(|b| b.home_dir().to_path_buf()))?;
    sub(
        "documents",
        user.as_ref()
            .and_then(|u| u.document_dir().map(Path::to_path_buf)),
    )?;
    sub(
        "config",
        base.as_ref().map(|b| b.config_dir().to_path_buf()),
    )?;
    sub(
        "data",
        base.as_ref().map(|b| b.data_local_dir().to_path_buf()),
    )?;
    sub("appdata", env("APPDATA"))?;
    sub("localappdata", env("LOCALAPPDATA"))?;
    sub("programfiles", env("ProgramFiles"))?;
    Some(PathBuf::from(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_known_placeholders() {
        let p = expand("{exe_dir}/inis/a.ini", Some(Path::new("/emu"))).unwrap();
        assert_eq!(p, PathBuf::from("/emu/inis/a.ini"));
        assert!(expand("{exe_dir}/x", None).is_none());
        assert_eq!(expand("/plain", None).unwrap(), PathBuf::from("/plain"));
    }
}
