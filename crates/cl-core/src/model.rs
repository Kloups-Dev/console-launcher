use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

/// A console family the launcher knows how to present and route to an emulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemId {
    Nes,
    Snes,
    N64,
    Gb,
    Gbc,
    Gba,
    Nds,
    MasterSystem,
    MegaDrive,
    Psx,
    Ps2,
    Psp,
    GameCube,
    Wii,
    WiiU,
}

/// Static description of a system.
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub id: SystemId,
    /// Display name.
    pub name: &'static str,
    pub manufacturer: &'static str,
    pub year: u16,
    /// Folder names accepted for this system inside a ROM directory
    /// (EmulationStation-style layout: `roms/ps2/game.iso`).
    pub folder_names: &'static [&'static str],
    /// File extensions (lowercase, without dot).
    pub extensions: &'static [&'static str],
    /// Relative emulation cost, 1 (light) to 10 (very heavy). Used by the
    /// estimator as a starting point only; real tests decide in the end.
    pub cost: u8,
}

impl SystemId {
    pub const ALL: [SystemId; 15] = [
        SystemId::Nes,
        SystemId::Snes,
        SystemId::N64,
        SystemId::Gb,
        SystemId::Gbc,
        SystemId::Gba,
        SystemId::Nds,
        SystemId::MasterSystem,
        SystemId::MegaDrive,
        SystemId::Psx,
        SystemId::Ps2,
        SystemId::Psp,
        SystemId::GameCube,
        SystemId::Wii,
        SystemId::WiiU,
    ];

    pub fn info(self) -> SystemInfo {
        use SystemId::*;
        let (name, manufacturer, year, folder_names, extensions, cost): (
            &'static str,
            &'static str,
            u16,
            &'static [&'static str],
            &'static [&'static str],
            u8,
        ) = match self {
            Nes => (
                "Nintendo Entertainment System",
                "Nintendo",
                1983,
                &["nes", "famicom"],
                &["nes", "fds", "unf", "unif"],
                1,
            ),
            Snes => (
                "Super Nintendo",
                "Nintendo",
                1990,
                &["snes", "sfc", "superfamicom"],
                &["sfc", "smc"],
                1,
            ),
            N64 => (
                "Nintendo 64",
                "Nintendo",
                1996,
                &["n64"],
                &["n64", "z64", "v64"],
                4,
            ),
            Gb => ("Game Boy", "Nintendo", 1989, &["gb", "gameboy"], &["gb"], 1),
            Gbc => ("Game Boy Color", "Nintendo", 1998, &["gbc"], &["gbc"], 1),
            Gba => ("Game Boy Advance", "Nintendo", 2001, &["gba"], &["gba"], 2),
            Nds => ("Nintendo DS", "Nintendo", 2004, &["nds"], &["nds"], 3),
            MasterSystem => (
                "Master System",
                "Sega",
                1985,
                &["mastersystem", "sms"],
                &["sms"],
                1,
            ),
            MegaDrive => (
                "Mega Drive / Genesis",
                "Sega",
                1988,
                &["megadrive", "genesis", "md"],
                &["md", "gen", "smd"],
                1,
            ),
            Psx => (
                "PlayStation",
                "Sony",
                1994,
                &["psx", "ps1", "playstation"],
                &["cue", "chd", "pbp", "m3u"],
                2,
            ),
            Ps2 => (
                "PlayStation 2",
                "Sony",
                2000,
                &["ps2"],
                &["iso", "chd", "cso", "gz"],
                7,
            ),
            Psp => (
                "PlayStation Portable",
                "Sony",
                2004,
                &["psp"],
                &["cso", "iso", "pbp"],
                4,
            ),
            GameCube => (
                "GameCube",
                "Nintendo",
                2001,
                &["gc", "gamecube", "ngc"],
                &["iso", "gcm", "rvz", "gcz", "ciso"],
                7,
            ),
            Wii => (
                "Wii",
                "Nintendo",
                2006,
                &["wii"],
                &["iso", "rvz", "wbfs", "wad", "gcz"],
                8,
            ),
            WiiU => (
                "Wii U",
                "Nintendo",
                2012,
                &["wiiu"],
                &["wud", "wux", "wua", "rpx"],
                8,
            ),
        };
        SystemInfo {
            id: self,
            name,
            manufacturer,
            year,
            folder_names,
            extensions,
            cost,
        }
    }

    pub fn as_str(self) -> &'static str {
        use SystemId::*;
        match self {
            Nes => "nes",
            Snes => "snes",
            N64 => "n64",
            Gb => "gb",
            Gbc => "gbc",
            Gba => "gba",
            Nds => "nds",
            MasterSystem => "mastersystem",
            MegaDrive => "megadrive",
            Psx => "psx",
            Ps2 => "ps2",
            Psp => "psp",
            GameCube => "gamecube",
            Wii => "wii",
            WiiU => "wiiu",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<SystemId> {
        let s = s.to_ascii_lowercase();
        SystemId::ALL
            .into_iter()
            .find(|id| id.as_str() == s || id.info().folder_names.contains(&s.as_str()))
    }
}

impl fmt::Display for SystemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A game found in the user's folders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    /// Stable identifier: system + relative path, hashed.
    pub id: String,
    pub system: SystemId,
    pub title: String,
    pub path: PathBuf,
    /// Optional cover image found next to the game or in a `media` folder.
    pub cover: Option<PathBuf>,
    pub favorite: bool,
}

/// Coarse hardware class. Real tests refine the choice per game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Low,
    Medium,
    High,
    Ultra,
}

impl Tier {
    pub fn index(self) -> usize {
        self as usize
    }
}

/// Graphics API preference reported to adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphicsApi {
    Vulkan,
    D3d11,
    D3d12,
    OpenGl,
}

/// One rung of a quality ladder: a named set of emulator settings.
/// Rungs go from lightest (index 0) to heaviest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityStep {
    /// Short label shown to the user, e.g. "720p", "2x native".
    pub label: String,
    /// Adapter-specific settings, already in the emulator's vocabulary.
    /// Key format is defined by each adapter (e.g. `GFX.Settings.InternalResolution`).
    pub settings: BTreeMap<String, String>,
}

/// The settings chosen for one game (or one system by default).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub emulator: String,
    pub system: SystemId,
    /// Index into the adapter's quality ladder.
    pub step: usize,
    pub step_label: String,
    pub api: GraphicsApi,
    /// Where the choice came from.
    pub origin: ProfileOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileOrigin {
    /// Estimated from hardware detection and the micro-benchmark.
    Estimated,
    /// Confirmed or adjusted by a real in-game measurement.
    Measured,
    /// Fixed by the user.
    User,
}
