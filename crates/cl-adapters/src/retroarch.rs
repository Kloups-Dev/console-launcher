//! RetroArch (GPL-3.0) with libretro cores, for the lighter systems.
//!
//! Settings are passed with `--appendconfig`, so the user's own
//! `retroarch.cfg` is never written. The override also disables
//! `config_save_on_exit` for the session, otherwise RetroArch could save the
//! appended values into the user's file when the game closes.
//!
//! The user's `retroarch.cfg` is read to notice settings changed by hand.

use crate::{exe_stem, step, AdapterError, EmulatorAdapter, LaunchContext, LaunchPlan};
use cl_core::config::EmulatorMode;
use cl_core::{GraphicsApi, QualityStep, SystemId};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const DRIVER_KEY: &str = "video_driver";

/// The user's main configuration file.
fn user_cfg(exe: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(d) = exe.parent() {
        candidates.push(d.join("retroarch.cfg"));
    }
    for t in [
        "{config}/retroarch/retroarch.cfg",
        "{home}/.var/app/org.libretro.RetroArch/config/retroarch/retroarch.cfg",
        "{appdata}/RetroArch/retroarch.cfg",
    ] {
        candidates.extend(cl_core::paths::expand(t, None));
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// Reads `key = "value"` lines.
pub fn parse_cfg(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.starts_with('#') {
                return None;
            }
            let (k, v) = l.split_once('=')?;
            Some((k.trim().to_owned(), v.trim().trim_matches('"').to_owned()))
        })
        .collect()
}

pub struct RetroArch;

/// Default core per system. Licences: see docs/prototype/DEPENDENCIES.*.md.
pub fn default_core(system: SystemId) -> Option<&'static str> {
    use SystemId::*;
    Some(match system {
        Nes => "mesen",
        Snes => "snes9x",
        N64 => "mupen64plus_next",
        Gb | Gbc => "gambatte",
        Gba => "mgba",
        Nds => "melondsds",
        MasterSystem | MegaDrive => "genesis_plus_gx",
        Psx => "mednafen_psx_hw",
        Psp => "ppsspp",
        Ps2 | GameCube | Wii | WiiU => return None,
    })
}

fn core_file(core: &str) -> String {
    let ext = if cfg!(windows) {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    format!("{core}_libretro.{ext}")
}

fn core_dirs(exe: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = exe.parent() {
        dirs.push(d.join("cores"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        dirs.push(home.join(".config/retroarch/cores"));
        dirs.push(home.join(".var/app/org.libretro.RetroArch/config/retroarch/cores"));
    }
    dirs.push("/usr/lib/libretro".into());
    dirs.push("/usr/lib/x86_64-linux-gnu/libretro".into());
    dirs.push("/usr/local/lib/libretro".into());
    dirs
}

pub fn find_core(exe: &Path, core: &str) -> Option<PathBuf> {
    let file = core_file(core);
    core_dirs(exe)
        .into_iter()
        .map(|d| d.join(&file))
        .find(|p| p.is_file())
}

fn video_driver(api: GraphicsApi) -> &'static str {
    match api {
        GraphicsApi::Vulkan => "vulkan",
        GraphicsApi::D3d11 => "d3d11",
        GraphicsApi::D3d12 => "d3d12",
        GraphicsApi::OpenGl => "glcore",
    }
}

impl EmulatorAdapter for RetroArch {
    fn id(&self) -> &str {
        "retroarch"
    }
    fn name(&self) -> &str {
        "RetroArch"
    }
    fn license(&self) -> &str {
        "GPL-3.0-or-later"
    }
    fn homepage(&self) -> &str {
        "https://www.retroarch.com"
    }
    fn systems(&self) -> &[SystemId] {
        use SystemId::*;
        &[
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
            Psp,
        ]
    }
    fn apis(&self) -> Vec<GraphicsApi> {
        if cfg!(windows) {
            vec![GraphicsApi::Vulkan, GraphicsApi::D3d11, GraphicsApi::OpenGl]
        } else {
            vec![GraphicsApi::Vulkan, GraphicsApi::OpenGl]
        }
    }
    fn exe_names(&self) -> Vec<String> {
        let names: &[&str] = if cfg!(windows) {
            &["retroarch.exe"]
        } else {
            &["retroarch", "RetroArch-Linux-x86_64.AppImage"]
        };
        names.iter().map(|s| s.to_string()).collect()
    }
    fn common_paths(&self) -> Vec<PathBuf> {
        if cfg!(windows) {
            vec![
                "C:\\RetroArch-Win64\\retroarch.exe".into(),
                "C:\\RetroArch\\retroarch.exe".into(),
            ]
        } else {
            vec![
                "/usr/bin/retroarch".into(),
                "/usr/local/bin/retroarch".into(),
            ]
        }
    }

    fn gpu_bound_ladder(&self) -> bool {
        false
    }

    fn managed_keys(&self, system: SystemId) -> Vec<String> {
        let mut keys = crate::ladder_keys(&self.quality_ladder(system));
        keys.push(DRIVER_KEY.into());
        keys.extend(
            crate::experience::settings("retroarch", cl_core::config::ExperienceMode::Balanced)
                .into_keys(),
        );
        keys
    }

    fn current_values(
        &self,
        exe: &Path,
        _mode: EmulatorMode,
        _generated_dir: &Path,
    ) -> Option<BTreeMap<String, String>> {
        let all = parse_cfg(&std::fs::read_to_string(user_cfg(exe)?).ok()?);
        let keys = self.managed_keys(SystemId::Snes);
        Some(all.into_iter().filter(|(k, _)| keys.contains(k)).collect())
    }

    /// Automatic tuning only switches threaded video. Run-ahead/rewind are
    /// core-dependent advanced choices, never promoted by a hardware estimate.
    fn quality_ladder(&self, _system: SystemId) -> Vec<QualityStep> {
        vec![
            step(
                "Eco",
                &[("video_threaded", "true"), ("run_ahead_enabled", "false")],
            ),
            step(
                "Standard",
                &[("video_threaded", "false"), ("run_ahead_enabled", "false")],
            ),
        ]
    }

    fn prepare(&self, ctx: &LaunchContext) -> Result<LaunchPlan, AdapterError> {
        let core = default_core(ctx.game.system).ok_or_else(|| {
            AdapterError::Unsupported(format!(
                "RetroArch has no default core for {}",
                ctx.game.system
            ))
        })?;
        let mut plan = LaunchPlan {
            program: ctx.exe.to_path_buf(),
            process_name: exe_stem(ctx.exe),
            ..Default::default()
        };

        let core_path = match find_core(ctx.exe, core) {
            Some(p) => p,
            None => {
                return Err(AdapterError::NotFound(format!(
                    "core '{core}' ({}) — install it from RetroArch's Online Updater",
                    core_file(core)
                )))
            }
        };

        // Override file, regenerated at each launch.
        let mut lines = vec![
            "# Generated by Console Launcher for this session. Safe to delete.".to_string(),
            "config_save_on_exit = \"false\"".into(),
            "video_fullscreen = \"true\"".into(),
            "pause_nonactive = \"false\"".into(),
        ];
        if ctx.mode == EmulatorMode::Managed {
            // Saves live outside emulator versions and generated configuration.
            let root = ctx.generated_dir.parent().unwrap_or(ctx.generated_dir);
            let save = root
                .join("saves/retroarch")
                .join(ctx.game.system.to_string())
                .join(&ctx.game.id);
            let states = root.join("states/retroarch").join(core).join(&ctx.game.id);
            let system = root.join("bios");
            let settings = root.join("settings/retroarch");
            let screenshots = root.join("screenshots/retroarch");
            for dir in [&save, &states, &system, &settings, &screenshots] {
                std::fs::create_dir_all(dir)?;
            }
            for (key, value) in [
                ("savefile_directory", save),
                ("savestate_directory", states),
                ("system_directory", system),
                ("rgui_config_directory", settings.clone()),
                ("core_options_path", settings.join("core-options.cfg")),
                ("content_history_dir", settings),
                ("screenshot_directory", screenshots),
            ] {
                lines.push(format!(
                    "{key} = \"{}\"",
                    value.to_string_lossy().replace('\\', "/")
                ));
            }
            for (key, value) in [
                ("autosave_interval", "10"),
                ("savestate_auto_save", "false"),
                ("savestate_auto_load", "false"),
                ("savestate_file_compression", "true"),
                ("savestate_auto_index", "true"),
                ("savestate_max_keep", "10"),
                ("audio_enable", "true"),
                ("audio_sync", "true"),
                ("audio_rate_control", "true"),
                ("audio_latency", "64"),
                ("audio_resampler", "sinc"),
                ("video_vsync", "true"),
                ("video_swap_interval", "1"),
                ("video_force_aspect", "true"),
                ("video_smooth", "false"),
                ("video_shader_enable", "false"),
                ("video_frame_delay", "0"),
                ("video_black_frame_insertion", "0"),
                ("rewind_enable", "false"),
                ("run_ahead_enabled", "false"),
                ("preemptive_frames_enable", "false"),
                ("input_autodetect_enable", "true"),
                ("input_exit_emulator", "escape"),
                ("input_menu_toggle", "f1"),
                ("input_save_state", "f2"),
                ("input_load_state", "f4"),
                ("quit_press_twice", "false"),
                ("menu_show_load_core", "false"),
                ("menu_show_load_content", "false"),
                ("rgui_show_start_screen", "false"),
            ] {
                if !ctx.locked_keys.iter().any(|k| k == key) {
                    lines.push(format!("{key} = \"{value}\""));
                }
            }
            if cfg!(windows) {
                lines.push("audio_driver = \"wasapi\"".into());
            }
            if let Some(runtime) = ctx.exe.parent() {
                for (key, subdir) in [
                    ("joypad_autoconfig_dir", "autoconfig"),
                    ("libretro_info_path", "info"),
                    ("assets_directory", "assets"),
                ] {
                    lines.push(format!(
                        "{key} = \"{}\"",
                        runtime.join(subdir).to_string_lossy().replace('\\', "/")
                    ));
                }
            }
        }
        if ctx.locked_keys.iter().any(|l| l == DRIVER_KEY) {
            plan.settings_report.kept_user_value.push(DRIVER_KEY.into());
        } else {
            // These hardware-rendered cores require an OpenGL context.
            let driver = if matches!(ctx.game.system, SystemId::N64 | SystemId::Psx) {
                "glcore"
            } else {
                video_driver(ctx.api)
            };
            lines.push(format!("{DRIVER_KEY} = \"{driver}\""));
        }
        if let Some(bios) = ctx.bios_dir {
            lines.push(format!("system_directory = \"{}\"", bios.display()));
        }
        for (k, v) in &ctx.step.settings {
            if ctx.locked_keys.iter().any(|l| l == k) {
                plan.settings_report.kept_user_value.push(k.clone());
            } else {
                lines.push(format!("{k} = \"{v}\""));
                plan.settings_report.applied.push(k.clone());
            }
        }
        let dir = ctx.generated_dir.join("retroarch");
        std::fs::create_dir_all(&dir)?;
        let file = dir.join(format!("{}.cfg", ctx.game.id));
        // RetroArch reads duplicates top-first: emit one entry per key.
        let merged = parse_cfg(&lines.join("\n"));
        let text = merged
            .iter()
            .map(|(k, v)| format!("{k} = \"{v}\"\n"))
            .collect::<String>();
        std::fs::write(&file, text)?;

        plan.args = vec![
            "-L".into(),
            core_path.display().to_string(),
            format!("--appendconfig={}", file.display()),
            "-f".into(),
            ctx.game.path.display().to_string(),
        ];
        if ctx.game.system == SystemId::Psx && ctx.bios_dir.is_none() {
            plan.notes.push(
                "PlayStation: the core needs a BIOS file from your own console in the BIOS folder."
                    .into(),
            );
        }
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::game;

    #[test]
    fn writes_override_and_never_touches_user_config() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("ra/retroarch");
        std::fs::create_dir_all(dir.path().join("ra/cores")).unwrap();
        std::fs::write(&exe, b"").unwrap();
        std::fs::write(dir.path().join("ra/cores").join(core_file("snes9x")), b"").unwrap();

        let g = game(SystemId::Snes, "/games/snes/a.sfc");
        let ladder = RetroArch.quality_ladder(SystemId::Snes);
        assert_eq!(ladder.len(), 2);
        let gen = dir.path().join("gen");
        let locked = vec!["video_threaded".to_string()];
        let plan = RetroArch
            .prepare(&LaunchContext {
                exe: &exe,
                game: &g,
                step: &ladder[1],
                api: GraphicsApi::Vulkan,
                mode: EmulatorMode::External,
                generated_dir: &gen,
                locked_keys: &locked,
                bios_dir: None,
                controller_overrides: None,
            })
            .unwrap();
        assert_eq!(plan.args[0], "-L");
        assert!(plan.args[1].ends_with(&core_file("snes9x")));
        assert_eq!(plan.args.last().unwrap(), "/games/snes/a.sfc");
        let cfg = std::fs::read_to_string(gen.join("retroarch/x.cfg")).unwrap();
        assert!(cfg.contains("config_save_on_exit = \"false\""));
        assert!(cfg.contains("run_ahead_enabled = \"false\""));
        assert!(!cfg.contains("video_threaded"));
        assert_eq!(plan.settings_report.kept_user_value, vec!["video_threaded"]);
    }

    #[test]
    fn reads_user_cfg_next_to_the_executable() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("retroarch.exe");
        std::fs::write(&exe, b"").unwrap();
        std::fs::write(
            dir.path().join("retroarch.cfg"),
            "# comment\nvideo_driver = \"d3d11\"\nrun_ahead_enabled = \"true\"\nmenu_driver = \"ozone\"\n",
        )
        .unwrap();
        let v = RetroArch
            .current_values(&exe, EmulatorMode::External, dir.path())
            .unwrap();
        assert_eq!(v["video_driver"], "d3d11");
        assert_eq!(v["run_ahead_enabled"], "true");
        assert!(!v.contains_key("menu_driver"));
    }

    #[test]
    fn heavy_systems_have_no_run_ahead() {
        assert_eq!(RetroArch.quality_ladder(SystemId::N64).len(), 2);
    }

    #[test]
    fn managed_saves_survive_runtime_versions_and_remain_separate_from_states() {
        let dir = tempfile::tempdir().unwrap();
        let g = game(SystemId::Snes, "/games/a.sfc");
        let generated = dir.path().join("generated");
        let saves = dir.path().join("saves/retroarch/snes/x");
        std::fs::create_dir_all(&saves).unwrap();
        std::fs::write(saves.join("a.srm"), b"progress").unwrap();
        let ladder = RetroArch.quality_ladder(g.system);
        for version in ["old", "new"] {
            let exe = dir
                .path()
                .join("emulators/retroarch/versions")
                .join(version)
                .join("retroarch.exe");
            std::fs::create_dir_all(exe.parent().unwrap().join("cores")).unwrap();
            std::fs::write(
                exe.parent()
                    .unwrap()
                    .join("cores")
                    .join(core_file("snes9x")),
                b"core",
            )
            .unwrap();
            RetroArch
                .prepare(&LaunchContext {
                    exe: &exe,
                    game: &g,
                    step: &ladder[1],
                    api: GraphicsApi::Vulkan,
                    mode: EmulatorMode::Managed,
                    generated_dir: &generated,
                    locked_keys: &[],
                    bios_dir: None,
                    controller_overrides: None,
                })
                .unwrap();
            let settings =
                parse_cfg(&std::fs::read_to_string(generated.join("retroarch/x.cfg")).unwrap());
            assert_eq!(Path::new(&settings["savefile_directory"]), saves);
            assert_ne!(
                settings["savefile_directory"],
                settings["savestate_directory"]
            );
            assert_eq!(settings["audio_sync"], "true");
            assert_eq!(settings["autosave_interval"], "10");
            assert_eq!(settings["run_ahead_enabled"], "false");
            assert_eq!(std::fs::read(saves.join("a.srm")).unwrap(), b"progress");
        }
    }
}
