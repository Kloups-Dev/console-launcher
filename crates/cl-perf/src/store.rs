//! Per-game tuning files (`profiles/<game id>.json`).

use crate::frametime::{FrameStats, Verdict};
use crate::tuner::TuneState;
use cl_core::{GraphicsApi, SystemId};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    /// Unix time, seconds.
    pub at: u64,
    pub step: usize,
    pub stats: FrameStats,
    pub verdict: Verdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTuning {
    pub game_id: String,
    pub system: SystemId,
    pub emulator: String,
    /// Emulator version the measurements were made with, when known.
    pub emulator_version: Option<String>,
    /// Hardware fingerprint (CPU + GPU names); a change restarts the search.
    pub hardware: String,
    pub api: GraphicsApi,
    pub state: TuneState,
    pub history: Vec<RunRecord>,
}

pub struct ProfileStore {
    dir: PathBuf,
}

impl ProfileStore {
    pub fn new(dir: impl Into<PathBuf>) -> ProfileStore {
        ProfileStore { dir: dir.into() }
    }

    fn file(&self, game_id: &str) -> PathBuf {
        let safe: String = game_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        self.dir.join(format!("{safe}.json"))
    }

    pub fn load(&self, game_id: &str) -> Option<GameTuning> {
        let text = std::fs::read_to_string(self.file(game_id)).ok()?;
        serde_json::from_str(&text).ok()
    }

    pub fn save(&self, t: &GameTuning) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.file(&t.game_id);
        let tmp = path.with_extension("json.tmp");
        std::fs::write(
            &tmp,
            serde_json::to_string_pretty(t).map_err(std::io::Error::other)?,
        )?;
        std::fs::rename(tmp, path)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}

impl GameTuning {
    /// Keeps the last 20 runs so the file stays small.
    pub fn push_run(&mut self, record: RunRecord) {
        self.state.record(record.verdict);
        self.history.push(record);
        let excess = self.history.len().saturating_sub(20);
        self.history.drain(..excess);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = ProfileStore::new(dir.path());
        let t = GameTuning {
            game_id: "abc123".into(),
            system: SystemId::Ps2,
            emulator: "pcsx2".into(),
            emulator_version: None,
            hardware: "cpu|gpu".into(),
            api: GraphicsApi::Vulkan,
            state: TuneState::new(2, 5),
            history: vec![],
        };
        store.save(&t).unwrap();
        assert_eq!(store.load("abc123").unwrap().state, t.state);
        assert!(store.load("../etc/passwd").is_none());
    }
}
