//! PCSX2 (GPL-3.0-or-later) for PlayStation 2.
//!
//! PCSX2 has no generic command-line setting override, so profiles are
//! applied only to **managed** installs, which run in portable mode
//! (`portable.txt` next to the executable, settings in `inis/PCSX2.ini`).
//! The INI is edited with the ownership rule: values the user changed by
//! hand are kept. External installs are launched with their own settings.

use crate::{exe_stem, step, AdapterError, EmulatorAdapter, LaunchContext, LaunchPlan};
use cl_core::config::EmulatorMode;
use cl_core::ini::{apply_owned, IniDoc, Ledger};
use cl_core::{GraphicsApi, QualityStep, SystemId};
use std::path::PathBuf;

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
    fn id(&self) -> &'static str {
        "pcsx2"
    }
    fn name(&self) -> &'static str {
        "PCSX2"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn homepage(&self) -> &'static str {
        "https://pcsx2.net"
    }
    fn systems(&self) -> &'static [SystemId] {
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
    fn exe_names(&self) -> &'static [&'static str] {
        if cfg!(windows) {
            &["pcsx2-qt.exe"]
        } else {
            &["pcsx2-qt"]
        }
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
            let ini_path = root.join("inis").join("PCSX2.ini");
            if ini_path.is_file() {
                let ledger_path = ctx.generated_dir.join("pcsx2-ledger.json");
                let mut ledger: Ledger = std::fs::read_to_string(&ledger_path)
                    .ok()
                    .and_then(|t| serde_json::from_str(&t).ok())
                    .unwrap_or_default();
                let mut changes = ctx.step.settings.clone();
                changes.insert("EmuCore/GS/Renderer".into(), renderer(ctx.api).into());
                let mut doc = IniDoc::parse(&std::fs::read_to_string(&ini_path)?);
                plan.settings_report =
                    apply_owned(&mut doc, &changes, &mut ledger, ctx.locked_keys);
                if !plan.settings_report.applied.is_empty() {
                    std::fs::write(&ini_path, doc.render())?;
                }
                std::fs::create_dir_all(ctx.generated_dir)?;
                std::fs::write(
                    &ledger_path,
                    serde_json::to_string_pretty(&ledger).unwrap_or_default(),
                )?;
            } else {
                plan.notes.push(
                    "PCSX2 has not been set up yet: start it once from the settings screen to choose your BIOS. \
                     The optimised profile applies from the next launch."
                        .into(),
                );
            }
        } else {
            plan.notes.push(
                "PCSX2 installed by the user: its own settings are used. Switch it to 'managed' to let the launcher optimise it."
                    .into(),
            );
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
    fn external_install_is_not_modified() {
        let dir = tempfile::tempdir().unwrap();
        let g = game(SystemId::Ps2, "/g/ps2/a.iso");
        let ladder = Pcsx2.quality_ladder(SystemId::Ps2);
        let exe = dir.path().join("pcsx2-qt");
        let plan = Pcsx2
            .prepare(&LaunchContext {
                exe: &exe,
                game: &g,
                step: &ladder[1],
                api: GraphicsApi::Vulkan,
                mode: EmulatorMode::External,
                generated_dir: dir.path(),
                locked_keys: &[],
                bios_dir: None,
            })
            .unwrap();
        assert!(!dir.path().join("portable.txt").exists());
        assert!(plan.settings_report.applied.is_empty());
        assert_eq!(plan.notes.len(), 1);
    }
}
