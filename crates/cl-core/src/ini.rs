//! Minimal INI editor that keeps the original file intact (comments, order,
//! unknown keys) and only changes the keys the launcher owns.
//!
//! Ownership rule: the launcher remembers, in a ledger, the last value it
//! wrote for each key. If the value found in the file differs from the
//! ledger, the user changed it by hand and the launcher leaves it alone.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IniDoc {
    lines: Vec<String>,
}

fn eq_ci(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

fn section_name(line: &str) -> Option<&str> {
    let t = line.trim();
    if t.starts_with('[') && t.ends_with(']') && t.len() >= 2 {
        Some(&t[1..t.len() - 1])
    } else {
        None
    }
}

fn split_kv(line: &str) -> Option<(&str, &str)> {
    let t = line.trim_start();
    if t.starts_with(';') || t.starts_with('#') {
        return None;
    }
    let pos = line.find('=')?;
    Some((line[..pos].trim(), line[pos + 1..].trim()))
}

impl IniDoc {
    pub fn parse(text: &str) -> IniDoc {
        IniDoc {
            lines: text.lines().map(str::to_owned).collect(),
        }
    }

    /// Returns `(section_start, section_end)` line indexes; `start` is the header line.
    fn section_range(&self, section: &str) -> Option<(usize, usize)> {
        let start = self
            .lines
            .iter()
            .position(|l| section_name(l).is_some_and(|s| eq_ci(s, section)))?;
        let end = self.lines[start + 1..]
            .iter()
            .position(|l| section_name(l).is_some())
            .map(|p| start + 1 + p)
            .unwrap_or(self.lines.len());
        Some((start, end))
    }

    fn find_key(&self, section: &str, key: &str) -> Option<usize> {
        let (start, end) = self.section_range(section)?;
        (start + 1..end).find(|&i| split_kv(&self.lines[i]).is_some_and(|(k, _)| eq_ci(k, key)))
    }

    pub fn get(&self, section: &str, key: &str) -> Option<String> {
        self.find_key(section, key)
            .and_then(|i| split_kv(&self.lines[i]).map(|(_, v)| v.to_owned()))
    }

    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        if let Some(i) = self.find_key(section, key) {
            let line = &self.lines[i];
            let eq = line.find('=').expect("key line has '='");
            // Keep the original key spelling and spacing style.
            let spaced = line[eq + 1..].starts_with(' ');
            let left = &line[..eq + 1];
            self.lines[i] = if spaced {
                format!("{left} {value}")
            } else {
                format!("{left}{value}")
            };
            return;
        }
        match self.section_range(section) {
            Some((_, end)) => {
                // Insert after the last non-blank line of the section.
                let mut at = end;
                while at > 0 && self.lines[at - 1].trim().is_empty() {
                    at -= 1;
                }
                self.lines.insert(at, format!("{key} = {value}"));
            }
            None => {
                if self.lines.last().is_some_and(|l| !l.trim().is_empty()) {
                    self.lines.push(String::new());
                }
                self.lines.push(format!("[{section}]"));
                self.lines.push(format!("{key} = {value}"));
            }
        }
    }

    pub fn render(&self) -> String {
        let mut out = self.lines.join("\n");
        out.push('\n');
        out
    }
}

/// Values last written by the launcher, keyed by `Section/Key`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    pub written: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyReport {
    /// Keys changed by this call.
    pub applied: Vec<String>,
    /// Keys left untouched because the user edited them or locked them.
    pub kept_user_value: Vec<String>,
}

/// Applies `changes` (`Section/Key` → value) to `doc` following the ownership rule.
/// The key is the part after the last `/`, so sections may contain `/`
/// (PCSX2 uses `[EmuCore/GS]`).
pub fn apply_owned(
    doc: &mut IniDoc,
    changes: &BTreeMap<String, String>,
    ledger: &mut Ledger,
    locked_keys: &[String],
) -> ApplyReport {
    let mut report = ApplyReport::default();
    for (full_key, value) in changes {
        let Some((section, key)) = full_key.rsplit_once('/') else {
            continue;
        };
        if locked_keys.iter().any(|k| eq_ci(k, full_key)) {
            report.kept_user_value.push(full_key.clone());
            continue;
        }
        let current = doc.get(section, key);
        let user_changed = match (ledger.written.get(full_key), &current) {
            (Some(ours), Some(cur)) => !eq_ci(ours, cur),
            _ => false,
        };
        if user_changed {
            report.kept_user_value.push(full_key.clone());
            continue;
        }
        if current.as_deref() != Some(value.as_str()) {
            doc.set(section, key, value);
            report.applied.push(full_key.clone());
        }
        ledger.written.insert(full_key.clone(), value.clone());
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "; comment kept\n[Settings]\nInternalResolution = 1\nMSAA = 1\n\n[Hacks]\nEFBToTextureEnable = True\n";

    fn changes(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn preserves_layout_and_comments() {
        let mut doc = IniDoc::parse(SAMPLE);
        doc.set("Settings", "InternalResolution", "3");
        let out = doc.render();
        assert!(out.starts_with("; comment kept\n[Settings]\nInternalResolution = 3\n"));
        assert!(out.contains("[Hacks]\nEFBToTextureEnable = True"));
    }

    #[test]
    fn adds_missing_key_and_section() {
        let mut doc = IniDoc::parse(SAMPLE);
        doc.set("settings", "ShaderCompilationMode", "2");
        doc.set("Enhancements", "MaxAnisotropy", "2");
        let out = doc.render();
        assert!(out.contains("MSAA = 1\nShaderCompilationMode = 2\n\n[Hacks]"));
        assert!(out.ends_with("[Enhancements]\nMaxAnisotropy = 2\n"));
    }

    #[test]
    fn keeps_values_the_user_changed() {
        let mut doc = IniDoc::parse(SAMPLE);
        let mut ledger = Ledger::default();
        let r = apply_owned(
            &mut doc,
            &changes(&[("Settings/InternalResolution", "2")]),
            &mut ledger,
            &[],
        );
        assert_eq!(r.applied, vec!["Settings/InternalResolution"]);

        // The user edits the value by hand in the emulator.
        doc.set("Settings", "InternalResolution", "4");
        let r = apply_owned(
            &mut doc,
            &changes(&[("Settings/InternalResolution", "1")]),
            &mut ledger,
            &[],
        );
        assert!(r.applied.is_empty());
        assert_eq!(r.kept_user_value, vec!["Settings/InternalResolution"]);
        assert_eq!(
            doc.get("Settings", "InternalResolution").as_deref(),
            Some("4")
        );
    }

    #[test]
    fn locked_keys_are_never_written() {
        let mut doc = IniDoc::parse(SAMPLE);
        let mut ledger = Ledger::default();
        let r = apply_owned(
            &mut doc,
            &changes(&[("Settings/MSAA", "4")]),
            &mut ledger,
            &["settings/msaa".to_string()],
        );
        assert!(r.applied.is_empty());
        assert_eq!(doc.get("Settings", "MSAA").as_deref(), Some("1"));
    }
}
