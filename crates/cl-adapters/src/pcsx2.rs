//! PCSX2 (GPL-3.0-or-later) for PlayStation 2.
//!
//! PCSX2 has no generic command-line setting override, so the profile is
//! written into its `PCSX2.ini` with the ownership rule ([`crate::edit_ini`]):
//! only the managed keys change, the original file is backed up once, and
//! values the user changed by hand are kept.
//! - Managed installs run in portable mode (`portable.txt` next to the
//!   executable, settings in `inis/PCSX2.ini`).
//! - Installs made by the user keep their own location (portable folder, or
//!   the per-user folder).

use crate::{
    edit_ini, exe_stem, read_ini_values, step, AdapterError, EmulatorAdapter, LaunchContext,
    LaunchPlan,
};
use cl_core::config::EmulatorMode;
use cl_core::{GraphicsApi, QualityStep, SystemId};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const RENDERER_KEY: &str = "EmuCore/GS/Renderer";

/// `PCSX2.ini` of this install, if PCSX2 has already created it.
fn ini_path(exe: &Path, mode: EmulatorMode) -> Option<PathBuf> {
    let root = exe.parent()?;
    let portable = root.join("portable.txt").exists() || root.join("portable.ini").exists();
    if mode == EmulatorMode::Managed || portable {
        let p = root.join("inis").join("PCSX2.ini");
        return p.is_file().then_some(p);
    }
    [
        "{documents}/PCSX2/inis/PCSX2.ini",
        "{config}/PCSX2/inis/PCSX2.ini",
    ]
    .iter()
    .filter_map(|t| cl_core::paths::expand(t, None))
    .find(|p| p.is_file())
}

pub struct Pcsx2;

/// `GSRendererType` values used in PCSX2.ini.
fn renderer(api: GraphicsApi) -> &'static str {
    match api {
        GraphicsApi::D3d11 => "3",
        GraphicsApi::OpenGl => "12",
        GraphicsApi::Vulkan => "14",
        GraphicsApi::D3d12 => "15",
    }
}

impl EmulatorAdapter for Pcsx2 {
    fn id(&self) -> &str {
        "pcsx2"
    }
    fn name(&self) -> &str {
        "PCSX2"
    }
    fn license(&self) -> &str {
        "GPL-3.0-or-later"
    }
    fn homepage(&self) -> &str {
        "https://pcsx2.net"
    }
    fn systems(&self) -> &[SystemId] {
        &[SystemId::Ps2]
    }
    fn apis(&self) -> Vec<GraphicsApi> {
        if cfg!(windows) {
            vec![
                GraphicsApi::Vulkan,
                GraphicsApi::D3d12,
                GraphicsApi::D3d11,
                GraphicsApi::OpenGl,
            ]
        } else {
            vec![GraphicsApi::Vulkan, GraphicsApi::OpenGl]
        }
    }
    fn exe_names(&self) -> Vec<String> {
        let names: &[&str] = if cfg!(windows) {
            &["pcsx2-qt.exe", "pcsx2-qtx64-avx2.exe", "pcsx2-qtx64.exe"]
        } else {
            &["pcsx2-qt", "PCSX2.AppImage"]
        };
        names.iter().map(|s| s.to_string()).collect()
    }
    fn common_paths(&self) -> Vec<PathBuf> {
        if cfg!(windows) {
            vec!["C:\\Program Files\\PCSX2\\pcsx2-qt.exe".into()]
        } else {
            vec!["/usr/bin/pcsx2-qt".into()]
        }
    }

    /// Upscaling multiplier of the native resolution.
    fn quality_ladder(&self, _system: SystemId) -> Vec<QualityStep> {
        [
            ("Native", "1"),
            ("720p", "2"),
            ("1080p", "3"),
            ("1440p", "4"),
            ("4K", "6"),
        ]
        .into_iter()
        .map(|(label, mult)| step(label, &[("EmuCore/GS/upscale_multiplier", mult)]))
        .collect()
    }

    fn managed_keys(&self, system: SystemId) -> Vec<String> {
        let mut keys = crate::ladder_keys(&self.quality_ladder(system));
        keys.push(RENDERER_KEY.into());
        keys
    }

    fn current_values(
        &self,
        exe: &Path,
        mode: EmulatorMode,
        _generated_dir: &Path,
    ) -> Option<BTreeMap<String, String>> {
        read_ini_values(&ini_path(exe, mode)?, &self.managed_keys(SystemId::Ps2))
    }

    fn prepare(&self, ctx: &LaunchContext) -> Result<LaunchPlan, AdapterError> {
        let mut plan = LaunchPlan {
            program: ctx.exe.to_path_buf(),
            process_name: exe_stem(ctx.exe),
            ..Default::default()
        };

        if ctx.mode == EmulatorMode::Managed {
            let root = ctx
                .exe
                .parent()
                .ok_or_else(|| AdapterError::NotFound("PCSX2 folder".into()))?;
            let portable = root.join("portable.txt");
            if !portable.exists() {
                std::fs::write(&portable, b"")?;
            }
        }
        match ini_path(ctx.exe, ctx.mode) {
            Some(ini) => {
                let mut changes = ctx.step.settings.clone();
                changes.insert(RENDERER_KEY.into(), renderer(ctx.api).into());
                plan.settings_report =
                    edit_ini(&ini, self.id(), &changes, ctx.locked_keys, ctx.generated_dir)?;
            }
            None => plan.notes.push(
                "PCSX2 has not been set up yet: open it once from the launcher settings to choose your BIOS. \
                 The optimised settings apply from the next launch."
                    .into(),
            ),
        }

        plan.args = vec![
            "-batch".into(),
            "-nogui".into(),
            "-fullscreen".into(),
            "--".into(),
            ctx.game.path.display().to_string(),
        ];
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::game;

    #[test]
    fn managed_install_edits_only_owned_keys() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("pcsx2");
        std::fs::create_dir_all(root.join("inis")).unwrap();
        let exe = root.join("pcsx2-qt");
        std::fs::write(&exe, b"").unwrap();
        let ini = root.join("inis/PCSX2.ini");
        std::fs::write(
            &ini,
            "[UI]\nTheme = darkfusion\n\n[EmuCore/GS]\nupscale_multiplier = 1\nRenderer = -1\n",
        )
        .unwrap();

        let g = game(SystemId::Ps2, "/g/ps2/a.iso");
        let ladder = Pcsx2.quality_ladder(SystemId::Ps2);
        let gen = dir.path().join("gen");
        let ctx = |step| LaunchContext {
            exe: &exe,
            game: &g,
            step,
            api: GraphicsApi::Vulkan,
            mode: EmulatorMode::Managed,
            generated_dir: &gen,
            locked_keys: &[],
            bios_dir: None,
            controller_overrides: None,
        };
        let plan = Pcsx2.prepare(&ctx(&ladder[2])).unwrap();
        assert!(root.join("portable.txt").exists());
        let text = std::fs::read_to_string(&ini).unwrap();
        assert!(text.contains("upscale_multiplier = 3"));
        assert!(text.contains("Renderer = 14"));
        assert!(text.contains("Theme = darkfusion"));
        assert_eq!(plan.args.last().unwrap(), "/g/ps2/a.iso");

        // The user sets 2x by hand; the next launch keeps it.
        std::fs::write(
            &ini,
            text.replace("upscale_multiplier = 3", "upscale_multiplier = 2"),
        )
        .unwrap();
        let plan = Pcsx2.prepare(&ctx(&ladder[4])).unwrap();
        assert_eq!(
            plan.settings_report.kept_user_value,
            vec!["EmuCore/GS/upscale_multiplier"]
        );
        assert!(std::fs::read_to_string(&ini)
            .unwrap()
            .contains("upscale_multiplier = 2"));
    }

    #[test]
    fn external_portable_install_is_optimised_with_backup() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("pcsx2");
        std::fs::create_dir_all(root.join("inis")).unwrap();
        std::fs::write(root.join("portable.txt"), b"").unwrap();
        let exe = root.join("pcsx2-qt.exe");
        std::fs::write(&exe, b"").unwrap();
        let ini = root.join("inis/PCSX2.ini");
        std::fs::write(&ini, "[EmuCore/GS]\nupscale_multiplier = 1\n").unwrap();
        let g = game(SystemId::Ps2, "/g/ps2/a.iso");
        let ladder = Pcsx2.quality_ladder(SystemId::Ps2);
        let gen = dir.path().join("gen");
        let plan = Pcsx2
            .prepare(&LaunchContext {
                exe: &exe,
                game: &g,
                step: &ladder[1],
                api: GraphicsApi::Vulkan,
                mode: EmulatorMode::External,
                generated_dir: &gen,
                locked_keys: &[],
                bios_dir: None,
                controller_overrides: None,
            })
            .unwrap();
        assert!(plan.notes.is_empty());
        assert!(std::fs::read_to_string(&ini)
            .unwrap()
            .contains("upscale_multiplier = 2"));
        assert!(gen.join("backups/pcsx2/PCSX2.ini.orig").is_file());
        let v = Pcsx2
            .current_values(&exe, EmulatorMode::External, &gen)
            .unwrap();
        assert_eq!(v["EmuCore/GS/upscale_multiplier"], "2");
    }
}
