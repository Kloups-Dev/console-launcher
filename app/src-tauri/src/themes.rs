//! Themes: folders of HTML/CSS/JS under `<data>/themes/<name>/`.
//!
//! The two templates shipped with the launcher are written there on first
//! start (never overwritten afterwards, so users can edit them freely).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const BUILTIN: &[(&str, &[(&str, &str)])] = &[
    (
        "horizon",
        &[
            (
                "theme.json",
                include_str!("../../../themes/horizon/theme.json"),
            ),
            (
                "index.html",
                include_str!("../../../themes/horizon/index.html"),
            ),
            (
                "style.css",
                include_str!("../../../themes/horizon/style.css"),
            ),
            ("theme.js", include_str!("../../../themes/horizon/theme.js")),
        ],
    ),
    (
        "grid",
        &[
            (
                "theme.json",
                include_str!("../../../themes/grid/theme.json"),
            ),
            (
                "index.html",
                include_str!("../../../themes/grid/index.html"),
            ),
            ("style.css", include_str!("../../../themes/grid/style.css")),
            ("theme.js", include_str!("../../../themes/grid/theme.js")),
        ],
    ),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeManifest {
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    /// User-adjustable options, exposed as CSS variables `--opt-<id>`.
    #[serde(default)]
    pub options: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadedTheme {
    pub id: String,
    pub dir: PathBuf,
    pub manifest: ThemeManifest,
    pub html: String,
    pub css: String,
    pub js: String,
    /// Option values chosen by the user (defaults applied by the SDK).
    pub user_options: BTreeMap<String, Value>,
}

/// FNV-1a 64-bit hash of a file's content.
fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Built-in files shipped by version 0.1, which had no install record.
const V01_FILES: &[(&str, &str, u64)] = &[
    ("horizon", "theme.json", 0xbb885da393a0342a),
    ("horizon", "index.html", 0x21de319a653aca67),
    ("horizon", "style.css", 0xa880c467bd04d75f),
    ("horizon", "theme.js", 0x243a620231c036c7),
    ("grid", "theme.json", 0x229e1bb7d21bc576),
    ("grid", "index.html", 0xc6127ddde02440da),
    ("grid", "style.css", 0x9ef7b7e9fe965e96),
    ("grid", "theme.js", 0xbd94aa50e8e07e24),
];

const RECORD: &str = ".builtin.json";

/// Copies the built-in themes into the user's theme folder, and updates
/// them when a new version of the launcher ships new files. A file the user
/// edited is never replaced: only files still identical to a version the
/// launcher installed are updated.
pub fn install_builtins(themes_dir: &Path) -> std::io::Result<()> {
    for (name, files) in BUILTIN {
        let dir = themes_dir.join(name);
        std::fs::create_dir_all(&dir)?;
        let record_path = dir.join(RECORD);
        let mut record: BTreeMap<String, u64> = std::fs::read_to_string(&record_path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        for (file, content) in *files {
            let path = dir.join(file);
            let new_hash = fnv(content.as_bytes());
            let ours = match std::fs::read(&path) {
                Err(_) => true,
                Ok(current) => {
                    let h = fnv(&current);
                    h == new_hash
                        || record.get(*file) == Some(&h)
                        || V01_FILES
                            .iter()
                            .any(|(t, f, old)| t == name && f == file && *old == h)
                }
            };
            if ours {
                if std::fs::read(&path).map(|c| fnv(&c)).ok() != Some(new_hash) {
                    std::fs::write(&path, content)?;
                }
                record.insert((*file).to_owned(), new_hash);
            } else {
                log::info!("theme {name}: {file} edited by the user, kept");
            }
        }
        std::fs::write(
            &record_path,
            serde_json::to_string_pretty(&record).unwrap_or_default(),
        )?;
    }
    Ok(())
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

pub fn list(themes_dir: &Path) -> Vec<(String, ThemeManifest)> {
    let mut out: Vec<_> = std::fs::read_dir(themes_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|e| {
            let id = e.file_name().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(e.path().join("theme.json")).ok()?;
            Some((id, serde_json::from_str(&text).ok()?))
        })
        .filter(|(id, _): &(String, ThemeManifest)| valid_name(id))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn options_file(dir: &Path) -> PathBuf {
    dir.join("user-options.json")
}

pub fn load(themes_dir: &Path, name: &str) -> Result<LoadedTheme, String> {
    if !valid_name(name) {
        return Err(format!("invalid theme name '{name}'"));
    }
    let dir = themes_dir.join(name);
    let read =
        |f: &str| std::fs::read_to_string(dir.join(f)).map_err(|e| format!("{name}/{f}: {e}"));
    let manifest: ThemeManifest = serde_json::from_str(&read("theme.json")?)
        .map_err(|e| format!("{name}/theme.json: {e}"))?;
    let user_options = std::fs::read_to_string(options_file(&dir))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    Ok(LoadedTheme {
        id: name.into(),
        manifest,
        html: read("index.html")?,
        css: read("style.css").unwrap_or_default(),
        js: read("theme.js").unwrap_or_default(),
        user_options,
        dir,
    })
}

pub fn set_option(themes_dir: &Path, name: &str, key: &str, value: Value) -> Result<(), String> {
    if !valid_name(name) {
        return Err("invalid theme name".into());
    }
    let dir = themes_dir.join(name);
    let mut opts: BTreeMap<String, Value> = std::fs::read_to_string(options_file(&dir))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    opts.insert(key.into(), value);
    std::fs::write(
        options_file(&dir),
        serde_json::to_string_pretty(&opts).unwrap_or_default(),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_install_and_load() {
        let dir = std::env::temp_dir().join(format!("cl-themes-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        install_builtins(&dir).unwrap();
        let names: Vec<_> = list(&dir).into_iter().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["grid", "horizon"]);
        let t = load(&dir, "horizon").unwrap();
        assert!(t.html.contains("data-region"));
        set_option(&dir, "horizon", "accent", Value::String("#ff0000".into())).unwrap();
        assert_eq!(
            load(&dir, "horizon").unwrap().user_options["accent"],
            "#ff0000"
        );
        assert!(load(&dir, "../etc").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
