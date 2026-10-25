//! Dolphin (GPL-2.0-or-later) for GameCube and Wii.
//!
//! Settings are passed with `--config=System.Section.Key=Value`, which
//! applies to the session only: the user's INI files are not written,
//! for external and managed installs alike. Managed installs also get
//! their own user folder (`--user`), so they stay separate from a copy of
//! Dolphin the user may have installed.
//!
//! The user's INI files are read to notice settings changed by hand in
//! Dolphin (`GFX.Settings.InternalResolution` → `Config/GFX.ini`,
//! section `[Settings]`, key `InternalResolution`).

use crate::{exe_stem, step, AdapterError, EmulatorAdapter, LaunchContext, LaunchPlan};
use cl_core::config::EmulatorMode;
use cl_core::ini::IniDoc;
use cl_core::{GraphicsApi, QualityStep, SystemId};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const BACKEND_KEY: &str = "Dolphin.Core.GFXBackend";

/// Folder holding Dolphin.ini and GFX.ini for this install.
fn config_dir(exe: &Path, mode: EmulatorMode, generated_dir: &Path) -> Option<PathBuf> {
    if mode == EmulatorMode::Managed {
        return Some(generated_dir.join("dolphin-user").join("Config"));
    }
    let exe_dir = exe.parent()?;
    if exe_dir.join("portable.txt").exists() {
        return Some(exe_dir.join("User").join("Config"));
    }
    [
        "{appdata}/Dolphin Emulator/Config",
        "{documents}/Dolphin Emulator/Config",
        "{config}/dolphin-emu",
    ]
    .iter()
    .filter_map(|t| cl_core::paths::expand(t, None))
    .find(|d| d.join("Dolphin.ini").is_file() || d.join("GFX.ini").is_file())
}

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
    fn id(&self) -> &str {
        "dolphin"
    }
    fn name(&self) -> &str {
        "Dolphin"
    }
    fn license(&self) -> &str {
        "GPL-2.0-or-later"
    }
    fn homepage(&self) -> &str {
        "https://dolphin-emu.org"
    }
    fn systems(&self) -> &[SystemId] {
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

    fn apis_for_gpu(&self, vendor_id: u32) -> Vec<GraphicsApi> {
        if cfg!(windows) {
            match vendor_id {
                // Dolphin documents D3D12 as slightly faster than D3D11 on
                // some integrated GPUs, particularly Intel models.
                0x8086 => vec![
                    GraphicsApi::D3d12,
                    GraphicsApi::D3d11,
                    GraphicsApi::Vulkan,
                    GraphicsApi::OpenGl,
                ],
                // Hybrid ubershaders and Vulkan can still stutter on NVIDIA
                // because of driver pipeline swaps. D3D11 is the conservative
                // Windows default; the tuner may still preserve a user choice.
                0x10DE => vec![
                    GraphicsApi::D3d11,
                    GraphicsApi::D3d12,
                    GraphicsApi::Vulkan,
                    GraphicsApi::OpenGl,
                ],
                // Dolphin generally recommends Vulkan for AMD.
                0x1002 | 0x1022 => self.apis(),
                // D3D11 is Dolphin's broadly compatible middle ground.
                _ => vec![
                    GraphicsApi::D3d11,
                    GraphicsApi::D3d12,
                    GraphicsApi::Vulkan,
                    GraphicsApi::OpenGl,
                ],
            }
        } else {
            match vendor_id {
                // Vulkan is not Dolphin's recommended Intel backend. NVIDIA
                // also has a documented Hybrid/Vulkan pipeline-swap caveat.
                0x8086 | 0x10DE => vec![GraphicsApi::OpenGl, GraphicsApi::Vulkan],
                _ => self.apis(),
            }
        }
    }

    fn tuning_revision(&self) -> u32 {
        // Revision 1 removes Skip Drawing and forced Dual Core, and chooses
        // the initial graphics API with Dolphin's vendor guidance.
        1
    }
    fn exe_names(&self) -> Vec<String> {
        let names: &[&str] = if cfg!(windows) {
            &["Dolphin.exe"]
        } else {
            &["dolphin-emu"]
        };
        names.iter().map(|s| s.to_string()).collect()
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
    /// 3 ≈ 1080p, 4 ≈ 1440p, 6 ≈ 4K. Specialized shaders are the safe
    /// low-end default. Higher rungs use Hybrid Ubershaders to reduce shader
    /// compilation stutter without deliberately skipping objects.
    /// Value mapping to check against the Dolphin version in use:
    /// ShaderCompilationMode 0 = specialized, 1 = exclusive ubershaders,
    /// 2 = hybrid ubershaders, 3 = skip drawing.
    fn quality_ladder(&self, _system: SystemId) -> Vec<QualityStep> {
        let mk = |label: &str, ir: &str, shader: &str| {
            let mut s = step(label, &[]);
            s.settings
                .insert("GFX.Settings.InternalResolution".into(), ir.into());
            s.settings
                .insert("GFX.Settings.ShaderCompilationMode".into(), shader.into());
            s
        };
        vec![
            mk("Native", "1", "0"),
            mk("720p", "2", "2"),
            mk("1080p", "3", "2"),
            mk("1440p", "4", "2"),
            mk("4K", "6", "2"),
        ]
    }

    fn managed_keys(&self, system: SystemId) -> Vec<String> {
        let mut keys = crate::ladder_keys(&self.quality_ladder(system));
        keys.push(BACKEND_KEY.into());
        keys
    }

    fn current_values(
        &self,
        exe: &Path,
        mode: EmulatorMode,
        generated_dir: &Path,
    ) -> Option<BTreeMap<String, String>> {
        let dir = config_dir(exe, mode, generated_dir)?;
        let mut files: BTreeMap<String, IniDoc> = BTreeMap::new();
        let mut out = BTreeMap::new();
        for key in self.managed_keys(SystemId::GameCube) {
            let mut parts = key.splitn(3, '.');
            let (Some(file), Some(section), Some(name)) =
                (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let doc = files.entry(file.to_owned()).or_insert_with(|| {
                IniDoc::parse(
                    &std::fs::read_to_string(dir.join(format!("{file}.ini"))).unwrap_or_default(),
                )
            });
            if let Some(v) = doc.get(section, name) {
                out.insert(key.clone(), v);
            }
        }
        Some(out)
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
        if ctx.locked_keys.iter().any(|l| l == BACKEND_KEY) {
            plan.settings_report
                .kept_user_value
                .push(BACKEND_KEY.into());
        } else {
            plan.args
                .push(format!("--config={BACKEND_KEY}={}", backend(ctx.api)));
        }
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
        if let Some(overrides) = ctx.controller_overrides {
            for (key, value) in overrides {
                if !key.starts_with("GCPad") && !key.starts_with("Wiimote") {
                    return Err(AdapterError::Unsupported(format!(
                        "Dolphin controller key '{key}' is not a GCPad/Wiimote setting"
                    )));
                }
                if value.contains('\n') || value.contains('\r') {
                    return Err(AdapterError::Unsupported(
                        "Dolphin controller values cannot contain line breaks".into(),
                    ));
                }
                plan.args.push(format!("--config={key}={value}"));
                plan.settings_report.applied.push(key.clone());
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
                controller_overrides: None,
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

    #[test]
    fn applies_validated_controller_overrides() {
        let dir = tempfile::tempdir().unwrap();
        let g = game(SystemId::GameCube, "/g/gc/a.rvz");
        let ladder = Dolphin.quality_ladder(SystemId::GameCube);
        let overrides =
            BTreeMap::from([("GCPad1/Buttons/A".to_string(), "Keyboard/Space".to_string())]);
        let plan = Dolphin
            .prepare(&LaunchContext {
                exe: std::path::Path::new("/usr/bin/dolphin-emu"),
                game: &g,
                step: &ladder[0],
                api: GraphicsApi::Vulkan,
                mode: EmulatorMode::External,
                generated_dir: dir.path(),
                locked_keys: &[],
                bios_dir: None,
                controller_overrides: Some(&overrides),
            })
            .unwrap();
        assert!(plan
            .args
            .contains(&"--config=GCPad1/Buttons/A=Keyboard/Space".to_string()));
        assert!(plan
            .settings_report
            .applied
            .contains(&"GCPad1/Buttons/A".to_string()));
    }

    #[test]
    fn rejects_non_dolphin_controller_keys() {
        let dir = tempfile::tempdir().unwrap();
        let g = game(SystemId::GameCube, "/g/gc/a.rvz");
        let ladder = Dolphin.quality_ladder(SystemId::GameCube);
        let overrides = BTreeMap::from([("GFX/Unsafe".to_string(), "1".to_string())]);
        let result = Dolphin.prepare(&LaunchContext {
            exe: std::path::Path::new("/usr/bin/dolphin-emu"),
            game: &g,
            step: &ladder[0],
            api: GraphicsApi::Vulkan,
            mode: EmulatorMode::External,
            generated_dir: dir.path(),
            locked_keys: &[],
            bios_dir: None,
            controller_overrides: Some(&overrides),
        });
        assert!(matches!(result, Err(AdapterError::Unsupported(_))));
    }

    #[test]
    fn reads_portable_user_settings() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("Dolphin.exe");
        std::fs::write(&exe, b"").unwrap();
        std::fs::write(dir.path().join("portable.txt"), b"").unwrap();
        let cfg = dir.path().join("User/Config");
        std::fs::create_dir_all(&cfg).unwrap();
        std::fs::write(cfg.join("GFX.ini"), "[Settings]\nInternalResolution = 4\n").unwrap();
        std::fs::write(cfg.join("Dolphin.ini"), "[Core]\nGFXBackend = D3D12\n").unwrap();
        let v = Dolphin
            .current_values(&exe, EmulatorMode::External, dir.path())
            .unwrap();
        assert_eq!(v["GFX.Settings.InternalResolution"], "4");
        assert_eq!(v[BACKEND_KEY], "D3D12");
        assert!(!v.contains_key("Dolphin.Core.CPUThread"));
    }

    #[test]
    fn automatic_ladder_never_skips_drawing_or_forces_dual_core() {
        let ladder = Dolphin.quality_ladder(SystemId::GameCube);
        assert_eq!(
            ladder[0].settings["GFX.Settings.ShaderCompilationMode"],
            "0"
        );
        for step in ladder {
            assert_ne!(step.settings["GFX.Settings.ShaderCompilationMode"], "3");
            assert!(!step.settings.contains_key("Dolphin.Core.CPUThread"));
        }
    }

    #[test]
    fn graphics_api_order_accounts_for_vendor() {
        let intel = Dolphin.apis_for_gpu(0x8086);
        let nvidia = Dolphin.apis_for_gpu(0x10DE);
        let amd = Dolphin.apis_for_gpu(0x1002);
        if cfg!(windows) {
            assert_eq!(intel[0], GraphicsApi::D3d12);
            assert_eq!(nvidia[0], GraphicsApi::D3d11);
            assert_eq!(amd[0], GraphicsApi::Vulkan);
        } else {
            assert_eq!(intel[0], GraphicsApi::OpenGl);
            assert_eq!(nvidia[0], GraphicsApi::OpenGl);
            assert_eq!(amd[0], GraphicsApi::Vulkan);
        }
    }
}
