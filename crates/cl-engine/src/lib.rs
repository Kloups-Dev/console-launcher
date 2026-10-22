//! The engine used by the interface and by `clctl`.
//!
//! It keeps the interface free of emulator details: the interface asks for
//! systems and games, then calls [`Engine::play`]; the engine chooses the
//! emulator, the settings, measures when needed and adjusts the profile.

use cl_adapters::{EmulatorAdapter, LaunchContext, LaunchPlan};
use cl_core::config::{AppConfig, EmulatorConfig, EmulatorMode};
use cl_core::paths::Paths;
use cl_core::{Game, SystemId, SystemInfo};
use cl_hwprobe::{GpuKind, HardwareReport};
use cl_perf::estimate::{estimate, GpuClass, HwSummary, LadderKind};
use cl_perf::frametime::{analyse, parse_file, verdict, FrameStats, Verdict};
use cl_perf::store::{GameTuning, ProfileStore, RunRecord};
use cl_perf::tuner::TuneState;
use cl_session::{CaptureRequest, SessionResult};
use cl_updater::ManagedEmulator;
use serde::Serialize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Config(#[from] cl_core::config::ConfigError),
    #[error("unknown emulator '{0}'")]
    UnknownEmulator(String),
    #[error(
        "{emulator} was not found. Set its path in the settings or let the launcher install it."
    )]
    EmulatorMissing { emulator: String },
    #[error(transparent)]
    Adapter(#[from] cl_adapters::AdapterError),
    #[error(transparent)]
    Session(#[from] cl_session::SessionError),
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemEntry {
    pub info: SystemInfo,
    pub games: usize,
    pub emulator: String,
    pub emulator_found: bool,
}

/// What will happen when a game is started.
#[derive(Debug, Clone, Serialize)]
pub struct PreparedLaunch {
    pub game: Game,
    pub emulator: String,
    pub step: usize,
    pub step_label: String,
    pub ladder: Vec<String>,
    pub measure: bool,
    pub plan: LaunchPlan,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayReport {
    pub session: SessionResult,
    pub stats: Option<FrameStats>,
    pub verdict: Option<Verdict>,
    /// Rung for the next launch, and its label.
    pub next_step: usize,
    pub next_label: String,
    pub tuning_done: bool,
    pub notes: Vec<String>,
}

/// Adapter, its settings and the executable found (if any).
pub type FoundEmulator = (Box<dyn EmulatorAdapter>, EmulatorConfig, Option<PathBuf>);

pub struct Engine {
    pub paths: Paths,
    pub config: AppConfig,
    hardware: Option<HardwareReport>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Engine {
    pub fn load(paths: Paths) -> Result<Engine, EngineError> {
        let config = AppConfig::load(&paths.config_file())?;
        let hardware = std::fs::read_to_string(paths.root.join("hardware.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok());
        Ok(Engine {
            paths,
            config,
            hardware,
        })
    }

    pub fn save_config(&self) -> Result<(), EngineError> {
        self.config.save(&self.paths.config_file())?;
        Ok(())
    }

    /// Hardware report, measured once and cached in `hardware.json`.
    pub fn hardware(&mut self, refresh: bool) -> &HardwareReport {
        if refresh || self.hardware.is_none() {
            let report = cl_hwprobe::probe(true);
            let _ = std::fs::create_dir_all(&self.paths.root);
            if let Ok(t) = serde_json::to_string_pretty(&report) {
                let _ = std::fs::write(self.paths.root.join("hardware.json"), t);
            }
            self.hardware = Some(report);
        }
        self.hardware.as_ref().expect("set above")
    }

    pub fn hw_summary(&mut self) -> HwSummary {
        let hw = self.hardware(false).clone();
        let gpu = hw.main_gpu();
        HwSummary {
            cpu_score: hw.bench.as_ref().map(|b| b.score).unwrap_or(500),
            gpu_class: match gpu.map(|g| g.kind) {
                Some(GpuKind::Discrete) => GpuClass::Discrete,
                Some(GpuKind::Integrated) => GpuClass::Integrated,
                _ => GpuClass::None,
            },
            vram_mb: gpu.map(|g| g.vram_mb).unwrap_or(0),
            has_vulkan: gpu.is_some_and(|g| g.kind != GpuKind::Cpu),
            ram_mb: hw.ram_mb,
        }
    }

    fn hw_fingerprint(&mut self) -> String {
        let hw = self.hardware(false);
        format!(
            "{}|{}",
            hw.cpu.brand,
            hw.main_gpu().map(|g| g.name.as_str()).unwrap_or("none")
        )
    }

    pub fn games(&self) -> Vec<Game> {
        let mut games = cl_library::scan(&self.config.rom_dirs);
        cl_library::Favorites::load(&self.paths.root.join("favorites.json")).apply(&mut games);
        games
    }

    pub fn toggle_favorite(&self, game_id: &str) -> std::io::Result<bool> {
        let file = self.paths.root.join("favorites.json");
        let mut f = cl_library::Favorites::load(&file);
        let on = f.toggle(game_id);
        f.save(&file)?;
        Ok(on)
    }

    pub fn emulator_id_for(&self, system: SystemId) -> String {
        self.config
            .system_emulator
            .get(&system)
            .cloned()
            .unwrap_or_else(|| cl_adapters::default_emulator(system).to_owned())
    }

    fn managed(&self, id: &str) -> ManagedEmulator {
        ManagedEmulator::new(&self.paths.emulators_dir(), id)
    }

    pub fn find_emulator(&self, id: &str) -> Result<FoundEmulator, EngineError> {
        let adapter =
            cl_adapters::by_id(id).ok_or_else(|| EngineError::UnknownEmulator(id.into()))?;
        let cfg = self.config.emulator(id);
        let managed = (cfg.mode == EmulatorMode::Managed)
            .then(|| self.managed(id).active_dir())
            .flatten();
        let exe = adapter.find_executable(&cfg, managed.as_deref());
        Ok((adapter, cfg, exe))
    }

    pub fn systems(&self, games: &[Game]) -> Vec<SystemEntry> {
        let grouped = cl_library::group_by_system(games);
        grouped
            .into_iter()
            .map(|(system, list)| {
                let emulator = self.emulator_id_for(system);
                let emulator_found = self
                    .find_emulator(&emulator)
                    .map(|(_, _, exe)| exe.is_some())
                    .unwrap_or(false);
                SystemEntry {
                    info: system.info(),
                    games: list.len(),
                    emulator,
                    emulator_found,
                }
            })
            .collect()
    }

    fn profiles(&self) -> ProfileStore {
        ProfileStore::new(self.paths.profiles_dir())
    }

    /// Loads the game's tuning, or creates it from the estimate. A hardware
    /// change restarts the search from a fresh estimate.
    pub fn tuning(&mut self, game: &Game, adapter: &dyn EmulatorAdapter) -> GameTuning {
        let ladder_len = adapter.quality_ladder(game.system).len();
        let fingerprint = self.hw_fingerprint();
        if let Some(t) = self.profiles().load(&game.id) {
            if t.hardware == fingerprint
                && t.emulator == adapter.id()
                && t.state.max + 1 == ladder_len
            {
                return t;
            }
        }
        let kind = if adapter.gpu_bound_ladder() {
            LadderKind::Resolution
        } else {
            LadderKind::Latency
        };
        let est = estimate(
            &self.hw_summary(),
            game.system,
            kind,
            ladder_len,
            &adapter.apis(),
        );
        GameTuning {
            game_id: game.id.clone(),
            system: game.system,
            emulator: adapter.id().into(),
            emulator_version: None,
            hardware: fingerprint,
            api: est.api,
            state: TuneState::new(est.step, ladder_len),
            history: Vec::new(),
        }
    }

    pub fn prepare(&mut self, game: &Game) -> Result<(PreparedLaunch, GameTuning), EngineError> {
        let emulator = self.emulator_id_for(game.system);
        let (adapter, cfg, exe) = self.find_emulator(&emulator)?;
        let exe = exe.ok_or_else(|| EngineError::EmulatorMissing {
            emulator: adapter.name().into(),
        })?;
        let tuning = self.tuning(game, adapter.as_ref());
        let ladder = adapter.quality_ladder(game.system);
        let step = &ladder[tuning.state.current.min(ladder.len() - 1)];

        let mut notes = Vec::new();
        let mut measure = self.config.perf.auto_tune && !tuning.state.done;
        if measure {
            if let Err(e) =
                cl_session::capture_tool_available(self.config.perf.presentmon_path.as_deref())
            {
                notes.push(format!("Automatic optimisation paused: {e}."));
                measure = false;
            }
        }
        let generated = self.paths.generated_dir();
        let mut plan = adapter.prepare(&LaunchContext {
            exe: &exe,
            game,
            step,
            api: tuning.api,
            mode: cfg.mode,
            generated_dir: &generated,
            locked_keys: &cfg.locked_keys,
            bios_dir: self.config.bios_dir.as_deref(),
        })?;
        if measure {
            let extra = adapter.measurement_args();
            // Overrides go before the game argument.
            let at = plan.args.len().saturating_sub(1);
            for (i, a) in extra.into_iter().enumerate() {
                plan.args.insert(at + i, a);
            }
        }
        notes.extend(plan.notes.iter().cloned());
        Ok((
            PreparedLaunch {
                game: game.clone(),
                emulator: adapter.id().into(),
                step: tuning.state.current,
                step_label: step.label.clone(),
                ladder: ladder.iter().map(|s| s.label.clone()).collect(),
                measure,
                plan,
                notes,
            },
            tuning,
        ))
    }

    /// Plays a game (blocking until the emulator closes) and updates its profile.
    pub fn play(&mut self, game: &Game) -> Result<PlayReport, EngineError> {
        let (prepared, mut tuning) = self.prepare(game)?;
        let capture = prepared.measure.then(|| CaptureRequest {
            dir: self.paths.captures_dir().join(&game.id),
            delay_seconds: self.config.perf.capture_delay_seconds,
            seconds: self.config.perf.capture_seconds,
            presentmon: self.config.perf.presentmon_path.clone(),
        });
        let session = cl_session::run(&prepared.plan, capture.as_ref())?;
        let mut notes = prepared.notes.clone();
        if let Some(n) = &session.capture_note {
            notes.push(n.clone());
        }

        let mut stats = None;
        let mut result_verdict = None;
        if let Some(file) = &session.capture_file {
            match parse_file(file)
                .map_err(|e| e.to_string())
                .and_then(|ft| analyse(&ft).map_err(|e| e.to_string()))
            {
                Ok(s) => {
                    let v = verdict(&s);
                    tuning.push_run(RunRecord {
                        at: now(),
                        step: prepared.step,
                        stats: s.clone(),
                        verdict: v,
                    });
                    stats = Some(s);
                    result_verdict = Some(v);
                }
                Err(e) => notes.push(format!("Measurement ignored: {e}")),
            }
        }
        self.profiles().save(&tuning)?;

        let ladder = prepared.ladder;
        Ok(PlayReport {
            session,
            stats,
            verdict: result_verdict,
            next_step: tuning.state.current,
            next_label: ladder
                .get(tuning.state.current)
                .cloned()
                .unwrap_or_default(),
            tuning_done: tuning.state.done,
            notes,
        })
    }

    /// Records a measurement made outside a session (e.g. `clctl analyse`).
    pub fn record_capture(
        &mut self,
        game: &Game,
        stats: FrameStats,
    ) -> Result<GameTuning, EngineError> {
        let emulator = self.emulator_id_for(game.system);
        let adapter =
            cl_adapters::by_id(&emulator).ok_or(EngineError::UnknownEmulator(emulator))?;
        let mut tuning = self.tuning(game, adapter.as_ref());
        let v = verdict(&stats);
        tuning.push_run(RunRecord {
            at: now(),
            step: tuning.state.current,
            stats,
            verdict: v,
        });
        self.profiles().save(&tuning)?;
        Ok(tuning)
    }

    /// The user picks a rung by hand: it sticks and the search stops.
    pub fn set_step(&mut self, game: &Game, step: usize) -> Result<GameTuning, EngineError> {
        let emulator = self.emulator_id_for(game.system);
        let adapter =
            cl_adapters::by_id(&emulator).ok_or(EngineError::UnknownEmulator(emulator))?;
        let mut tuning = self.tuning(game, adapter.as_ref());
        tuning.state.set_by_user(step);
        self.profiles().save(&tuning)?;
        Ok(tuning)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cl_hwprobe::{BenchResult, CpuInfo, GpuInfo};

    fn engine_with_fake_hw(dir: &std::path::Path) -> Engine {
        let paths = Paths::at(dir);
        let hw = HardwareReport {
            os: "test".into(),
            cpu: CpuInfo {
                brand: "Test CPU".into(),
                physical_cores: 8,
                logical_cores: 16,
                max_mhz: 4500,
                avx2: true,
            },
            ram_mb: 16_384,
            gpus: vec![GpuInfo {
                name: "Test GPU".into(),
                kind: GpuKind::Discrete,
                vendor_id: 0x1002,
                vendor: "AMD".into(),
                vram_mb: 8_192,
                vulkan: "1.3.0".into(),
            }],
            bench: Some(BenchResult {
                score: 1200,
                best_ms: 200.0,
            }),
        };
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join("hardware.json"),
            serde_json::to_string(&hw).unwrap(),
        )
        .unwrap();
        Engine::load(paths).unwrap()
    }

    #[test]
    fn prepares_a_launch_with_estimated_profile() {
        let dir = tempfile::tempdir().unwrap();
        let roms = dir.path().join("roms/gc");
        std::fs::create_dir_all(&roms).unwrap();
        std::fs::write(roms.join("Game.rvz"), b"").unwrap();
        let fake_dolphin = dir.path().join("dolphin-emu");
        std::fs::write(&fake_dolphin, b"").unwrap();

        let mut engine = engine_with_fake_hw(&dir.path().join("data"));
        engine.config.rom_dirs.push(dir.path().join("roms"));
        engine.config.perf.auto_tune = false;
        engine.config.emulators.insert(
            "dolphin".into(),
            EmulatorConfig {
                path: Some(fake_dolphin),
                ..Default::default()
            },
        );

        let games = engine.games();
        assert_eq!(games.len(), 1);
        let systems = engine.systems(&games);
        assert!(systems[0].emulator_found);

        let (p, _) = engine.prepare(&games[0]).unwrap();
        assert_eq!(p.emulator, "dolphin");
        // 8 GB discrete GPU → rung 3 of the estimate table.
        assert_eq!(p.step_label, "1440p");
        assert!(!p.measure);

        let t = engine.set_step(&games[0], 1).unwrap();
        assert!(t.state.done);
        let (p, _) = engine.prepare(&games[0]).unwrap();
        assert_eq!(p.step_label, "720p");
    }

    #[test]
    fn missing_emulator_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let mut engine = engine_with_fake_hw(dir.path());
        engine.config.emulators.insert(
            "pcsx2".into(),
            EmulatorConfig {
                path: Some(dir.path().join("nope")),
                ..Default::default()
            },
        );
        let g = Game {
            id: "g".into(),
            system: SystemId::Ps2,
            title: "x".into(),
            path: "x.iso".into(),
            cover: None,
            favorite: false,
        };
        assert!(matches!(
            engine.prepare(&g),
            Err(EngineError::EmulatorMissing { .. })
        ));
    }
}
