//! Managed emulator installs.
//!
//! Layout, per emulator, under `<data>/emulators/<id>/`:
//!
//! ```text
//! state.json          active version, installed versions
//! versions/<ver>/     one folder per installed version
//! downloads/          archives being downloaded
//! ```
//!
//! Rules:
//! - Updates follow the policy chosen by the user (`auto`, `notify`, `off`).
//! - A pinned version is never replaced.
//! - The previous version is kept, so one click goes back to it.
//! - Downloads come only from the emulator's official release page.
//!   Nothing is bundled with the launcher.

pub mod retroarch;
pub mod sources;

use cl_core::config::{EmulatorConfig, UpdatePolicy};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

pub use sources::{Release, Source};

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("network error: {0}")]
    Network(String),
    #[error("archive error: {0}")]
    Archive(String),
    #[error("{0}")]
    Other(String),
}

/// How many versions stay installed (active + previous ones).
pub const KEEP_VERSIONS: usize = 2;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallState {
    pub active: Option<String>,
    /// Oldest first.
    pub installed: Vec<String>,
}

/// Compares versions like "v2.2.0", "2.10", "1.19.1". Falls back to text order.
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches(['v', 'V'])
            .split(|c: char| !c.is_ascii_digit())
            .filter(|p| !p.is_empty())
            .filter_map(|p| p.parse().ok())
            .collect()
    };
    let (pa, pb) = (parse(a), parse(b));
    if pa.is_empty() || pb.is_empty() {
        return a.cmp(b);
    }
    let n = pa.len().max(pb.len());
    for i in 0..n {
        let (x, y) = (
            pa.get(i).copied().unwrap_or(0),
            pb.get(i).copied().unwrap_or(0),
        );
        match x.cmp(&y) {
            Ordering::Equal => continue,
            o => return o,
        }
    }
    Ordering::Equal
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Decision {
    UpToDate,
    /// Install now (auto policy, or nothing installed yet).
    Install {
        version: String,
    },
    /// Tell the user a new version exists.
    Notify {
        version: String,
    },
    /// Not checked or not applied, with the reason.
    Skip {
        reason: String,
    },
}

/// Pure decision logic, separated from the network for testing.
pub fn decide(state: &InstallState, cfg: &EmulatorConfig, latest: Option<&str>) -> Decision {
    if let Some(pin) = &cfg.pinned {
        return if state.active.as_deref() == Some(pin.as_str()) {
            Decision::Skip {
                reason: format!("version {pin} pinned by the user"),
            }
        } else if state.installed.contains(pin) {
            Decision::Skip {
                reason: format!("version {pin} pinned: activate it with 'rollback' or 'activate'"),
            }
        } else {
            Decision::Install {
                version: pin.clone(),
            }
        };
    }
    if cfg.updates == UpdatePolicy::Off && state.active.is_some() {
        return Decision::Skip {
            reason: "updates disabled by the user".into(),
        };
    }
    let Some(latest) = latest else {
        return Decision::Skip {
            reason: "no release information".into(),
        };
    };
    match &state.active {
        None => Decision::Install {
            version: latest.into(),
        },
        Some(cur) if compare_versions(latest, cur) == Ordering::Greater => match cfg.updates {
            UpdatePolicy::Auto => Decision::Install {
                version: latest.into(),
            },
            UpdatePolicy::Notify | UpdatePolicy::Off => Decision::Notify {
                version: latest.into(),
            },
        },
        Some(_) => Decision::UpToDate,
    }
}

/// One emulator's managed folder.
pub struct ManagedEmulator {
    pub id: String,
    pub root: PathBuf,
}

impl ManagedEmulator {
    pub fn new(emulators_dir: &Path, id: &str) -> ManagedEmulator {
        ManagedEmulator {
            id: id.into(),
            root: emulators_dir.join(id),
        }
    }

    fn state_file(&self) -> PathBuf {
        self.root.join("state.json")
    }

    pub fn version_dir(&self, version: &str) -> PathBuf {
        let safe: String = version
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || ".-_".contains(c) {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        self.root.join("versions").join(safe)
    }

    pub fn state(&self) -> InstallState {
        std::fs::read_to_string(self.state_file())
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    fn save_state(&self, s: &InstallState) -> Result<(), UpdateError> {
        std::fs::create_dir_all(&self.root)?;
        let tmp = self.root.join("state.json.tmp");
        std::fs::write(
            &tmp,
            serde_json::to_string_pretty(s).map_err(|e| UpdateError::Other(e.to_string()))?,
        )?;
        std::fs::rename(tmp, self.state_file())?;
        Ok(())
    }

    /// Folder of the active version, for the adapters.
    pub fn active_dir(&self) -> Option<PathBuf> {
        self.state()
            .active
            .map(|v| self.version_dir(&v))
            .filter(|d| d.is_dir())
    }

    /// Installs `archive` as `version` and makes it active. `exe_name` is
    /// used to rename a single-file download (AppImage) to the name the
    /// adapter looks for.
    pub fn install_archive(
        &self,
        version: &str,
        archive: &Path,
        exe_name: &str,
    ) -> Result<(), UpdateError> {
        let dest = self.version_dir(version);
        let staging = dest.with_extension("partial");
        if staging.exists() {
            std::fs::remove_dir_all(&staging)?;
        }
        std::fs::create_dir_all(&staging)?;
        let name = archive
            .file_name()
            .map(|n| n.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        let extracted = if name.ends_with(".zip") {
            extract_zip(archive, &staging)
        } else if name.ends_with(".7z") {
            sevenz_rust2::decompress_file(archive, &staging)
                .map_err(|e| UpdateError::Archive(e.to_string()))
        } else if name.ends_with(".appimage") || name.ends_with(".exe") {
            let target = staging.join(exe_name);
            std::fs::copy(archive, &target)?;
            make_executable(&target)
        } else {
            Err(UpdateError::Archive(format!(
                "unsupported file type: {name}"
            )))
        };
        if let Err(e) = extracted {
            let _ = std::fs::remove_dir_all(&staging);
            return Err(e);
        }
        if dest.exists() {
            std::fs::remove_dir_all(&dest)?;
        }
        std::fs::rename(&staging, &dest)?;

        let mut state = self.state();
        state.installed.retain(|v| v != version);
        state.installed.push(version.into());
        state.active = Some(version.into());
        self.prune(&mut state)?;
        self.save_state(&state)
    }

    /// Removes the oldest versions beyond [`KEEP_VERSIONS`]; never the active one.
    fn prune(&self, state: &mut InstallState) -> Result<(), UpdateError> {
        while state.installed.len() > KEEP_VERSIONS {
            let Some(pos) = state
                .installed
                .iter()
                .position(|v| Some(v) != state.active.as_ref())
            else {
                break;
            };
            let old = state.installed.remove(pos);
            let dir = self.version_dir(&old);
            if dir.exists() {
                std::fs::remove_dir_all(dir)?;
            }
        }
        Ok(())
    }

    /// Activates the version installed before the active one.
    pub fn rollback(&self) -> Result<String, UpdateError> {
        let mut state = self.state();
        let active = state
            .active
            .clone()
            .ok_or_else(|| UpdateError::Other("nothing installed".into()))?;
        let previous = state
            .installed
            .iter()
            .filter(|v| **v != active)
            .max_by(|a, b| compare_versions(a, b))
            .cloned()
            .ok_or_else(|| UpdateError::Other("no previous version kept".into()))?;
        state.active = Some(previous.clone());
        self.save_state(&state)?;
        Ok(previous)
    }

    pub fn activate(&self, version: &str) -> Result<(), UpdateError> {
        let mut state = self.state();
        if !state.installed.iter().any(|v| v == version) {
            return Err(UpdateError::Other(format!(
                "version {version} is not installed"
            )));
        }
        state.active = Some(version.into());
        self.save_state(&state)
    }
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<(), UpdateError> {
    let file = std::fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| UpdateError::Archive(e.to_string()))?;
    // `extract` rejects entries escaping the destination folder.
    zip.extract(dest)
        .map_err(|e| UpdateError::Archive(e.to_string()))
}

fn make_executable(path: &Path) -> Result<(), UpdateError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut p = std::fs::metadata(path)?.permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(path, p)?;
    }
    let _ = path;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn cfg(updates: UpdatePolicy, pinned: Option<&str>) -> EmulatorConfig {
        EmulatorConfig {
            updates,
            pinned: pinned.map(Into::into),
            ..Default::default()
        }
    }

    fn state(active: Option<&str>, installed: &[&str]) -> InstallState {
        InstallState {
            active: active.map(Into::into),
            installed: installed.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn version_ordering() {
        assert_eq!(compare_versions("v2.10.0", "2.9.9"), Ordering::Greater);
        assert_eq!(compare_versions("1.19", "1.19.0"), Ordering::Equal);
        assert_eq!(compare_versions("2509", "2506"), Ordering::Greater);
    }

    #[test]
    fn decisions_follow_policy() {
        let s = state(Some("2.2.0"), &["2.2.0"]);
        assert_eq!(
            decide(&s, &cfg(UpdatePolicy::Auto, None), Some("v2.4.0")),
            Decision::Install {
                version: "v2.4.0".into()
            }
        );
        assert_eq!(
            decide(&s, &cfg(UpdatePolicy::Notify, None), Some("v2.4.0")),
            Decision::Notify {
                version: "v2.4.0".into()
            }
        );
        assert!(matches!(
            decide(&s, &cfg(UpdatePolicy::Off, None), Some("v2.4.0")),
            Decision::Skip { .. }
        ));
        assert_eq!(
            decide(&s, &cfg(UpdatePolicy::Auto, None), Some("2.2.0")),
            Decision::UpToDate
        );
        // Nothing installed yet: install even with updates off.
        assert!(matches!(
            decide(
                &state(None, &[]),
                &cfg(UpdatePolicy::Off, None),
                Some("2.2.0")
            ),
            Decision::Install { .. }
        ));
    }

    #[test]
    fn pinned_version_is_never_replaced() {
        let s = state(Some("2.2.0"), &["2.2.0"]);
        assert!(matches!(
            decide(&s, &cfg(UpdatePolicy::Auto, Some("2.2.0")), Some("9.0.0")),
            Decision::Skip { .. }
        ));
        assert_eq!(
            decide(
                &state(Some("2.4.0"), &["2.4.0"]),
                &cfg(UpdatePolicy::Auto, Some("2.2.0")),
                Some("9.0.0")
            ),
            Decision::Install {
                version: "2.2.0".into()
            }
        );
    }

    fn make_zip(path: &Path, inner: &str) {
        let f = std::fs::File::create(path).unwrap();
        let mut z = zip::ZipWriter::new(f);
        z.start_file::<_, ()>(inner, zip::write::SimpleFileOptions::default())
            .unwrap();
        z.write_all(b"binary").unwrap();
        z.finish().unwrap();
    }

    #[test]
    fn install_keeps_previous_and_rolls_back() {
        let dir = tempfile::tempdir().unwrap();
        let m = ManagedEmulator::new(&dir.path().join("emulators"), "pcsx2");
        for v in ["1.0.0", "1.1.0", "1.2.0"] {
            let a = dir.path().join(format!("{v}.zip"));
            make_zip(&a, "PCSX2/pcsx2-qt.exe");
            m.install_archive(v, &a, "pcsx2-qt.exe").unwrap();
        }
        let s = m.state();
        assert_eq!(s.active.as_deref(), Some("1.2.0"));
        assert_eq!(s.installed, vec!["1.1.0", "1.2.0"]);
        assert!(!m.version_dir("1.0.0").exists());
        assert!(m.version_dir("1.2.0").join("PCSX2/pcsx2-qt.exe").is_file());

        assert_eq!(m.rollback().unwrap(), "1.1.0");
        assert_eq!(m.active_dir().unwrap(), m.version_dir("1.1.0"));
    }

    #[test]
    fn single_file_download_is_renamed() {
        let dir = tempfile::tempdir().unwrap();
        let m = ManagedEmulator::new(dir.path(), "pcsx2");
        let a = dir
            .path()
            .join("pcsx2-v2.2.0-linux-appimage-x64-Qt.AppImage");
        std::fs::write(&a, b"elf").unwrap();
        m.install_archive("2.2.0", &a, "pcsx2-qt").unwrap();
        assert!(m.version_dir("2.2.0").join("pcsx2-qt").is_file());
    }

    #[test]
    fn broken_archive_leaves_previous_version_active() {
        let dir = tempfile::tempdir().unwrap();
        let m = ManagedEmulator::new(dir.path(), "x");
        let good = dir.path().join("good.zip");
        make_zip(&good, "x.exe");
        m.install_archive("1.0", &good, "x.exe").unwrap();
        let bad = dir.path().join("bad.zip");
        std::fs::write(&bad, b"not a zip").unwrap();
        assert!(m.install_archive("2.0", &bad, "x.exe").is_err());
        assert_eq!(m.state().active.as_deref(), Some("1.0"));
        assert!(!m.version_dir("2.0").exists());
    }
}
