//! Where official releases come from.
//!
//! Official release sources used for launcher-managed downloads.

use crate::UpdateError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub enum Source {
    GitHub {
        owner: &'static str,
        repo: &'static str,
        /// The asset must contain every word (case-insensitive)…
        windows_words: &'static [&'static str],
        linux_words: &'static [&'static str],
        /// …and end with one of these suffixes.
        windows_suffix: &'static [&'static str],
        linux_suffix: &'static [&'static str],
    },
    /// A release deliberately selected and hash-verified by the launcher.
    /// Used when an emulator does not expose a suitable releases API.
    Pinned {
        version: &'static str,
        asset_name: &'static str,
        url: &'static str,
        sha256: &'static str,
    },
    /// The user installs and updates the emulator; the page is shown.
    Manual { page: &'static str },
}

/// Official source for each managed emulator.
/// Asset naming rules to re-check when a project changes its release files.
pub fn source_for(id: &str) -> Option<Source> {
    Some(match id {
        "pcsx2" => Source::GitHub {
            owner: "PCSX2",
            repo: "pcsx2",
            windows_words: &["windows", "x64"],
            linux_words: &["linux", "x64"],
            windows_suffix: &[".7z", ".zip"],
            linux_suffix: &[".appimage"],
        },
        "mgba" => Source::GitHub {
            owner: "mgba-emu",
            repo: "mgba",
            windows_words: &["windows", "x64"],
            linux_words: &["linux", "x64"],
            windows_suffix: &[".7z", ".zip"],
            linux_suffix: &[".appimage"],
        },
        "duckstation" => Source::GitHub {
            owner: "stenzek",
            repo: "duckstation",
            windows_words: &["windows", "x64"],
            linux_words: &["linux", "x64"],
            windows_suffix: &[".zip", ".7z"],
            linux_suffix: &[".appimage"],
        },
        "ppsspp" => Source::GitHub {
            owner: "hrydgard",
            repo: "ppsspp",
            windows_words: &["windows", "amd64"],
            linux_words: &["linux", "x86_64"],
            windows_suffix: &[".zip", ".7z"],
            linux_suffix: &[".tar.gz", ".zip"],
        },
        "melonds" => Source::GitHub {
            owner: "melonDS-emu",
            repo: "melonDS",
            windows_words: &["windows"],
            linux_words: &["linux"],
            windows_suffix: &[".zip", ".7z"],
            linux_suffix: &[".appimage", ".tar.gz"],
        },
        "cemu" => Source::GitHub {
            owner: "CemuProject",
            repo: "Cemu",
            windows_words: &["windows", "x64"],
            linux_words: &["linux", "x64"],
            windows_suffix: &[".zip", ".7z"],
            linux_suffix: &[".appimage", ".tar.gz"],
        },
        "dolphin" if cfg!(windows) => Source::Pinned {
            version: "2609a",
            asset_name: "dolphin-2609a-x64.7z",
            url: "https://dl.dolphin-emu.org/releases/2609a/dolphin-2609a-x64.7z",
            sha256: "BBFD13BF9E6D2A4164A15C392B1F32A9B18150E5136746B388704608FBB60F76",
        },
        "dolphin" => Source::Manual {
            page: "https://dolphin-emu.org/download/",
        },
        "retroarch" => Source::Manual {
            page: "https://www.retroarch.com/?page=platforms",
        },
        _ => return None,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub version: String,
    pub asset_name: String,
    pub url: String,
    pub size: u64,
    /// Expected archive hash for curated downloads.
    #[serde(default)]
    pub sha256: Option<String>,
}

#[derive(Deserialize)]
struct GhAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    prerelease: bool,
    draft: bool,
    assets: Vec<GhAsset>,
}

const USER_AGENT: &str = concat!("console-launcher/", env!("CARGO_PKG_VERSION"));

pub fn pick_asset<'a>(
    names: impl IntoIterator<Item = &'a str>,
    words: &[&str],
    suffix: &[&str],
) -> Option<&'a str> {
    names.into_iter().find(|n| {
        let l = n.to_ascii_lowercase();
        words.iter().all(|w| l.contains(w))
            && suffix.iter().any(|s| l.ends_with(s))
            && !l.contains("symbols")
    })
}

/// Latest stable release for this platform.
pub fn latest(source: &Source) -> Result<Option<Release>, UpdateError> {
    if let Source::Pinned {
        version,
        asset_name,
        url,
        sha256,
    } = source
    {
        return Ok(Some(Release {
            version: (*version).into(),
            asset_name: (*asset_name).into(),
            url: (*url).into(),
            size: 0,
            sha256: Some((*sha256).into()),
        }));
    }
    let Source::GitHub {
        owner,
        repo,
        windows_words,
        linux_words,
        windows_suffix,
        linux_suffix,
    } = source
    else {
        return Ok(None);
    };
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let rel: GhRelease = ureq::get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| UpdateError::Network(e.to_string()))?
        .body_mut()
        .read_json()
        .map_err(|e| UpdateError::Network(e.to_string()))?;
    if rel.draft || rel.prerelease {
        return Ok(None);
    }
    let (words, suffix) = if cfg!(windows) {
        (*windows_words, *windows_suffix)
    } else {
        (*linux_words, *linux_suffix)
    };
    let Some(name) = pick_asset(rel.assets.iter().map(|a| a.name.as_str()), words, suffix) else {
        return Ok(None);
    };
    let asset = rel
        .assets
        .iter()
        .find(|a| a.name == name)
        .expect("picked from list");
    Ok(Some(Release {
        version: rel.tag_name.trim_start_matches(['v', 'V']).to_owned(),
        asset_name: asset.name.clone(),
        url: asset.browser_download_url.clone(),
        size: asset.size,
        sha256: None,
    }))
}

/// Downloads a release into `dir`, returns the file path.
pub fn download(release: &Release, dir: &Path) -> Result<PathBuf, UpdateError> {
    std::fs::create_dir_all(dir)?;
    let safe: String = release
        .asset_name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || ".-_".contains(*c))
        .collect();
    let dest = dir.join(&safe);
    let partial = dir.join(format!("{safe}.part"));
    let mut resp = ureq::get(&release.url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| UpdateError::Network(e.to_string()))?;
    let mut reader = resp
        .body_mut()
        .with_config()
        .limit(4 * 1024 * 1024 * 1024)
        .reader();
    let mut file = std::fs::File::create(&partial)?;
    let mut hash = Sha256::new();
    let mut written = 0u64;
    let mut buffer = [0u8; 128 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        written += count as u64;
        hash.update(&buffer[..count]);
        file.write_all(&buffer[..count])?;
    }
    file.flush()?;
    if release.size > 0 && written != release.size {
        let _ = std::fs::remove_file(&partial);
        return Err(UpdateError::Network(format!(
            "incomplete download ({written}/{} bytes)",
            release.size
        )));
    }
    if let Some(expected) = &release.sha256 {
        let actual = format!("{:X}", hash.finalize());
        if !actual.eq_ignore_ascii_case(expected) {
            let _ = std::fs::remove_file(&partial);
            return Err(UpdateError::Other(format!(
                "archive hash mismatch (expected {expected}, got {actual})"
            )));
        }
    }
    std::fs::rename(&partial, &dest)?;
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_platform_asset() {
        let names = [
            "pcsx2-v2.2.0-macos-Qt.tar.xz",
            "pcsx2-v2.2.0-windows-x64-Qt-symbols.7z",
            "pcsx2-v2.2.0-windows-x64-Qt.7z",
            "pcsx2-v2.2.0-linux-appimage-x64-Qt.AppImage",
        ];
        assert_eq!(
            pick_asset(names, &["windows", "x64"], &[".7z"]),
            Some("pcsx2-v2.2.0-windows-x64-Qt.7z")
        );
        assert_eq!(
            pick_asset(names, &["linux", "x64"], &[".appimage"]),
            Some("pcsx2-v2.2.0-linux-appimage-x64-Qt.AppImage")
        );
    }

    #[test]
    fn dolphin_windows_release_is_curated() {
        let source = source_for("dolphin").unwrap();
        if cfg!(windows) {
            let release = latest(&source).unwrap().unwrap();
            assert_eq!(release.version, "2609a");
            assert_eq!(release.sha256.as_deref().map(str::len), Some(64));
            assert!(release.url.starts_with("https://dl.dolphin-emu.org/"));
        } else {
            assert!(matches!(source, Source::Manual { .. }));
        }
    }

    #[test]
    fn supported_emulators_have_direct_official_sources() {
        for id in ["pcsx2", "mgba", "duckstation", "ppsspp", "melonds", "cemu"] {
            assert!(
                matches!(source_for(id), Some(Source::GitHub { .. })),
                "{id} should expose an official release source"
            );
        }
        assert!(matches!(
            source_for("retroarch"),
            Some(Source::Manual { .. })
        ));
    }
}
