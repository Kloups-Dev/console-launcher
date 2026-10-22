//! Dolphin (GPL-2.0-or-later) for GameCube and Wii.
//!
//! Settings are passed with `--config=System.Section.Key=Value`, which
//! applies to the session only: the user's INI files are not written,
//! for external and managed installs alike. Managed installs also get
//! their own user folder (`--user`), so they stay separate from a copy of
//! Dolphin the user may have installed.

use crate::{exe_stem, step, AdapterError, EmulatorAdapter, LaunchContext, LaunchPlan};
use cl_core::config::EmulatorMode;
use cl_core::{GraphicsApi, QualityStep, SystemId};
use std::path::PathBuf;

pub struct Dolphin;

fn backend(api: GraphicsApi) -> &'static str {
    match api {
        GraphicsApi::Vulkan => "Vulkan",
        GraphicsApi::D3d11 => "D3D",
        GraphicsApi::D3d12 => "D3D12",
        GraphicsApi::OpenGl => "OGL",
    }
}

impl EmulatorAdapter for Dolphin {
    fn id(&self) -> &'static str {
        "dolphin"
    }
    fn name(&self) -> &'static str {
        "Dolphin"
    }
    fn license(&self) -> &'static str {
        "GPL-2.0-or-later"
    }
    fn homepage(&self) -> &'static str {
        "https://dolphin-emu.org"
    }
    fn systems(&self) -> &'static [SystemId] {
        &[SystemId::GameCube, SystemId::Wii]
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
            &["Dolphin.exe"]
        } else {
            &["dolphin-emu"]
        }
    }
    fn common_paths(&self) -> Vec<PathBuf> {
        if cfg!(windows) {
            vec!["C:\\Program Files\\Dolphin\\Dolphin.exe".into()]
        } else {
            vec![
                "/usr/bin/dolphin-emu".into(),
                "/usr/games/dolphin-emu".into(),
            ]
        }
    }

    /// Internal resolution multiplier: 1 = native (~480p), 2 ≈ 720p,
    /// 3 ≈ 1080p, 4 ≈ 1440p, 6 ≈ 4K. Hybrid ubershaders avoid shader
    /// compilation stutter at a GPU cost, so the lightest step keeps
    /// asynchronous compilation instead.
    /// Value mapping to check against the Dolphin version in use:
    /// ShaderCompilationMode 0 = specialized, 1 = exclusive ubershaders,
    /// 2 = hybrid ubershaders, 3 = skip drawing.
    fn quality_ladder(&self, _system: SystemId) -> Vec<QualityStep> {
        let common = [("Dolphin.Core.CPUThread", "True")];
        let mk = |label: &str, ir: &str, shader: &str| {
            let mut s = step(label, &common);
            s.settings
                .insert("GFX.Settings.InternalResolution".into(), ir.into());
            s.settings
                .insert("GFX.Settings.ShaderCompilationMode".into(), shader.into());
            s
        };
        vec![
            mk("Native", "1", "3"),
            mk("720p", "2", "2"),
            mk("1080p", "3", "2"),
            mk("1440p", "4", "2"),
            mk("4K", "6", "2"),
        ]
    }

    /// Dolphin skips duplicate frames by default; measured sessions present
    /// all of them. Key name to confirm on the Dolphin version in use.
    fn measurement_args(&self) -> Vec<String> {
        vec!["--config=GFX.Settings.SkipPresentingDuplicateFrames=False".into()]
    }

    fn prepare(&self, ctx: &LaunchContext) -> Result<LaunchPlan, AdapterError> {
        let mut plan = LaunchPlan {
            program: ctx.exe.to_path_buf(),
            process_name: exe_stem(ctx.exe),
            ..Default::default()
        };
        plan.args.push("--batch".into());
        if ctx.mode == EmulatorMode::Managed {
            let user = ctx.generated_dir.join("dolphin-user");
            std::fs::create_dir_all(&user)?;
            plan.args.push(format!("--user={}", user.display()));
        }
        plan.args.push(format!(
            "--config=Dolphin.Core.GFXBackend={}",
            backend(ctx.api)
        ));
        plan.args
            .push("--config=Dolphin.Display.Fullscreen=True".into());
        for (k, v) in &ctx.step.settings {
            if ctx.locked_keys.iter().any(|l| l == k) {
                plan.settings_report.kept_user_value.push(k.clone());
            } else {
                plan.args.push(format!("--config={k}={v}"));
                plan.settings_report.applied.push(k.clone());
            }
        }
        plan.args
            .push(format!("--exec={}", ctx.game.path.display()));
        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::game;

    #[test]
    fn builds_session_overrides() {
        let dir = tempfile::tempdir().unwrap();
        let g = game(SystemId::GameCube, "/g/gc/a.rvz");
        let ladder = Dolphin.quality_ladder(SystemId::GameCube);
        let plan = Dolphin
            .prepare(&LaunchContext {
                exe: std::path::Path::new("/usr/bin/dolphin-emu"),
                game: &g,
                step: &ladder[2],
                api: GraphicsApi::Vulkan,
                mode: EmulatorMode::External,
                generated_dir: dir.path(),
                locked_keys: &[],
                bios_dir: None,
            })
            .unwrap();
        assert_eq!(plan.args.first().unwrap(), "--batch");
        assert!(plan
            .args
            .contains(&"--config=GFX.Settings.InternalResolution=3".to_string()));
        assert!(plan
            .args
            .contains(&"--config=Dolphin.Core.GFXBackend=Vulkan".to_string()));
        assert!(!plan.args.iter().any(|a| a.starts_with("--user")));
        assert_eq!(plan.args.last().unwrap(), "--exec=/g/gc/a.rvz");
    }
}
