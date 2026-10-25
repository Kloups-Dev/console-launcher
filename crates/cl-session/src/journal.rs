//! Bounded local records describing emulator sessions.

use cl_perf::frametime::FrameStats;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const MAX_RECORDS: usize = 50;
const MAX_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub status: String,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub game_id: String,
    pub game_title: String,
    pub system: String,
    pub emulator: String,
    pub profile_step: usize,
    pub profile_label: String,
    pub hardware: String,
    pub measure_requested: bool,
    pub capture_produced: bool,
    pub capture_note: Option<String>,
    pub exit_code: Option<i32>,
    pub played_seconds: Option<u64>,
    pub verdict: Option<String>,
    pub frame_stats: Option<FrameStats>,
    pub error: Option<String>,
}

/// Writes one session record atomically and removes oldest records over the
/// count or size budget. The journal contains no paths or input events.
pub fn write_record(dir: &Path, record: &SessionRecord) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{}.json", record.session_id));
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_vec_pretty(record).map_err(std::io::Error::other)?;
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, &path)?;
    rotate(dir)?;
    Ok(path)
}

fn rotate(dir: &Path) -> std::io::Result<()> {
    let mut files: Vec<(std::time::SystemTime, u64, PathBuf)> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension()?.eq_ignore_ascii_case("json")).then_some(path)
        })
        .filter_map(|path| {
            let metadata = std::fs::metadata(&path).ok()?;
            Some((
                metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
                metadata.len(),
                path,
            ))
        })
        .collect();
    files.sort_by_key(|(modified, _, path)| (*modified, path.clone()));

    let mut total = files.iter().map(|(_, size, _)| *size).sum::<u64>();
    while files.len() > MAX_RECORDS || total > MAX_BYTES {
        let Some((_, size, path)) = files.first() else {
            break;
        };
        let path = path.clone();
        total = total.saturating_sub(*size);
        files.remove(0);
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str) -> SessionRecord {
        SessionRecord {
            session_id: id.into(),
            game_id: "game".into(),
            ..Default::default()
        }
    }

    #[test]
    fn writes_json_and_keeps_journal_bounded() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..55 {
            write_record(dir.path(), &record(&format!("{i:03}"))).unwrap();
        }
        let count = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .count();
        assert_eq!(count, MAX_RECORDS);
        let saved: SessionRecord =
            serde_json::from_str(&std::fs::read_to_string(dir.path().join("054.json")).unwrap())
                .unwrap();
        assert_eq!(saved.game_id, "game");
    }
}
