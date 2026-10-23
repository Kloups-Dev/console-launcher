//! Emulators described by a profile file instead of Rust code.
//!
//! A profile (TOML) tells the launcher how to find an emulator, how to start
//! a game, where its settings live and which settings make up each rung of
//! the quality ladder. The emulators bundled with the launcher are described
//! this way (`crates/cl-adapters/profiles/`), and anyone can add a profile
//! in the `emulator-profiles` folder of the data directory to teach the
//! launcher a new emulator. Reference: docs/prototype/EMULATORS.en.md.

use crate::{
    edit_ini, exe_stem, first_existing, read_ini_values, AdapterError, EmulatorAdapter,
    LaunchContext, LaunchPlan,
};
use cl_core::config::{CustomEmulator, EmulatorMode};
use cl_core::{GraphicsApi, QualityStep, SystemId};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// One value per operating system.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PerOs {
    pub windows: Vec<String>,
    pub linux: Vec<String>,
}

impl PerOs {
    fn current(&self) -> &[String] {
        if cfg!(windows) {
            &self.windows
        } else {
            &self.linux
        }
    }
}

/// How the profile's settings reach the emulator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    /// The launcher does not know the emulator's settings.
    #[default]
    None,
    /// The emulator's INI file is edited (managed keys only, backup kept).
    Ini,
    /// Settings are passed on the command line for the session only.
    Cli,
}

/// Value of the graphics back-end setting for each API the emulator offers.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ApiMap {
    pub key: String,
    pub vulkan: Option<String>,
    pub d3d12: Option<String>,
    pub d3d11: Option<String>,
    pub opengl: Option<String>,
}

impl ApiMap {
    fn value(&self, api: GraphicsApi) -> Option<&String> {
        match api {
            GraphicsApi::Vulkan => self.vulkan.as_ref(),
            GraphicsApi::D3d12 => self.d3d12.as_ref(),
            GraphicsApi::D3d11 => self.d3d11.as_ref(),
            GraphicsApi::OpenGl => self.opengl.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub method: Method,
    /// Where the emulator keeps its settings, most specific first (portable
    /// install, then per-user folders). Placeholders: see `cl_core::paths::expand`.
    /// For `cli`, the file is only read, to notice changes made by the user.
    pub files: Vec<String>,
    /// `cli` only: arguments added for each setting, `{key}` and `{value}`
    /// being replaced (for example `["-C", "{key}={value}"]`).
    pub cli: Vec<String>,
    /// `cli` only: INI section where the emulator stores those keys.
    pub file_section: Option<String>,
    /// Settings applied on every launch, whatever the rung.
    pub always: BTreeMap<String, String>,
    pub api: Option<ApiMap>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepDef {
    pub label: String,
    #[serde(default)]
    pub set: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub homepage: String,
    pub systems: Vec<SystemId>,
    #[serde(default)]
    pub exe: PerOs,
    #[serde(default)]
    pub paths: PerOs,
    /// Command-line arguments; `{rom}` is replaced by the game file.
    #[serde(default = "default_args")]
    pub args: Vec<String>,
    /// Arguments that open the emulator without a game.
    #[serde(default)]
    pub setup_args: Vec<String>,
    #[serde(default)]
    pub settings: Settings,
    /// Quality ladder, lightest first.
    #[serde(default)]
    pub ladder: Vec<StepDef>,
    /// True when the ladder raises the rendering resolution.
    #[serde(default = "yes")]
    pub gpu_bound: bool,
}

fn default_args() -> Vec<String> {
    vec!["{rom}".into()]
}

fn yes() -> bool {
    true
}

impl Profile {
    pub fn from_toml(text: &str, file: &str) -> Result<Profile, AdapterError> {
        let p: Profile = toml::from_str(text).map_err(|e| AdapterError::Profile {
            file: file.into(),
            message: e.to_string(),
        })?;
        let bad = |message: &str| AdapterError::Profile {
            file: file.into(),
            message: message.into(),
        };
        if p.id.trim().is_empty() || p.id.contains(['/', '\\']) {
            return Err(bad("`id` must be a simple name"));
        }
        if p.systems.is_empty() {
            return Err(bad("`systems` is empty"));
        }
        if p.settings.method == Method::Cli && p.settings.cli.is_empty() {
            return Err(bad("`settings.cli` is required with method = \"cli\""));
        }
        Ok(p)
    }

    /// Profile of an emulator added by the user without a profile file:
    /// launched directly, no setting tuned.
    pub fn custom(id: &str, c: &CustomEmulator) -> Profile {
        let exe = c
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .into_iter()
            .collect::<Vec<_>>();
        Profile {
            id: id.into(),
            name: if c.name.trim().is_empty() {
                c.path
                    .file_stem()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| id.into())
            } else {
                c.name.clone()
            },
            license: String::new(),
            homepage: String::new(),
            systems: c.systems.clone(),
            exe: PerOs {
                windows: exe.clone(),
                linux: exe,
            },
            paths: PerOs::default(),
            args: if c.args.is_empty() {
                default_args()
            } else {
                c.args.clone()
            },
            setup_args: Vec::new(),
            settings: Settings::default(),
            ladder: Vec::new(),
            gpu_bound: true,
        }
    }
}

const BUILTIN: [(&str, &str); 5] = [
    ("mgba.toml", include_str!("../profiles/mgba.toml")),
    (
        "duckstation.toml",
        include_str!("../profiles/duckstation.toml"),
    ),
    ("ppsspp.toml", include_str!("../profiles/ppsspp.toml")),
    ("melonds.toml", include_str!("../profiles/melonds.toml")),
    ("cemu.toml", include_str!("../profiles/cemu.toml")),
];

/// Profiles bundled with the launcher.
pub fn builtin_profiles() -> Vec<Profile> {
    BUILTIN
        .iter()
        .map(|(f, t)| Profile::from_toml(t, f).expect("bundled profiles are checked by tests"))
        .collect()
}

/// Profiles found in a folder (`*.toml`). Invalid files are reported, not fatal.
pub fn load_dir(dir: &Path) -> (Vec<Profile>, Vec<AdapterError>) {
    let mut ok = Vec::new();
    let mut errors = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return (ok, errors);
    };
    let mut files: Vec<PathBuf> = rd
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
        })
        .collect();
    files.sort();
    for f in files {
        let name = f.display().to_string();
        match std::fs::read_to_string(&f) {
            Ok(t) => match Profile::from_toml(&t, &name) {
                Ok(p) => ok.push(p),
                Err(e) => errors.push(e),
            },
            Err(e) => errors.push(AdapterError::Io(e)),
        }
    }
    (ok, errors)
}

pub struct ProfileAdapter {
    p: Profile,
}

impl ProfileAdapter {
    pub fn new(p: Profile) -> ProfileAdapter {
        ProfileAdapter { p }
    }

    pub fn profile(&self) -> &Profile {
        &self.p
    }

    /// Settings to apply for a rung: the rung, the fixed settings and the API.
    fn changes(&self, step: &QualityStep, api: GraphicsApi) -> BTreeMap<String, String> {
        let mut out = self.p.settings.always.clone();
        out.extend(step.settings.clone());
        if let Some(m) = &self.p.settings.api {
            if let Some(v) = m.value(api) {
                out.insert(m.key.clone(), v.clone());
            }
        }
        out
    }

    fn settings_file(&self, exe: &Path) -> Option<PathBuf> {
        first_existing(&self.p.settings.files, exe)
    }
}

fn fill(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut s = template.to_owned();
    for (k, v) in pairs {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

impl EmulatorAdapter for ProfileAdapter {
    fn id(&self) -> &str {
        &self.p.id
    }
    fn name(&self) -> &str {
        &self.p.name
    }
    fn license(&self) -> &str {
        &self.p.license
    }
    fn homepage(&self) -> &str {
        &self.p.homepage
    }
    fn systems(&self) -> &[SystemId] {
        &self.p.systems
    }
    fn apis(&self) -> Vec<GraphicsApi> {
        let Some(m) = &self.p.settings.api else {
            return Vec::new();
        };
        let order = if cfg!(windows) {
            vec![
                GraphicsApi::Vulkan,
                GraphicsApi::D3d12,
                GraphicsApi::D3d11,
                GraphicsApi::OpenGl,
            ]
        } else {
            vec![GraphicsApi::Vulkan, GraphicsApi::OpenGl]
        };
        order
            .into_iter()
            .filter(|a| m.value(*a).is_some())
            .collect()
    }
    fn exe_names(&self) -> Vec<String> {
        self.p.exe.current().to_vec()
    }
    fn common_paths(&self) -> Vec<PathBuf> {
        self.p
            .paths
            .current()
            .iter()
            .filter_map(|t| cl_core::paths::expand(t, None))
            .collect()
    }
    fn quality_ladder(&self, _system: SystemId) -> Vec<QualityStep> {
        if self.p.ladder.is_empty() || self.p.settings.method == Method::None {
            return vec![QualityStep {
                label: "Standard".into(),
                settings: BTreeMap::new(),
            }];
        }
        self.p
            .ladder
            .iter()
            .map(|s| QualityStep {
                label: s.label.clone(),
                settings: s.set.clone(),
            })
            .collect()
    }
    fn gpu_bound_ladder(&self) -> bool {
        self.p.gpu_bound
    }
    fn tunes_settings(&self) -> bool {
        self.p.settings.method != Method::None && !self.p.ladder.is_empty()
    }
    fn setup_args(&self) -> Vec<String> {
        self.p.setup_args.clone()
    }
    fn managed_keys(&self, system: SystemId) -> Vec<String> {
        if self.p.settings.method == Method::None {
            return Vec::new();
        }
        let mut keys: BTreeSet<String> = crate::ladder_keys(&self.quality_ladder(system))
            .into_iter()
            .collect();
        keys.extend(self.p.settings.always.keys().cloned());
        if let Some(m) = &self.p.settings.api {
            keys.insert(m.key.clone());
        }
        keys.into_iter().collect()
    }

    fn current_values(
        &self,
        exe: &Path,
        _mode: EmulatorMode,
        _generated_dir: &Path,
    ) -> Option<BTreeMap<String, String>> {
        let keys = self.managed_keys(self.p.systems[0]);
        let file = self.settings_file(exe)?;
        match self.p.settings.method {
            Method::None => None,
            Method::Ini => read_ini_values(&file, &keys),
            Method::Cli => {
                let section = self.p.settings.file_section.as_deref()?;
                let in_file: Vec<String> = keys.iter().map(|k| format!("{section}/{k}")).collect();
                let values = read_ini_values(&file, &in_file)?;
                Some(
                    keys.iter()
                        .zip(&in_file)
                        .filter_map(|(k, f)| values.get(f).map(|v| (k.clone(), v.clone())))
                        .collect(),
                )
            }
        }
    }

    fn prepare(&self, ctx: &LaunchContext) -> Result<LaunchPlan, AdapterError> {
        let mut plan = LaunchPlan {
            program: ctx.exe.to_path_buf(),
            process_name: exe_stem(ctx.exe),
            ..Default::default()
        };
        let changes = self.changes(ctx.step, ctx.api);
        let rom = ctx.game.path.display().to_string();
        match self.p.settings.method {
            Method::None => {}
            Method::Ini => match self.settings_file(ctx.exe) {
                Some(file) => {
                    plan.settings_report =
                        edit_ini(&file, &self.p.id, &changes, ctx.locked_keys, ctx.generated_dir)?;
                }
                None if !changes.is_empty() => plan.notes.push(format!(
                    "{} has not created its settings file yet: open it once from the launcher settings. \
                     The optimised settings apply from the next launch.",
                    self.p.name
                )),
                None => {}
            },
            Method::Cli => {
                for (k, v) in &changes {
                    if ctx.locked_keys.iter().any(|l| l == k) {
                        plan.settings_report.kept_user_value.push(k.clone());
                        continue;
                    }
                    for t in &self.p.settings.cli {
                        plan.args.push(fill(t, &[("key", k), ("value", v)]));
                    }
                    plan.settings_report.applied.push(k.clone());
                }
            }
        }
        plan.args
            .extend(self.p.args.iter().map(|a| fill(a, &[("rom", &rom)])));
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::game;

    fn ctx<'a>(
        exe: &'a Path,
        g: &'a cl_core::Game,
        step: &'a QualityStep,
        gen: &'a Path,
        locked: &'a [String],
    ) -> LaunchContext<'a> {
        LaunchContext {
            exe,
            game: g,
            step,
            api: GraphicsApi::Vulkan,
            mode: EmulatorMode::External,
            generated_dir: gen,
            locked_keys: locked,
            bios_dir: None,
        }
    }

    #[test]
    fn bundled_profiles_are_valid() {
        let all = builtin_profiles();
        let ids: Vec<_> = all.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, ["mgba", "duckstation", "ppsspp", "melonds", "cemu"]);
        for p in &all {
            assert!(
                !p.exe.windows.is_empty() && !p.exe.linux.is_empty(),
                "{}",
                p.id
            );
            assert!(p.args.iter().any(|a| a.contains("{rom}")), "{}", p.id);
        }
    }

    #[test]
    fn ini_profile_edits_the_portable_settings_file() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("duckstation-qt");
        std::fs::write(&exe, b"").unwrap();
        std::fs::write(
            dir.path().join("settings.ini"),
            "[Main]\nSettingsVersion = 3\n\n[GPU]\nResolutionScale = 1\n",
        )
        .unwrap();
        let a = ProfileAdapter::new(
            builtin_profiles()
                .into_iter()
                .find(|p| p.id == "duckstation")
                .unwrap(),
        );
        let ladder = a.quality_ladder(SystemId::Psx);
        let g = game(SystemId::Psx, "/g/psx/a.chd");
        let gen = dir.path().join("gen");
        let plan = a.prepare(&ctx(&exe, &g, &ladder[2], &gen, &[])).unwrap();
        assert_eq!(plan.args, ["-batch", "-fullscreen", "--", "/g/psx/a.chd"]);
        let text = std::fs::read_to_string(dir.path().join("settings.ini")).unwrap();
        assert!(text.contains("ResolutionScale = 3"));
        assert!(text.contains("Renderer = Vulkan"));
        assert!(text.contains("SettingsVersion = 3"));
        let cur = a
            .current_values(&exe, EmulatorMode::External, &gen)
            .unwrap();
        assert_eq!(
            cur.get("GPU/ResolutionScale").map(String::as_str),
            Some("3")
        );
    }

    #[test]
    fn cli_profile_passes_overrides_and_reads_user_file() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("mGBA.exe");
        std::fs::write(&exe, b"").unwrap();
        std::fs::write(dir.path().join("config.ini"), "[ports.qt]\nframeskip=2\n").unwrap();
        let a = ProfileAdapter::new(
            builtin_profiles()
                .into_iter()
                .find(|p| p.id == "mgba")
                .unwrap(),
        );
        let ladder = a.quality_ladder(SystemId::Gba);
        let g = game(SystemId::Gba, "/g/gba/a.gba");
        let gen = dir.path().join("gen");
        let plan = a.prepare(&ctx(&exe, &g, &ladder[0], &gen, &[])).unwrap();
        assert_eq!(plan.args, ["-C", "frameskip=1", "-f", "/g/gba/a.gba"]);
        let locked = vec!["frameskip".to_string()];
        let plan = a
            .prepare(&ctx(&exe, &g, &ladder[0], &gen, &locked))
            .unwrap();
        assert_eq!(plan.args, ["-f", "/g/gba/a.gba"]);
        let cur = a
            .current_values(&exe, EmulatorMode::External, &gen)
            .unwrap();
        assert_eq!(cur.get("frameskip").map(String::as_str), Some("2"));
    }

    #[test]
    fn custom_emulator_launches_without_tuning() {
        let a = ProfileAdapter::new(Profile::custom(
            "custom-x",
            &CustomEmulator {
                name: String::new(),
                path: "/opt/emu/MyEmu".into(),
                args: vec!["--fullscreen".into(), "{rom}".into()],
                systems: vec![SystemId::Snes],
            },
        ));
        assert_eq!(a.name(), "MyEmu");
        assert!(!a.tunes_settings());
        assert_eq!(a.quality_ladder(SystemId::Snes).len(), 1);
        let dir = tempfile::tempdir().unwrap();
        let g = game(SystemId::Snes, "/g/a.sfc");
        let step = &a.quality_ladder(SystemId::Snes)[0];
        let plan = a
            .prepare(&ctx(Path::new("/opt/emu/MyEmu"), &g, step, dir.path(), &[]))
            .unwrap();
        assert_eq!(plan.args, ["--fullscreen", "/g/a.sfc"]);
    }

    #[test]
    fn invalid_profiles_are_reported() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("bad.toml"),
            "id = \"x\"\nname = \"X\"\nsystems = []\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("good.toml"),
            "id = \"y\"\nname = \"Y\"\nsystems = [\"snes\"]\n[exe]\nlinux = [\"y\"]\nwindows = [\"y.exe\"]\n",
        )
        .unwrap();
        let (ok, errors) = load_dir(dir.path());
        assert_eq!(ok.len(), 1);
        assert_eq!(errors.len(), 1);
    }
}
