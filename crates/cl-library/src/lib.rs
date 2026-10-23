//! Game library.
//!
//! Expected layout (same as EmulationStation / RetroBat / Batocera, so an
//! existing collection works as is):
//!
//! ```text
//! <rom_dir>/
//!   ps2/  Game A.iso
//!   snes/ Game B.sfc
//!   snes/media/covers/Game B.png   (optional cover)
//! ```
//!
//! A file outside a system folder is accepted only when it sits directly in
//! the game folder and its extension belongs to a single system (`.sfc` →
//! SNES, but `.iso` is ambiguous). Emulator, BIOS and save folders are
//! skipped, so pointing the launcher at a whole RetroBat folder works.

use cl_core::{Game, SystemId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MAX_DEPTH: usize = 4;

/// Folders that never contain games (emulators, firmware, saves, media…).
const SKIPPED_DIRS: [&str; 22] = [
    "media",
    "images",
    "videos",
    "manuals",
    "emulators",
    "emulator",
    "bios",
    "saves",
    "savestates",
    "states",
    "system",
    "cores",
    "shaders",
    "cheats",
    "screenshots",
    "themes",
    "config",
    "configs",
    "resources",
    "plugins",
    "downloaded_images",
    "downloaded_videos",
];

fn skipped_dir(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.starts_with('.') || SKIPPED_DIRS.contains(&n.as_str())
}
const COVER_EXTS: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

/// Stable identifier (FNV-1a 64-bit) so favourites and profiles survive rescans.
pub fn game_id(system: SystemId, rel_path: &Path) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let key = format!(
        "{}:{}",
        system,
        rel_path.to_string_lossy().replace('\\', "/")
    );
    for b in key.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// Turns "Super Game (Europe) (En,Fr) [!]" into "Super Game".
pub fn clean_title(stem: &str) -> String {
    let mut out = String::with_capacity(stem.len());
    let mut depth = 0i32;
    for c in stem.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = (depth - 1).max(0),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    let t = out.replace('_', " ");
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.is_empty() {
        stem.to_owned()
    } else {
        t
    }
}

fn ext_of(p: &Path) -> Option<String> {
    p.extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
}

/// System given by a folder name in the first two levels
/// (`snes/game.sfc` or `roms/snes/game.sfc`).
fn system_from_folders(rel: &Path) -> Option<SystemId> {
    rel.parent()?
        .components()
        .take(2)
        .filter_map(|c| c.as_os_str().to_str())
        .find_map(SystemId::from_str_loose)
}

fn system_from_extension(ext: &str) -> Option<SystemId> {
    let mut matches = SystemId::ALL
        .into_iter()
        .filter(|s| s.info().extensions.contains(&ext));
    let first = matches.next()?;
    matches.next().is_none().then_some(first)
}

fn find_cover(path: &Path, system_root: Option<&Path>) -> Option<PathBuf> {
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    let mut dirs = vec![path.parent()?.to_path_buf()];
    if let Some(root) = system_root {
        for sub in ["media/covers", "media/box2dfront", "images"] {
            dirs.push(root.join(sub));
        }
    }
    dirs.iter()
        .flat_map(|d| COVER_EXTS.iter().map(|e| d.join(format!("{stem}.{e}"))))
        .find(|p| p.is_file())
}

/// Scans the given folders. Media folders (`media`, `images`) are skipped.
pub fn scan(rom_dirs: &[PathBuf]) -> Vec<Game> {
    let mut games = Vec::new();
    for root in rom_dirs {
        // Folders that hold an .m3u playlist: their disc files are not listed
        // separately, the playlist stands for the whole multi-disc game.
        let mut m3u_dirs: HashSet<PathBuf> = HashSet::new();
        let entries: Vec<_> = WalkDir::new(root)
            .max_depth(MAX_DEPTH)
            .into_iter()
            .filter_entry(|e| {
                e.depth() == 0
                    || !(e.file_type().is_dir() && skipped_dir(&e.file_name().to_string_lossy()))
            })
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .collect();
        for e in &entries {
            if ext_of(e.path()).as_deref() == Some("m3u") {
                if let Some(p) = e.path().parent() {
                    m3u_dirs.insert(p.to_path_buf());
                }
            }
        }
        for e in entries {
            let path = e.path();
            let Some(ext) = ext_of(path) else { continue };
            let Ok(rel) = path.strip_prefix(root) else {
                continue;
            };
            let system = match system_from_folders(rel) {
                Some(s) if s.info().extensions.contains(&ext.as_str()) => s,
                Some(_) => continue,
                // Loose files only directly in the folder.
                None if rel.components().count() == 1 => match system_from_extension(&ext) {
                    Some(s) => s,
                    None => continue,
                },
                None => continue,
            };
            if ext != "m3u" && path.parent().is_some_and(|p| m3u_dirs.contains(p)) {
                continue;
            }
            let system_root = rel
                .components()
                .next()
                .filter(|c| {
                    c.as_os_str().to_str().and_then(SystemId::from_str_loose) == Some(system)
                })
                .map(|c| root.join(c.as_os_str()));
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            games.push(Game {
                id: game_id(system, rel),
                system,
                title: clean_title(&stem),
                path: path.to_path_buf(),
                cover: find_cover(path, system_root.as_deref()),
                favorite: false,
            });
        }
    }
    games.sort_by(|a, b| {
        (a.system, a.title.to_lowercase()).cmp(&(b.system, b.title.to_lowercase()))
    });
    games
}

/// Games grouped by system, in the order shown by the interface.
pub fn group_by_system(games: &[Game]) -> BTreeMap<SystemId, Vec<Game>> {
    let mut map: BTreeMap<SystemId, Vec<Game>> = BTreeMap::new();
    for g in games {
        map.entry(g.system).or_default().push(g.clone());
    }
    map
}

/// Favourites, stored as a set of game ids in `favorites.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Favorites {
    pub ids: BTreeSet<String>,
}

impl Favorites {
    pub fn load(path: &Path) -> Favorites {
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

    pub fn toggle(&mut self, id: &str) -> bool {
        if !self.ids.remove(id) {
            self.ids.insert(id.to_owned());
            true
        } else {
            false
        }
    }

    pub fn apply(&self, games: &mut [Game]) {
        for g in games {
            g.favorite = self.ids.contains(&g.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn touch(p: &Path) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, b"x").unwrap();
    }

    #[test]
    fn cleans_titles() {
        assert_eq!(clean_title("Super Game (Europe) (En,Fr) [!]"), "Super Game");
        assert_eq!(clean_title("my_game_2"), "my game 2");
        assert_eq!(clean_title("(Demo)"), "(Demo)");
    }

    #[test]
    fn scans_system_folders_and_unique_extensions() {
        let dir = tempfile::tempdir().unwrap();
        let r = dir.path();
        touch(&r.join("ps2/Game A (USA).iso"));
        touch(&r.join("gamecube/Game B.rvz"));
        touch(&r.join("snes/Game C.sfc"));
        touch(&r.join("snes/media/covers/Game C.png"));
        touch(&r.join("snes/readme.txt"));
        touch(&r.join("loose.gba")); // unique extension, no folder
        touch(&r.join("loose.iso")); // ambiguous, ignored
        touch(&r.join("psx/Multi/Multi (Disc 1).cue"));
        touch(&r.join("psx/Multi/Multi (Disc 2).cue"));
        touch(&r.join("psx/Multi/Multi.m3u"));

        // Not games: emulator files, deep loose files.
        touch(&r.join("emulators/pcsx2/resources/a.gz"));
        touch(&r.join("emulators/ppsspp/assets/psp/b.iso"));
        touch(&r.join("misc/deep/c.gba"));

        let games = scan(&[r.to_path_buf()]);
        let names: Vec<_> = games.iter().map(|g| (g.system, g.title.as_str())).collect();
        assert_eq!(
            names,
            vec![
                (SystemId::Snes, "Game C"),
                (SystemId::Gba, "loose"),
                (SystemId::Psx, "Multi"),
                (SystemId::Ps2, "Game A"),
                (SystemId::GameCube, "Game B"),
            ]
        );
        let snes = games.iter().find(|g| g.system == SystemId::Snes).unwrap();
        assert!(snes
            .cover
            .as_ref()
            .unwrap()
            .ends_with("media/covers/Game C.png"));
    }

    #[test]
    fn whole_retrobat_folder_finds_only_games() {
        let dir = tempfile::tempdir().unwrap();
        let r = dir.path();
        touch(&r.join("roms/gba/Pokemon.gba"));
        touch(&r.join("roms/ps2/Game.iso"));
        touch(&r.join("emulators/retroarch/cores/x.gz"));
        touch(&r.join("bios/scph.bin"));
        let games = scan(&[r.to_path_buf()]);
        let names: Vec<_> = games.iter().map(|g| g.title.as_str()).collect();
        assert_eq!(names, vec!["Pokemon", "Game"]);
    }

    #[test]
    fn ids_are_stable_and_favorites_round_trip() {
        let a = game_id(SystemId::Ps2, Path::new("ps2/a.iso"));
        assert_eq!(a, game_id(SystemId::Ps2, Path::new("ps2\\a.iso")));
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("fav.json");
        let mut f = Favorites::default();
        assert!(f.toggle(&a));
        f.save(&file).unwrap();
        assert!(Favorites::load(&file).ids.contains(&a));
    }
}
