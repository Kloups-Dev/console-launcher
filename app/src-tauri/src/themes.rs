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

pub fn install_builtins(themes_dir: &Path) -> std::io::Result<()> {
    for (name, files) in BUILTIN {
        let dir = themes_dir.join(name);
        if dir.exists() {
            continue;
        }
        std::fs::create_dir_all(&dir)?;
        for (file, content) in *files {
            std::fs::write(dir.join(file), content)?;
        }
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
