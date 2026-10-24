//! Detects the settings a user changed in an emulator, so the automatic
//! optimisation stops touching them.
//!
//! For every key the launcher manages, the watch keeps the last value seen
//! in the emulator's own settings (or the value the launcher wrote there).
//! When a later read finds a different value, the user changed it inside the
//! emulator: the key is marked as the user's and left alone until the user
//! asks the launcher to take it back ([`SettingsWatch::reset`]).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingsWatch {
    /// Last value seen or written for each managed key; `None` means the
    /// key was absent from the emulator's file.
    pub baseline: BTreeMap<String, Option<String>>,
    /// Keys changed by the user. The launcher leaves them alone.
    pub user_changed: BTreeSet<String>,
}

/// Compares two setting values the way emulators write them: case, spaces
/// and quotes are ignored, and numbers are compared by value (`3` = `3.000000`).
pub fn same_value(a: Option<&str>, b: Option<&str>) -> bool {
    fn norm(v: &str) -> String {
        v.trim().trim_matches('"').trim().to_ascii_lowercase()
    }
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            let (a, b) = (norm(a), norm(b));
            if a == b {
                return true;
            }
            match (a.parse::<f64>(), b.parse::<f64>()) {
                (Ok(x), Ok(y)) => (x - y).abs() < 1e-6,
                _ => false,
            }
        }
        _ => false,
    }
}

impl SettingsWatch {
    pub fn load(path: &Path) -> SettingsWatch {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap_or_default())
    }

    /// Compares `current` (values read from the emulator's files) with the
    /// baseline. Returns the keys newly found to be changed by the user.
    pub fn detect(&mut self, current: &BTreeMap<String, String>, keys: &[String]) -> Vec<String> {
        let mut found = Vec::new();
        for k in keys {
            if self.user_changed.contains(k) {
                continue;
            }
            if let Some(base) = self.baseline.get(k) {
                if !same_value(base.as_deref(), current.get(k).map(String::as_str)) {
                    self.user_changed.insert(k.clone());
                    found.push(k.clone());
                }
            }
        }
        found
    }

    /// Records the values now in place as the new baseline (keys owned by
    /// the user are skipped).
    pub fn record(&mut self, current: &BTreeMap<String, String>, keys: &[String]) {
        for k in keys {
            if !self.user_changed.contains(k) {
                self.baseline.insert(k.clone(), current.get(k).cloned());
            }
        }
    }

    /// The user gives the keys back to the automatic optimisation.
    pub fn reset(&mut self) {
        self.baseline.clear();
        self.user_changed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn values_compare_like_emulators_write_them() {
        assert!(same_value(Some("3"), Some("3.000000")));
        assert!(same_value(Some("\"true\""), Some("True")));
        assert!(!same_value(Some("2"), Some("3")));
        assert!(!same_value(None, Some("1")));
    }

    #[test]
    fn detects_user_changes_and_resets() {
        let keys = vec!["res".to_string(), "api".to_string()];
        let mut w = SettingsWatch::default();
        // First sight: nothing to compare yet.
        assert!(w.detect(&map(&[("res", "1")]), &keys).is_empty());
        w.record(&map(&[("res", "3")]), &keys);
        // Same values (written by the launcher) → nothing.
        assert!(w.detect(&map(&[("res", "3.0")]), &keys).is_empty());
        // The user sets 2 and picks an API that was absent.
        let found = w.detect(&map(&[("res", "2"), ("api", "Vulkan")]), &keys);
        assert_eq!(found, vec!["res", "api"]);
        w.record(&map(&[("res", "2")]), &keys);
        assert_eq!(w.baseline.get("res"), Some(&Some("3".to_string())));
        // Reported once, kept afterwards.
        assert!(w.detect(&map(&[("res", "4")]), &keys).is_empty());
        assert_eq!(w.user_changed.len(), 2);
        w.reset();
        assert!(w.user_changed.is_empty() && w.baseline.is_empty());
    }
}
