//! The engine used by the interface and by `clctl`.
//!
//! It keeps the interface free of emulator details: the interface asks for
//! systems and games, then calls [`Engine::play`]; the engine chooses the
//! emulator, the settings, measures when needed and adjusts the profile.
//!
//! Settings the user changes inside an emulator are noticed before each
//! launch (`cl_core::watch`) and left alone from then on, until the user
//! gives them back to the automatic optimisation
//! ([`Engine::reset_emulator_tuning`]).

use cl_adapters::profile::{self, Profile, ProfileAdapter};
use cl_adapters::{EmulatorAdapter, ExeIndex, LaunchContext, LaunchPlan, SystemTweaks};
use cl_core::config::{AppConfig, CustomEmulator, EmulatorConfig, EmulatorMode, ImportedGame};
use cl_core::paths::Paths;
use cl_core::watch::SettingsWatch;
use cl_core::{Game, SystemId, SystemInfo};
use cl_hwprobe::{GpuKind, HardwareReport};
use cl_perf::estimate::{estimate, GpuClass, HwSummary, LadderKind};
use cl_perf::frametime::{analyse, parse_file, verdict, FrameStats, Verdict};
use cl_perf::store::{GameTuning, ProfileStore, RunRecord};
use cl_perf::tuner::TuneState;
use cl_session::{CaptureRequest, SessionRecord, SessionResult};
use cl_updater::{sources, ManagedEmulator, Source};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Config(#[from] cl_core::config::ConfigError),
    #[error("unknown emulator '{0}'")]
    UnknownEmulator(String),
    #[error("{emulator} was not found. Choose its executable in Settings › Emulators.")]
    EmulatorMissing { id: String, emulator: String },
    #[error("no emulator can run {0}. Add one in Settings › Emulators.")]
    NoEmulator(String),
    #[error("{0} is not a file")]
    NotAFile(PathBuf),
    #[error("{path} is not a supported {system} game")]
    UnsupportedGameFile { path: PathBuf, system: SystemId },
    #[error("invalid custom emulator: {0}")]
    InvalidCustomEmulator(String),
    #[error(transparent)]
    Adapter(#[from] cl_adapters::AdapterError),
    #[error(transparent)]
    Session(#[from] cl_session::SessionError),
    #[error(transparent)]
    Update(#[from] cl_updater::UpdateError),
    #[error("automatic installation is not available for {0}")]
    InstallUnavailable(String),
    #[error("metadata: {0}")]
    Metadata(String),
    #[error("file error: {0}")]
    Io(#[from] std::io::Error),
}

impl EngineError {
    /// Message in the interface language (`fr` or English).
    pub fn localized(&self, lang: &str) -> String {
        if !lang.starts_with("fr") {
            return self.to_string();
        }
        match self {
            EngineError::EmulatorMissing { emulator, .. } => format!(
                "{emulator} est introuvable. Choisis son exécutable dans Réglages › Émulateurs."
            ),
            EngineError::NoEmulator(system) => format!(
                "Aucun émulateur ne lance {system}. Ajoutes-en un dans Réglages › Émulateurs."
            ),
            EngineError::UnknownEmulator(id) => format!("Émulateur inconnu : {id}"),
            EngineError::NotAFile(p) => format!("{} n'est pas un fichier", p.display()),
            EngineError::UnsupportedGameFile { path, system } => format!(
                "{} n'est pas un fichier de jeu compatible avec {}",
                path.display(),
                system.info().name
            ),
            EngineError::InvalidCustomEmulator(message) => {
                format!("Configuration d'émulateur personnalisée invalide : {message}")
            }
            EngineError::InstallUnavailable(name) => {
                format!("L'installation automatique n'est pas disponible pour {name}")
            }
            other => other.to_string(),
        }
    }

    /// Short machine-readable code, for interfaces that translate messages.
    pub fn code(&self) -> &'static str {
        match self {
            EngineError::EmulatorMissing { .. } => "emulator_missing",
            EngineError::NoEmulator(_) => "no_emulator",
            EngineError::UnknownEmulator(_) => "unknown_emulator",
            _ => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemEntry {
    pub info: SystemInfo,
    pub games: usize,
    pub emulator: String,
    pub emulator_name: String,
    pub emulator_found: bool,
}

/// What will happen when a game is started.
#[derive(Debug, Clone, Serialize)]
pub struct PreparedLaunch {
    pub game: Game,
    pub emulator: String,
    pub emulator_name: String,
    pub step: usize,
    pub step_label: String,
    pub ladder: Vec<String>,
    pub measure: bool,
    /// False when only the system-level optimisations apply.
    pub tunes_settings: bool,
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

/// One emulator as shown in the settings.
#[derive(Debug, Clone, Serialize)]
pub struct EmulatorStatus {
    pub id: String,
    pub name: String,
    /// Added by the user (no profile).
    pub custom: bool,
    /// Described by a profile file in the data folder.
    pub user_profile: bool,
    pub systems: Vec<SystemId>,
    pub license: String,
    pub homepage: String,
    pub mode: EmulatorMode,
    /// Path chosen by the user, if any.
    pub configured_path: Option<PathBuf>,
    /// Executable that will be used (chosen or detected).
    pub path: Option<PathBuf>,
    pub found: bool,
    /// Whether the launcher tunes the emulator's own settings.
    pub tunes_settings: bool,
    /// Settings the user changed in the emulator (left alone) or locked.
    pub user_settings: Vec<String>,
    /// Active launcher-managed version, when present.
    pub managed_version: Option<String>,
    /// Curated version offered without a network lookup (Dolphin on Windows).
    pub install_version: Option<String>,
    pub automatic_install: bool,
    /// Official download page when installation must be completed manually.
    pub download_page: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallReport {
    pub emulator: String,
    pub version: String,
    pub executable: PathBuf,
    pub already_installed: bool,
}

/// Emulator choice for one system, as shown in the settings.
#[derive(Debug, Clone, Serialize)]
pub struct SystemChoice {
    pub info: SystemInfo,
    pub games: usize,
    /// Emulator used now.
    pub emulator: String,
    /// True when the launcher picks the emulator (no choice saved).
    pub automatic: bool,
    pub options: Vec<EmulatorOption>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmulatorOption {
    pub id: String,
    pub name: String,
    pub found: bool,
}

/// What a folder given by the user turned out to be.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FolderKind {
    Games,
    /// A folder of emulators (RetroBat `emulators`, …).
    Emulators,
    /// The executable of a known emulator.
    Emulator {
        id: String,
    },
    AlreadyAdded,
}

/// Adapter, its settings and the executable found (if any).
pub type FoundEmulator = (Box<dyn EmulatorAdapter>, EmulatorConfig, Option<PathBuf>);

pub struct Engine {
    pub paths: Paths,
    pub config: AppConfig,
    hardware: Option<HardwareReport>,
    user_profiles: Vec<Profile>,
    /// Profile files that could not be read, for the settings screen.
    pub profile_errors: Vec<String>,
    index: RefCell<Option<ExeIndex>>,
    library_cache: RefCell<Option<Vec<Game>>>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn session_outcome(exit_code: Option<i32>) -> (&'static str, Option<String>) {
    match exit_code {
        Some(0) => ("completed", None),
        Some(code) => ("failed", Some(format!("emulator exited with code {code}"))),
        None => (
            "failed",
            Some("emulator did not report a normal exit code".into()),
        ),
    }
}

fn session_can_update_tuning(exit_code: Option<i32>) -> bool {
    exit_code == Some(0)
}

/// Picks the French or English text according to the interface language.
fn tr(lang: &str, en: String, fr: String) -> String {
    if lang.starts_with("fr") {
        fr
    } else {
        en
    }
}

fn same_name(p: &Path, names: &[String]) -> bool {
    p.file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .is_some_and(|n| names.iter().any(|x| x.to_lowercase() == n))
}

fn validate_custom_emulator(
    path: &Path,
    systems: &[SystemId],
    args: &[String],
) -> Result<(), EngineError> {
    if !path.is_file() {
        return Err(EngineError::NotAFile(path.to_path_buf()));
    }
    if systems.is_empty() {
        return Err(EngineError::InvalidCustomEmulator(
            "at least one console must be selected".into(),
        ));
    }
    if !args.is_empty() && !args.iter().any(|arg| arg.contains("{rom}")) {
        return Err(EngineError::InvalidCustomEmulator(
            "arguments must contain {rom}, or be empty".into(),
        ));
    }
    Ok(())
}

impl Engine {
    pub fn load(paths: Paths) -> Result<Engine, EngineError> {
        let config = AppConfig::load(&paths.config_file())?;
        let hardware = std::fs::read_to_string(paths.root.join("hardware.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok());
        let mut engine = Engine {
            paths,
            config,
            hardware,
            user_profiles: Vec::new(),
            profile_errors: Vec::new(),
            index: RefCell::new(None),
            library_cache: RefCell::new(None),
        };
        engine.reload_profiles();
        if engine.migrate_config() {
            log::info!("settings: misplaced emulator paths moved");
            if let Err(e) = engine.save_config() {
                log::warn!("cannot save the migrated settings: {e}");
            }
        }
        Ok(engine)
    }

    pub fn save_config(&self) -> Result<(), EngineError> {
        self.config.save(&self.paths.config_file())?;
        self.index.replace(None);
        Ok(())
    }

    fn invalidate_library(&self) {
        self.library_cache.replace(None);
    }

    /// Reads the profile files of the data folder again.
    pub fn reload_profiles(&mut self) {
        let (ok, errors) = profile::load_dir(&self.paths.root.join("emulator-profiles"));
        self.user_profiles = ok;
        self.profile_errors = errors.into_iter().map(|e| e.to_string()).collect();
        self.index.replace(None);
    }

    /// Every emulator known: custom ones, profile files (which may replace
    /// a bundled profile of the same id), then built-in adapters.
    pub fn adapters(&self) -> Vec<Box<dyn EmulatorAdapter>> {
        let mut out: Vec<Box<dyn EmulatorAdapter>> = Vec::new();
        for (id, c) in &self.config.custom_emulators {
            out.push(Box::new(ProfileAdapter::new(Profile::custom(id, c))));
        }
        for p in &self.user_profiles {
            out.push(Box::new(ProfileAdapter::new(p.clone())));
        }
        let mut seen: BTreeSet<String> = out.iter().map(|a| a.id().to_owned()).collect();
        for a in cl_adapters::registry() {
            if seen.insert(a.id().to_owned()) {
                out.push(a);
            }
        }
        out
    }

    pub fn adapter(&self, id: &str) -> Option<Box<dyn EmulatorAdapter>> {
        self.adapters().into_iter().find(|a| a.id() == id)
    }

    /// Folders searched for emulator executables: the folders chosen by
    /// the user, the `emulators` folder next to a `roms` folder (RetroBat
    /// layout), and on Windows `X:\RetroBat\emulators` on every drive.
    pub fn emulator_search_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = self.config.emulator_dirs.clone();
        // Reuse emulators installed by the first Windows prototype. This is
        // also a sensible conventional location for portable installations.
        if cfg!(windows) {
            if let Some(dir) =
                cl_core::paths::expand("{localappdata}/ConsoleLauncher/emulators", None)
            {
                dirs.push(dir);
            }
        }
        for r in &self.config.rom_dirs {
            if r.file_name()
                .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case("roms"))
            {
                if let Some(parent) = r.parent() {
                    dirs.push(parent.join("emulators"));
                }
            }
            dirs.push(r.join("emulators"));
        }
        if cfg!(windows) {
            for letter in b'C'..=b'Z' {
                dirs.push(PathBuf::from(format!(
                    "{}:\\RetroBat\\emulators",
                    letter as char
                )));
            }
        }
        let mut seen = BTreeSet::new();
        dirs.into_iter()
            .filter(|d| d.is_dir() && seen.insert(d.clone()))
            .collect()
    }

    fn with_index<T>(&self, f: impl FnOnce(&ExeIndex) -> T) -> T {
        if self.index.borrow().is_none() {
            let names: Vec<String> = self.adapters().iter().flat_map(|a| a.exe_names()).collect();
            let idx = ExeIndex::build(&self.emulator_search_dirs(), &names);
            self.index.replace(Some(idx));
        }
        f(self.index.borrow().as_ref().expect("built above"))
    }

    /// Moves paths the user put in the wrong list: an emulator executable
    /// or a folder of emulators added as a game folder. Returns true when
    /// the settings changed.
    pub fn migrate_config(&mut self) -> bool {
        let dirs = std::mem::take(&mut self.config.rom_dirs);
        let mut changed = false;
        for d in dirs {
            match self.classify(&d) {
                Some(FolderKind::Emulator { id }) => {
                    self.set_path_in_config(&id, Some(d));
                    changed = true;
                }
                Some(FolderKind::Emulators) => {
                    // A folder already inside an emulator folder adds nothing.
                    if !self.config.emulator_dirs.iter().any(|e| d.starts_with(e)) {
                        self.config.emulator_dirs.push(d);
                    }
                    changed = true;
                }
                _ => self.config.rom_dirs.push(d),
            }
        }
        if changed {
            self.index.replace(None);
            self.invalidate_library();
        }
        changed
    }

    /// Tells what a path is: an emulator executable, a folder of
    /// emulators, or a game folder (`None` when it does not exist).
    fn classify(&self, p: &Path) -> Option<FolderKind> {
        let adapters = self.adapters();
        if p.is_file() {
            return adapters
                .iter()
                .filter(|a| !self.config.custom_emulators.contains_key(a.id()))
                .find(|a| same_name(p, &a.exe_names()))
                .map(|a| FolderKind::Emulator { id: a.id().into() });
        }
        if !p.is_dir() {
            return None;
        }
        let name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if name == "emulators" {
            return Some(FolderKind::Emulators);
        }
        let has_games = !cl_library::scan(&[p.to_path_buf()]).is_empty();
        if has_games {
            return Some(FolderKind::Games);
        }
        let names: Vec<String> = adapters.iter().flat_map(|a| a.exe_names()).collect();
        if cl_adapters::find_in_tree(p, &names).is_some() {
            return Some(FolderKind::Emulators);
        }
        Some(FolderKind::Games)
    }

    /// Adds a folder or file chosen by the user where it belongs.
    pub fn add_path(&mut self, p: PathBuf) -> Result<FolderKind, EngineError> {
        let kind = self
            .classify(&p)
            .ok_or_else(|| EngineError::NotAFile(p.clone()))?;
        match &kind {
            FolderKind::Games => {
                if self.config.rom_dirs.contains(&p) {
                    return Ok(FolderKind::AlreadyAdded);
                }
                self.config.rom_dirs.push(p);
                self.invalidate_library();
            }
            FolderKind::Emulators => {
                if self.config.emulator_dirs.iter().any(|e| p.starts_with(e)) {
                    return Ok(FolderKind::AlreadyAdded);
                }
                self.config.emulator_dirs.push(p);
            }
            FolderKind::Emulator { id } => self.set_path_in_config(id, Some(p)),
            FolderKind::AlreadyAdded => {}
        }
        self.save_config()?;
        Ok(kind)
    }

    pub fn remove_path(&mut self, p: &Path) -> Result<(), EngineError> {
        self.config.rom_dirs.retain(|d| d != p);
        self.config.emulator_dirs.retain(|d| d != p);
        self.invalidate_library();
        self.save_config()
    }

    /// Adds one game file without imposing a RetroBat-style folder layout.
    /// The console is explicit because common formats such as ISO and RVZ are
    /// shared by several systems.
    pub fn add_game(&mut self, path: PathBuf, system: SystemId) -> Result<(), EngineError> {
        if !path.is_file() {
            return Err(EngineError::NotAFile(path));
        }
        if cl_library::imported_game(system, &path).is_none() {
            return Err(EngineError::UnsupportedGameFile { path, system });
        }
        if let Some(existing) = self
            .config
            .imported_games
            .iter_mut()
            .find(|g| g.path == path)
        {
            existing.system = system;
        } else {
            self.config
                .imported_games
                .push(ImportedGame { path, system });
        }
        self.invalidate_library();
        self.save_config()
    }

    pub fn remove_game(&mut self, path: &Path) -> Result<(), EngineError> {
        self.config.imported_games.retain(|g| g.path != path);
        self.invalidate_library();
        self.save_config()
    }

    fn set_path_in_config(&mut self, id: &str, path: Option<PathBuf>) {
        if let Some(c) = self.config.custom_emulators.get_mut(id) {
            if let Some(p) = path {
                c.path = p;
            }
            return;
        }
        self.config.emulators.entry(id.to_owned()).or_default().path = path;
    }

    /// Sets (or clears, to detect it again) the executable of an emulator.
    pub fn set_emulator_path(
        &mut self,
        id: &str,
        path: Option<PathBuf>,
    ) -> Result<(), EngineError> {
        if self.adapter(id).is_none() {
            return Err(EngineError::UnknownEmulator(id.into()));
        }
        if path.is_none() && self.config.custom_emulators.contains_key(id) {
            return Err(EngineError::InvalidCustomEmulator(
                "a custom emulator has no automatic detection path; remove it or choose a new executable"
                    .into(),
            ));
        }
        if let Some(p) = &path {
            if !p.is_file() {
                return Err(EngineError::NotAFile(p.clone()));
            }
        }
        self.set_path_in_config(id, path);
        self.save_config()
    }

    /// Adds an emulator chosen by the user. When the executable belongs to
    /// an emulator the launcher knows, that emulator is used instead (with
    /// its automatic optimisation). Returns the emulator id.
    pub fn add_custom_emulator(
        &mut self,
        path: PathBuf,
        name: String,
        systems: Vec<SystemId>,
        args: Vec<String>,
    ) -> Result<String, EngineError> {
        if !path.is_file() {
            return Err(EngineError::NotAFile(path));
        }
        if !args.is_empty() && !args.iter().any(|arg| arg.contains("{rom}")) {
            return Err(EngineError::InvalidCustomEmulator(
                "arguments must contain {rom}, or be empty".into(),
            ));
        }
        if let Some(FolderKind::Emulator { id }) = self.classify(&path) {
            self.set_path_in_config(&id, Some(path));
            self.save_config()?;
            return Ok(id);
        }
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "emulator".into());
        let slug: String = stem
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        let mut id = format!("custom-{slug}");
        let mut n = 2;
        while self.config.custom_emulators.contains_key(&id) {
            id = format!("custom-{slug}-{n}");
            n += 1;
        }
        self.config.custom_emulators.insert(
            id.clone(),
            CustomEmulator {
                name,
                path,
                args,
                systems,
            },
        );
        self.save_config()?;
        Ok(id)
    }

    pub fn update_custom_emulator(
        &mut self,
        id: &str,
        c: CustomEmulator,
    ) -> Result<(), EngineError> {
        if !self.config.custom_emulators.contains_key(id) {
            return Err(EngineError::UnknownEmulator(id.into()));
        }
        validate_custom_emulator(&c.path, &c.systems, &c.args)?;
        self.config.custom_emulators.insert(id.into(), c);
        self.save_config()
    }

    pub fn remove_custom_emulator(&mut self, id: &str) -> Result<(), EngineError> {
        self.config.custom_emulators.remove(id);
        self.config.system_emulator.retain(|_, e| e != id);
        self.save_config()
    }

    /// Chooses the emulator of a system; `None` lets the launcher pick.
    pub fn set_system_emulator(
        &mut self,
        system: SystemId,
        id: Option<String>,
    ) -> Result<(), EngineError> {
        match id {
            Some(id) => {
                let ok = self
                    .adapter(&id)
                    .is_some_and(|a| a.systems().contains(&system));
                if !ok {
                    return Err(EngineError::UnknownEmulator(id));
                }
                self.config.system_emulator.insert(system, id);
            }
            None => {
                self.config.system_emulator.remove(&system);
            }
        }
        self.save_config()
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

    /// System-level optimisations for this machine.
    fn tweaks(&mut self) -> SystemTweaks {
        if !self.config.perf.system_tweaks {
            return SystemTweaks::default();
        }
        let hw = self.hardware(false);
        let has = |k: GpuKind| hw.gpus.iter().any(|g| g.kind == k);
        SystemTweaks {
            high_priority: true,
            // Only useful when Windows could pick the integrated GPU.
            prefer_discrete_gpu: has(GpuKind::Discrete) && has(GpuKind::Integrated),
        }
    }

    pub fn games(&self) -> Vec<Game> {
        if let Some(games) = self.library_cache.borrow().as_ref() {
            return games.clone();
        }
        let identity_file = self.paths.identity_cache_file();
        let mut identities = cl_library::IdentityCache::load(&identity_file);
        let mut games = cl_library::scan_cached(&self.config.rom_dirs, &mut identities);
        for imported in &self.config.imported_games {
            if games.iter().any(|g| g.path == imported.path) {
                continue;
            }
            if let Some(game) =
                cl_library::imported_game_cached(imported.system, &imported.path, &mut identities)
            {
                games.push(game);
            }
        }
        if let Err(error) = identities.save(&identity_file) {
            log::warn!("cannot save game identity cache: {error}");
        }
        // Enrichment is cache-first and optional: launching remains fully
        // functional offline and provider outages never affect a scan.
        cl_library::MetadataCache::load(&self.paths.metadata_cache_file()).apply(&mut games);
        games.sort_by(|a, b| {
            (a.system, a.title.to_lowercase()).cmp(&(b.system, b.title.to_lowercase()))
        });
        cl_library::Favorites::load(&self.paths.root.join("favorites.json")).apply(&mut games);
        self.library_cache.replace(Some(games.clone()));
        games
    }

    /// Invalidates and rebuilds the library for an explicit user refresh.
    /// Ordinary consumers keep using the in-memory snapshot.
    pub fn refresh_games(&self) -> Vec<Game> {
        self.invalidate_library();
        self.games()
    }

    /// Explicit network enrichment for one game. Scanning and launching stay
    /// cache-only and therefore continue to work offline.
    pub fn refresh_game_metadata(
        &self,
        id: &str,
    ) -> Result<Option<cl_core::GameMetadata>, EngineError> {
        let game = self
            .games()
            .into_iter()
            .find(|game| game.id == id)
            .ok_or_else(|| EngineError::Metadata(format!("unknown game {id}")))?;
        let Some(mut metadata) =
            cl_library::provider::lookup_title_screen(&game).map_err(EngineError::Metadata)?
        else {
            return Ok(None);
        };
        if let Some(url) = metadata.cover_url.as_deref() {
            let file_name = game
                .identity
                .serial
                .as_deref()
                .unwrap_or(&game.id)
                .chars()
                .filter(|character| character.is_ascii_alphanumeric() || *character == '-')
                .collect::<String>();
            let destination = self
                .paths
                .artwork_cache_dir()
                .join(game.system.as_str())
                .join(format!(
                    "{file_name}.{}",
                    cl_library::provider::artwork_extension(url).unwrap_or("png")
                ));
            match cl_library::provider::cache_cover(url, &destination) {
                Ok(path) => metadata.cover_path = Some(path),
                Err(error) => log::warn!("cannot cache artwork for {}: {error}", game.id),
            }
        }
        let file = self.paths.metadata_cache_file();
        let mut cache = cl_library::MetadataCache::load(&file);
        cache
            .records
            .insert(cl_library::MetadataCache::key(&game), metadata.clone());
        cache.save(&file)?;
        self.invalidate_library();
        Ok(Some(metadata))
    }

    pub fn toggle_favorite(&self, game_id: &str) -> std::io::Result<bool> {
        let file = self.paths.root.join("favorites.json");
        let mut f = cl_library::Favorites::load(&file);
        let on = f.toggle(game_id);
        f.save(&file)?;
        self.invalidate_library();
        Ok(on)
    }

    /// Emulators able to run a system, preferred first.
    fn candidates(&self, system: SystemId) -> Vec<Box<dyn EmulatorAdapter>> {
        let mut all: Vec<Box<dyn EmulatorAdapter>> = self
            .adapters()
            .into_iter()
            .filter(|a| a.systems().contains(&system))
            .collect();
        let pref = cl_adapters::preferred_emulators(system);
        all.sort_by_key(|a| pref.iter().position(|p| *p == a.id()).unwrap_or(pref.len()));
        all
    }

    /// Emulator used for a system: the user's choice, otherwise the first
    /// installed emulator in the order of preference.
    pub fn emulator_id_for(&self, system: SystemId) -> String {
        if let Some(id) = self.config.system_emulator.get(&system) {
            if self.adapter(id).is_some() {
                return id.clone();
            }
        }
        let candidates = self.candidates(system);
        candidates
            .iter()
            .find(|a| self.locate(a.as_ref()).1.is_some())
            .or(candidates.first())
            .map(|a| a.id().to_owned())
            .unwrap_or_else(|| cl_adapters::default_emulator(system).to_owned())
    }

    fn managed(&self, id: &str) -> ManagedEmulator {
        ManagedEmulator::new(&self.paths.emulators_dir(), id)
    }

    fn locate(&self, adapter: &dyn EmulatorAdapter) -> (EmulatorConfig, Option<PathBuf>) {
        let id = adapter.id();
        if let Some(c) = self.config.custom_emulators.get(id) {
            let cfg = EmulatorConfig {
                path: Some(c.path.clone()),
                ..self.config.emulator(id)
            };
            return (cfg, c.path.is_file().then(|| c.path.clone()));
        }
        let cfg = self.config.emulator(id);
        let managed = (cfg.mode == EmulatorMode::Managed)
            .then(|| self.managed(id).active_dir())
            .flatten();
        let exe = self.with_index(|idx| adapter.find_executable(&cfg, managed.as_deref(), idx));
        (cfg, exe)
    }

    pub fn find_emulator(&self, id: &str) -> Result<FoundEmulator, EngineError> {
        let adapter = self
            .adapter(id)
            .ok_or_else(|| EngineError::UnknownEmulator(id.into()))?;
        let (cfg, exe) = self.locate(adapter.as_ref());
        Ok((adapter, cfg, exe))
    }

    pub fn systems(&self, games: &[Game]) -> Vec<SystemEntry> {
        let grouped = cl_library::group_by_system(games);
        grouped
            .into_iter()
            .map(|(system, list)| {
                let emulator = self.emulator_id_for(system);
                let (name, found) = match self.find_emulator(&emulator) {
                    Ok((a, _, exe)) => (a.name().to_owned(), exe.is_some()),
                    Err(_) => (emulator.clone(), false),
                };
                SystemEntry {
                    info: system.info(),
                    games: list.len(),
                    emulator,
                    emulator_name: name,
                    emulator_found: found,
                }
            })
            .collect()
    }

    fn watch_file(&self, id: &str) -> PathBuf {
        self.paths.generated_dir().join(format!("{id}-watch.json"))
    }

    /// Every emulator with its state, for the settings screen.
    pub fn emulators_status(&self) -> Vec<EmulatorStatus> {
        let user_ids: BTreeSet<&str> = self.user_profiles.iter().map(|p| p.id.as_str()).collect();
        self.adapters()
            .iter()
            .map(|a| {
                let id = a.id().to_owned();
                let (cfg, exe) = self.locate(a.as_ref());
                let custom = self.config.custom_emulators.contains_key(&id);
                let managed = self.managed(&id).state();
                let source = sources::source_for(&id);
                let install_version = match &source {
                    Some(Source::Pinned { version, .. }) => Some((*version).to_owned()),
                    _ => None,
                };
                let automatic_install = source
                    .as_ref()
                    .is_some_and(|s| !matches!(s, Source::Manual { .. }));
                let download_page = match &source {
                    Some(Source::Manual { page }) => Some((*page).to_owned()),
                    _ => None,
                };
                let mut user_settings: BTreeSet<String> =
                    SettingsWatch::load(&self.watch_file(&id)).user_changed;
                user_settings.extend(cfg.locked_keys.iter().cloned());
                EmulatorStatus {
                    name: a.name().to_owned(),
                    custom,
                    user_profile: user_ids.contains(id.as_str()),
                    systems: a.systems().to_vec(),
                    license: a.license().to_owned(),
                    homepage: a.homepage().to_owned(),
                    mode: cfg.mode,
                    configured_path: if custom {
                        Some(self.config.custom_emulators[&id].path.clone())
                    } else {
                        cfg.path.clone()
                    },
                    found: exe.is_some(),
                    path: exe,
                    tunes_settings: a.tunes_settings(),
                    user_settings: user_settings.into_iter().collect(),
                    managed_version: managed.active,
                    install_version,
                    automatic_install,
                    download_page,
                    id,
                }
            })
            .collect()
    }

    /// Emulator choice per system (systems with games first).
    pub fn system_choices(&self, games: &[Game]) -> Vec<SystemChoice> {
        let grouped = cl_library::group_by_system(games);
        let mut out: Vec<SystemChoice> = SystemId::ALL
            .into_iter()
            .map(|system| {
                let options = self
                    .candidates(system)
                    .iter()
                    .map(|a| EmulatorOption {
                        id: a.id().to_owned(),
                        name: a.name().to_owned(),
                        found: self.locate(a.as_ref()).1.is_some(),
                    })
                    .collect();
                SystemChoice {
                    info: system.info(),
                    games: grouped.get(&system).map_or(0, Vec::len),
                    emulator: self.emulator_id_for(system),
                    automatic: !self.config.system_emulator.contains_key(&system),
                    options,
                }
            })
            .collect();
        out.sort_by_key(|c| (c.games == 0, c.info.id));
        out
    }

    /// Gives every setting of an emulator back to the automatic
    /// optimisation: forgets the settings the user changed in it and the
    /// keys locked in the settings.
    pub fn reset_emulator_tuning(&mut self, id: &str) -> Result<(), EngineError> {
        let mut w = SettingsWatch::load(&self.watch_file(id));
        w.reset();
        w.save(&self.watch_file(id))?;
        cl_adapters::forget_ledger(id, &self.paths.generated_dir());
        if let Some(cfg) = self.config.emulators.get_mut(id) {
            cfg.locked_keys.clear();
        }
        self.save_config()
    }

    /// Opens an emulator without a game, so the user can set it up.
    pub fn open_emulator(&self, id: &str) -> Result<(), EngineError> {
        let (adapter, _, exe) = self.find_emulator(id)?;
        let exe = exe.ok_or_else(|| EngineError::EmulatorMissing {
            id: id.into(),
            emulator: adapter.name().into(),
        })?;
        let mut cmd = std::process::Command::new(&exe);
        cmd.args(adapter.setup_args());
        if let Some(dir) = exe.parent() {
            cmd.current_dir(dir);
        }
        cmd.spawn().map_err(|source| {
            EngineError::Session(cl_session::SessionError::Spawn {
                program: exe.display().to_string(),
                source,
            })
        })?;
        Ok(())
    }

    /// Downloads, verifies and activates an emulator from its official source.
    /// Curated releases include a fixed archive hash; existing verified files
    /// are reused without downloading again.
    pub fn install_emulator(&mut self, id: &str) -> Result<InstallReport, EngineError> {
        let adapter = self
            .adapter(id)
            .ok_or_else(|| EngineError::UnknownEmulator(id.into()))?;
        let source = sources::source_for(id)
            .ok_or_else(|| EngineError::InstallUnavailable(adapter.name().into()))?;
        if matches!(source, Source::Manual { .. }) {
            return Err(EngineError::InstallUnavailable(adapter.name().into()));
        }
        let release = sources::latest(&source)?
            .ok_or_else(|| EngineError::InstallUnavailable(adapter.name().into()))?;
        let managed = self.managed(id);
        let names = adapter.exe_names();
        let existing = managed
            .active_dir()
            .and_then(|dir| cl_adapters::find_in_tree(&dir, &names));
        let already_installed =
            managed.state().active.as_deref() == Some(&release.version) && existing.is_some();
        let executable = if let Some(exe) = existing.filter(|_| already_installed) {
            exe
        } else {
            let archive = sources::download(&release, &managed.root.join("downloads"))?;
            let exe_name = names.first().cloned().unwrap_or_default();
            let install = managed.install_archive(&release.version, &archive, &exe_name);
            let _ = std::fs::remove_file(&archive);
            install?;
            cl_adapters::find_in_tree(
                &managed
                    .active_dir()
                    .ok_or_else(|| EngineError::InstallUnavailable(adapter.name().into()))?,
                &names,
            )
            .ok_or_else(|| EngineError::InstallUnavailable(adapter.name().into()))?
        };
        let cfg = self.config.emulators.entry(id.to_owned()).or_default();
        cfg.mode = EmulatorMode::Managed;
        cfg.path = None;
        self.save_config()?;
        Ok(InstallReport {
            emulator: adapter.name().into(),
            version: release.version,
            executable,
            already_installed,
        })
    }

    fn profiles(&self) -> ProfileStore {
        ProfileStore::new(self.paths.profiles_dir())
    }

    /// Loads the game's tuning, or creates it from the estimate. A hardware
    /// change restarts the search from a fresh estimate.
    pub fn tuning(&mut self, game: &Game, adapter: &dyn EmulatorAdapter) -> GameTuning {
        let ladder_len = adapter.quality_ladder(game.system).len();
        let mut fingerprint = self.hw_fingerprint();
        let tuning_revision = adapter.tuning_revision();
        if tuning_revision > 0 {
            fingerprint.push_str(&format!("|{}-policy-{tuning_revision}", adapter.id()));
        }
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
        let gpu_vendor_id = self
            .hardware(false)
            .main_gpu()
            .map(|gpu| gpu.vendor_id)
            .unwrap_or(0);
        let apis = adapter.apis_for_gpu(gpu_vendor_id);
        let est = estimate(&self.hw_summary(), game.system, kind, ladder_len, &apis);
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

    fn adapter_for_game(&self, game: &Game) -> Result<FoundEmulator, EngineError> {
        let emulator = self.emulator_id_for(game.system);
        if self.adapter(&emulator).is_none() {
            return Err(EngineError::NoEmulator(game.system.info().name.into()));
        }
        self.find_emulator(&emulator)
    }

    /// What a launch would do, without touching any file (for the interface).
    pub fn preview(&mut self, game: &Game) -> Result<(PreparedLaunch, GameTuning), EngineError> {
        let (adapter, _cfg, exe) = self.adapter_for_game(game)?;
        let exe = exe.ok_or_else(|| EngineError::EmulatorMissing {
            id: adapter.id().into(),
            emulator: adapter.name().into(),
        })?;
        let tuning = self.tuning(game, adapter.as_ref());
        let ladder = adapter.quality_ladder(game.system);
        let step = &ladder[tuning.state.current.min(ladder.len() - 1)];
        let (measure, mut notes) = self.measure_decision(&tuning);
        let lang = self.config.language.clone();
        let name = adapter.name().to_owned();
        if !adapter.tunes_settings() {
            notes.push(tr(
                &lang,
                format!("{name}: its own settings are not known to the launcher; system-level optimisations only."),
                format!("{name} : ses réglages ne sont pas connus du lanceur, seules les optimisations système s'appliquent."),
            ));
        }
        let user = SettingsWatch::load(&self.watch_file(adapter.id())).user_changed;
        if !user.is_empty() {
            let list = user.into_iter().collect::<Vec<_>>().join(", ");
            notes.push(tr(
                &lang,
                format!("Settings you changed in {name} are kept: {list}."),
                format!("Réglages modifiés par toi dans {name}, conservés : {list}."),
            ));
        }
        Ok((
            PreparedLaunch {
                game: game.clone(),
                emulator: adapter.id().into(),
                emulator_name: adapter.name().into(),
                step: tuning.state.current,
                step_label: step.label.clone(),
                ladder: ladder.iter().map(|s| s.label.clone()).collect(),
                measure,
                tunes_settings: adapter.tunes_settings(),
                plan: LaunchPlan {
                    program: exe,
                    ..Default::default()
                },
                notes,
            },
            tuning,
        ))
    }

    fn measure_decision(&self, tuning: &GameTuning) -> (bool, Vec<String>) {
        let mut notes = Vec::new();
        let mut measure = self.config.perf.auto_tune && !tuning.state.done;
        if measure {
            if let Err(e) =
                cl_session::capture_tool_available(self.config.perf.presentmon_path.as_deref())
            {
                let fr = if e.contains("permission") {
                    "Mesure de fluidité pas encore autorisée par Windows (Réglages › Mesure de fluidité) : réglages choisis d'après l'estimation de la machine.".to_string()
                } else {
                    "Mesure de fluidité indisponible (outil de mesure introuvable) : réglages choisis d'après l'estimation de la machine.".to_string()
                };
                notes.push(tr(
                    &self.config.language,
                    format!("Smoothness check unavailable ({e}): settings come from the hardware estimate."),
                    fr,
                ));
                measure = false;
            }
        }
        (measure, notes)
    }

    /// Prepares a launch: notices settings the user changed in the
    /// emulator, writes the optimised settings, builds the command.
    pub fn prepare(&mut self, game: &Game) -> Result<(PreparedLaunch, GameTuning), EngineError> {
        let (adapter, cfg, exe) = self.adapter_for_game(game)?;
        let exe = exe.ok_or_else(|| EngineError::EmulatorMissing {
            id: adapter.id().into(),
            emulator: adapter.name().into(),
        })?;
        let tuning = self.tuning(game, adapter.as_ref());
        let ladder = adapter.quality_ladder(game.system);
        let step = &ladder[tuning.state.current.min(ladder.len() - 1)];
        let (measure, mut notes) = self.measure_decision(&tuning);
        let generated = self.paths.generated_dir();

        // Settings changed by hand in the emulator since the last launch.
        let keys = adapter.managed_keys(game.system);
        let watch_file = self.watch_file(adapter.id());
        let mut watch = SettingsWatch::load(&watch_file);
        if let Some(cur) = adapter.current_values(&exe, cfg.mode, &generated) {
            let found = watch.detect(&cur, &keys);
            if !found.is_empty() {
                let (list, name) = (found.join(", "), adapter.name());
                notes.push(tr(
                    &self.config.language,
                    format!("You changed {list} in {name}: the launcher leaves it alone (Settings › Emulators to undo)."),
                    format!("Tu as modifié {list} dans {name} : le lanceur n'y touche plus (Réglages › Émulateurs pour annuler)."),
                ));
            }
        }
        let mut locked: Vec<String> = cfg.locked_keys.clone();
        locked.extend(watch.user_changed.iter().cloned());

        let mut plan = adapter.prepare(&LaunchContext {
            exe: &exe,
            game,
            step,
            api: tuning.api,
            mode: cfg.mode,
            generated_dir: &generated,
            locked_keys: &locked,
            bios_dir: self.config.bios_dir.as_deref(),
            controller_overrides: self.config.emulator_mappings.get(&format!(
                "{}:{}",
                adapter.id(),
                game.system
            )),
        })?;
        // The INI ledger may also have noticed a value changed by hand.
        for k in &plan.settings_report.kept_user_value {
            if !cfg.locked_keys.contains(k) {
                watch.user_changed.insert(k.clone());
            }
        }
        if let Some(after) = adapter.current_values(&exe, cfg.mode, &generated) {
            watch.record(&after, &keys);
        }
        if let Err(e) = watch.save(&watch_file) {
            log::warn!("cannot save {}: {e}", watch_file.display());
        }

        plan.tweaks = self.tweaks();
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
                emulator_name: adapter.name().into(),
                step: tuning.state.current,
                step_label: step.label.clone(),
                ladder: ladder.iter().map(|s| s.label.clone()).collect(),
                measure,
                tunes_settings: adapter.tunes_settings(),
                plan,
                notes,
            },
            tuning,
        ))
    }

    /// Plays a game (blocking until the emulator closes) and updates its profile.
    pub fn play(&mut self, game: &Game) -> Result<PlayReport, EngineError> {
        let (prepared, mut tuning) = self.prepare(game)?;
        let session_id = format!(
            "{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            std::process::id()
        );
        let started_at = now();
        let mut journal = SessionRecord {
            session_id,
            status: "started".into(),
            started_at,
            game_id: game.id.clone(),
            game_title: game.title.clone(),
            system: game.system.to_string(),
            emulator: format!("{} ({})", prepared.emulator, prepared.emulator_name),
            profile_step: prepared.step,
            profile_label: prepared.step_label.clone(),
            hardware: self.hw_fingerprint(),
            measure_requested: prepared.measure,
            ..Default::default()
        };
        let journal_path =
            match cl_session::journal::write_record(&self.paths.sessions_dir(), &journal) {
                Ok(path) => Some(path),
                Err(error) => {
                    log::warn!("cannot create session journal: {error}");
                    None
                }
            };
        let capture = prepared.measure.then(|| CaptureRequest {
            dir: self.paths.captures_dir().join(&game.id),
            delay_seconds: self.config.perf.capture_delay_seconds,
            seconds: self.config.perf.capture_seconds,
            presentmon: self.config.perf.presentmon_path.clone(),
        });
        let session = match cl_session::run(&prepared.plan, capture.as_ref()) {
            Ok(session) => session,
            Err(error) => {
                journal.status = "failed".into();
                journal.ended_at = Some(now());
                journal.error = Some(error.to_string());
                if let Some(path) = &journal_path {
                    if let Err(write_error) =
                        cl_session::journal::write_record(&self.paths.sessions_dir(), &journal)
                    {
                        log::warn!(
                            "cannot update session journal {}: {write_error}",
                            path.display()
                        );
                    }
                }
                return Err(error.into());
            }
        };
        let mut notes = prepared.notes.clone();
        if let Some(n) = &session.capture_note {
            notes.push(n.clone());
        }
        let session_can_update_tuning = session_can_update_tuning(session.exit_code);

        let mut stats = None;
        let mut result_verdict = None;
        if let Some(file) = &session.capture_file {
            match parse_file(file)
                .map_err(|e| e.to_string())
                .and_then(|ft| analyse(&ft).map_err(|e| e.to_string()))
            {
                Ok(s) => {
                    let v = verdict(&s);
                    if session_can_update_tuning {
                        tuning.push_run(RunRecord {
                            at: now(),
                            step: prepared.step,
                            stats: s.clone(),
                            verdict: v,
                        });
                    } else {
                        notes.push(
                            "Measurement shown but not used for tuning: emulator failed".into(),
                        );
                    }
                    stats = Some(s);
                    result_verdict = Some(v);
                }
                Err(e) => notes.push(format!("Measurement ignored: {e}")),
            }
        }
        if session_can_update_tuning {
            self.profiles().save(&tuning)?;
        }
        let (status, error) = session_outcome(session.exit_code);
        if let Some(detail) = &error {
            notes.push(detail.clone());
        }
        journal.status = status.into();
        journal.error = error;
        journal.ended_at = Some(now());
        journal.capture_produced = session.capture_file.is_some();
        journal.capture_note = session.capture_note.clone();
        journal.exit_code = session.exit_code;
        journal.played_seconds = Some(session.played_seconds);
        journal.verdict = result_verdict.map(|v| format!("{v:?}"));
        journal.frame_stats = stats.clone();
        if let Some(path) = journal_path {
            if let Err(error) =
                cl_session::journal::write_record(&self.paths.sessions_dir(), &journal)
            {
                log::warn!("cannot update session journal {}: {error}", path.display());
            }
        }

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
        let (adapter, _, _) = self.adapter_for_game(game)?;
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
        let (adapter, _, _) = self.adapter_for_game(game)?;
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
    fn session_outcome_requires_a_zero_exit_code() {
        assert_eq!(session_outcome(Some(0)), ("completed", None));
        assert!(session_can_update_tuning(Some(0)));
        assert_eq!(
            session_outcome(Some(3)),
            ("failed", Some("emulator exited with code 3".into()))
        );
        assert!(!session_can_update_tuning(Some(3)));
        assert_eq!(
            session_outcome(None),
            (
                "failed",
                Some("emulator did not report a normal exit code".into())
            )
        );
        assert!(!session_can_update_tuning(None));
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

        let (p, tuning) = engine.prepare(&games[0]).unwrap();
        assert_eq!(p.emulator, "dolphin");
        // 8 GB discrete GPU → rung 3 of the estimate table.
        assert_eq!(p.step_label, "1440p");
        assert!(!p.measure);
        assert_eq!(tuning.api, cl_core::GraphicsApi::Vulkan);
        assert!(tuning.hardware.ends_with("|dolphin-policy-1"));
        assert!(p
            .plan
            .args
            .contains(&"--config=GFX.Settings.ShaderCompilationMode=2".into()));
        assert!(!p.plan.args.iter().any(|arg| arg.contains("CPUThread")));

        let t = engine.set_step(&games[0], 1).unwrap();
        assert!(t.state.done);
        let (p, _) = engine.prepare(&games[0]).unwrap();
        assert_eq!(p.step_label, "720p");
    }

    #[test]
    fn dolphin_policy_change_discards_stale_automatic_tuning() {
        let dir = tempfile::tempdir().unwrap();
        let mut engine = engine_with_fake_hw(&dir.path().join("data"));
        let game = Game {
            id: "old-dolphin-profile".into(),
            system: SystemId::GameCube,
            title: "Game".into(),
            path: "Game.rvz".into(),
            cover: None,
            favorite: false,
            identity: Default::default(),
            metadata: None,
        };
        engine
            .profiles()
            .save(&GameTuning {
                game_id: game.id.clone(),
                system: game.system,
                emulator: "dolphin".into(),
                emulator_version: None,
                hardware: "Test CPU|Test GPU".into(),
                api: cl_core::GraphicsApi::D3d11,
                state: TuneState::new(0, 5),
                history: Vec::new(),
            })
            .unwrap();

        let tuning = engine.tuning(&game, &cl_adapters::Dolphin);
        assert_eq!(tuning.state.current, 3);
        assert_eq!(tuning.api, cl_core::GraphicsApi::Vulkan);
        assert!(tuning.hardware.ends_with("|dolphin-policy-1"));
    }

    #[test]
    fn imports_a_single_game_and_rejects_the_wrong_console() {
        let dir = tempfile::tempdir().unwrap();
        let game = dir.path().join("Game.rvz");
        touch(&game);
        let mut engine = engine_with_fake_hw(&dir.path().join("data"));

        engine.add_game(game.clone(), SystemId::GameCube).unwrap();
        assert_eq!(engine.games().len(), 1);
        assert_eq!(engine.games()[0].system, SystemId::GameCube);

        assert!(matches!(
            engine.add_game(game, SystemId::Ps2),
            Err(EngineError::UnsupportedGameFile { .. })
        ));
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
            identity: Default::default(),
            metadata: None,
        };
        assert!(matches!(
            engine.prepare(&g),
            Err(EngineError::EmulatorMissing { .. })
        ));
    }

    fn touch(p: &std::path::Path) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"").unwrap();
    }

    fn exe(name: &str) -> String {
        if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_owned()
        }
    }

    /// The situation of a RetroBat user who added the emulator folders as
    /// game folders, because the settings had no other field.
    #[test]
    fn retrobat_paths_are_moved_and_emulators_found() {
        let dir = tempfile::tempdir().unwrap();
        let rb = dir.path().join("RetroBat");
        touch(&rb.join("roms/gba/Pokemon.gba"));
        touch(&rb.join("roms/psx/Game.cue"));
        let dolphin_name = if cfg!(windows) {
            "Dolphin.exe"
        } else {
            "dolphin-emu"
        };
        let mgba_name = if cfg!(windows) { "mGBA.exe" } else { "mgba-qt" };
        touch(&rb.join("emulators/dolphin-emu").join(dolphin_name));
        touch(&rb.join("emulators/mgba").join(mgba_name));
        touch(&rb.join("emulators/pcsx2/resources/x.gz"));

        let data = dir.path().join("data");
        let mut engine = engine_with_fake_hw(&data);
        engine.config.rom_dirs = vec![
            rb.join("roms"),
            rb.join("emulators"),
            rb.join("emulators/dolphin-emu"),
            rb.join("emulators/dolphin-emu").join(dolphin_name),
        ];
        engine.save_config().unwrap();
        let engine = Engine::load(Paths::at(&data)).unwrap();
        assert_eq!(engine.config.rom_dirs, vec![rb.join("roms")]);
        assert_eq!(engine.config.emulator_dirs, vec![rb.join("emulators")]);
        assert_eq!(
            engine.config.emulator("dolphin").path,
            Some(rb.join("emulators/dolphin-emu").join(dolphin_name))
        );

        let games = engine.games();
        assert_eq!(games.len(), 2);
        // GBA: mGBA is installed, so it is chosen before RetroArch.
        assert_eq!(engine.emulator_id_for(SystemId::Gba), "mgba");
        let systems = engine.systems(&games);
        let gba = systems.iter().find(|s| s.info.id == SystemId::Gba).unwrap();
        assert!(gba.emulator_found);
        // PlayStation: nothing installed, the preferred emulator is shown.
        assert_eq!(engine.emulator_id_for(SystemId::Psx), "duckstation");
        let choices = engine.system_choices(&games);
        assert_eq!(choices[0].games, 1);
        assert!(choices.iter().all(|c| !c.options.is_empty()));
    }

    #[test]
    fn user_change_is_kept_until_reset() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("roms/psx/Game.cue"));
        let emu = dir.path().join("duck");
        let duck = emu.join(exe("duckstation-qt"));
        touch(&duck);
        let ini = emu.join("settings.ini");
        std::fs::write(&ini, "[GPU]\nResolutionScale = 1\n").unwrap();

        let mut engine = engine_with_fake_hw(&dir.path().join("data"));
        engine.config.rom_dirs.push(dir.path().join("roms"));
        engine.config.perf.auto_tune = false;
        engine
            .set_emulator_path("duckstation", Some(duck.clone()))
            .unwrap();
        let game = engine.games().remove(0);

        let (p, _) = engine.prepare(&game).unwrap();
        assert_eq!(p.emulator, "duckstation");
        let written = std::fs::read_to_string(&ini).unwrap();
        assert!(!written.contains("ResolutionScale = 1\n"), "{written}");

        // The user picks 2x inside DuckStation.
        let text = std::fs::read_to_string(&ini).unwrap();
        let line = text
            .lines()
            .find(|l| l.starts_with("ResolutionScale"))
            .unwrap();
        std::fs::write(&ini, text.replace(line, "ResolutionScale = 2")).unwrap();
        let (p, _) = engine.prepare(&game).unwrap();
        assert!(
            p.notes
                .iter()
                .any(|n| n.contains("GPU/ResolutionScale") && !n.is_empty()),
            "{:?}",
            p.notes
        );
        assert!(std::fs::read_to_string(&ini)
            .unwrap()
            .contains("ResolutionScale = 2"));
        let st = engine.emulators_status();
        let d = st.iter().find(|e| e.id == "duckstation").unwrap();
        assert_eq!(d.user_settings, vec!["GPU/ResolutionScale"]);
        // Still the user's value on the next launch.
        engine.prepare(&game).unwrap();
        assert!(std::fs::read_to_string(&ini)
            .unwrap()
            .contains("ResolutionScale = 2"));

        // "Re-enable automatic optimisation".
        engine.reset_emulator_tuning("duckstation").unwrap();
        engine.prepare(&game).unwrap();
        assert!(!std::fs::read_to_string(&ini)
            .unwrap()
            .contains("ResolutionScale = 2"));
        assert!(engine
            .emulators_status()
            .iter()
            .all(|e| e.user_settings.is_empty()));
    }

    #[test]
    fn custom_emulator_is_used_and_known_ones_are_recognised() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("roms/snes/Game.sfc"));
        let mine = dir.path().join("bin").join(exe("my-snes-emu"));
        touch(&mine);
        let ppsspp_name = if cfg!(windows) {
            "PPSSPPWindows64.exe"
        } else {
            "PPSSPPSDL"
        };
        let ppsspp = dir.path().join("ppsspp").join(ppsspp_name);
        touch(&ppsspp);

        let mut engine = engine_with_fake_hw(&dir.path().join("data"));
        engine.config.rom_dirs.push(dir.path().join("roms"));
        engine.config.perf.auto_tune = false;
        let id = engine
            .add_custom_emulator(mine.clone(), "Mine".into(), vec![SystemId::Snes], vec![])
            .unwrap();
        assert!(id.starts_with("custom-"));
        // A known executable added "as custom" becomes the known emulator.
        let known = engine
            .add_custom_emulator(ppsspp.clone(), String::new(), vec![SystemId::Psp], vec![])
            .unwrap();
        assert_eq!(known, "ppsspp");
        assert_eq!(engine.config.emulator("ppsspp").path, Some(ppsspp));

        engine
            .set_system_emulator(SystemId::Snes, Some(id.clone()))
            .unwrap();
        let game = engine.games().remove(0);
        let (p, _) = engine.prepare(&game).unwrap();
        assert_eq!(p.emulator, id);
        assert!(!p.tunes_settings);
        assert_eq!(p.plan.args, vec![game.path.display().to_string()]);
        assert!(p.plan.tweaks.high_priority);
        engine.remove_custom_emulator(&id).unwrap();
        assert!(engine.config.system_emulator.is_empty());
    }

    #[test]
    fn custom_emulator_updates_require_valid_launch_data() {
        let dir = tempfile::tempdir().unwrap();
        let exe_path = dir.path().join("bin").join(exe("my-emu"));
        touch(&exe_path);
        let mut engine = engine_with_fake_hw(&dir.path().join("data"));
        let id = engine
            .add_custom_emulator(
                exe_path.clone(),
                "Mine".into(),
                vec![SystemId::Snes],
                vec![],
            )
            .unwrap();

        let missing = dir.path().join("missing-emu");
        let error = engine
            .update_custom_emulator(
                &id,
                CustomEmulator {
                    name: "Mine".into(),
                    path: missing,
                    args: vec!["{rom}".into()],
                    systems: vec![SystemId::Snes],
                },
            )
            .unwrap_err();
        assert!(matches!(error, EngineError::NotAFile(_)));

        let error = engine
            .update_custom_emulator(
                &id,
                CustomEmulator {
                    name: "Mine".into(),
                    path: exe_path,
                    args: vec!["--fullscreen".into()],
                    systems: vec![SystemId::Snes],
                },
            )
            .unwrap_err();
        assert!(matches!(error, EngineError::InvalidCustomEmulator(_)));
    }
}
