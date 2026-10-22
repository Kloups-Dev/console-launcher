//! Where official releases come from.
//!
//! Version 0.1 supports GitHub releases (used by PCSX2). Dolphin and
//! RetroArch publish their builds on their own servers; their sources are
//! listed as `Manual` until dedicated readers are written (see the roadmap).

use crate::UpdateError;
use serde::{Deserialize, Serialize};
use std::io::Write;
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
    let written = std::io::copy(&mut reader, &mut file)?;
    file.flush()?;
    if release.size > 0 && written != release.size {
        let _ = std::fs::remove_file(&partial);
        return Err(UpdateError::Network(format!(
            "incomplete download ({written}/{} bytes)",
            release.size
        )));
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
}
