//! Game sessions.
//!
//! Starts the command prepared by an adapter, optionally records frame
//! times, and waits for the emulator to close so the interface can come
//! back to the front.
//!
//! Frame capture tools (both MIT, shipped with the launcher packages in
//! `tools/` next to the executable; a copy installed on the machine is used
//! when present):
//! - Linux: MangoHud, logging enabled and overlay hidden. An installed
//!   `mangohud` is run as a wrapper; the bundled copy is injected directly
//!   (`LD_PRELOAD` for OpenGL, an implicit Vulkan layer for Vulkan).
//! - Windows: PresentMon, started next to the emulator and watching its
//!   process name. PresentMon needs administrator rights or membership of
//!   the "Performance Log Users" group ([`request_capture_permission`]).

use cl_adapters::LaunchPlan;
use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

pub mod journal;
pub use journal::SessionRecord;

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("cannot start {program}: {source}")]
    Spawn {
        program: String,
        source: std::io::Error,
    },
    #[error("error while waiting for the emulator: {0}")]
    Wait(std::io::Error),
}

#[derive(Debug, Clone)]
pub struct CaptureRequest {
    pub dir: PathBuf,
    pub delay_seconds: u32,
    pub seconds: u32,
    /// Windows only.
    pub presentmon: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SessionResult {
    pub exit_code: Option<i32>,
    pub played_seconds: u64,
    pub capture_file: Option<PathBuf>,
    /// Why no capture was produced, when one was requested.
    pub capture_note: Option<String>,
    /// System-level optimisations actually applied.
    pub tweaks_applied: Vec<String>,
}

/// Windows process creation flags.
#[cfg(windows)]
const ABOVE_NORMAL_PRIORITY_CLASS: u32 = 0x0000_8000;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Sets the Windows "high performance" GPU preference for an executable,
/// unless a preference already exists (chosen by the user in the Windows
/// graphics settings, or by an earlier launch). Returns what was done.
#[cfg(windows)]
fn prefer_discrete_gpu(exe: &Path) -> Result<&'static str, String> {
    use std::os::windows::process::CommandExt;
    const KEY: &str = r"HKCU\Software\Microsoft\DirectX\UserGpuPreferences";
    let exe = exe.display().to_string();
    let exists = Command::new("reg")
        .args(["query", KEY, "/v", &exe])
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if exists {
        return Ok("GPU preference already set for this emulator: kept");
    }
    let ok = Command::new("reg")
        .args([
            "add",
            KEY,
            "/v",
            &exe,
            "/t",
            "REG_SZ",
            "/d",
            "GpuPreference=2;",
            "/f",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        Ok("high-performance GPU selected for this emulator")
    } else {
        Err("could not set the GPU preference".into())
    }
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(name))
        .find(|p| p.is_file())
}

/// Folder of the tools shipped with the launcher: `CONSOLE_LAUNCHER_TOOLS`,
/// or `tools/` next to the running executable.
pub fn bundled_tools_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("CONSOLE_LAUNCHER_TOOLS") {
        return Some(PathBuf::from(d));
    }
    let exe = std::env::current_exe().ok()?;
    Some(exe.parent()?.join("tools"))
}

/// The frame capture tool that will be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureTool {
    PresentMon(PathBuf),
    /// `mangohud` wrapper installed on the machine.
    MangoHudWrapper(PathBuf),
    /// Folder holding the bundled `libMangoHud*.so` files.
    MangoHudBundled(PathBuf),
}

impl CaptureTool {
    pub fn path(&self) -> &Path {
        match self {
            CaptureTool::PresentMon(p)
            | CaptureTool::MangoHudWrapper(p)
            | CaptureTool::MangoHudBundled(p) => p,
        }
    }
    /// True when the tool comes with the launcher.
    pub fn bundled(&self) -> bool {
        let Some(dir) = bundled_tools_dir() else {
            return false;
        };
        self.path().starts_with(dir)
    }
}

fn bundled_presentmon() -> Option<PathBuf> {
    let dir = bundled_tools_dir()?.join("presentmon");
    let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .is_some_and(|n| n.starts_with("presentmon") && n.ends_with(".exe"))
        })
        .collect();
    found.sort();
    found.pop()
}

fn bundled_mangohud() -> Option<PathBuf> {
    let dir = bundled_tools_dir()?.join("mangohud");
    (dir.join("libMangoHud_shim.so").is_file() && dir.join("libMangoHud.so").is_file())
        .then_some(dir)
}

/// Finds the capture tool: the path chosen in the settings (Windows), a
/// copy installed on the machine, then the copy shipped with the launcher.
pub fn find_capture_tool(presentmon: Option<&Path>) -> Result<CaptureTool, String> {
    if cfg!(windows) {
        presentmon
            .filter(|p| p.is_file())
            .map(Path::to_path_buf)
            .or_else(|| which("PresentMon.exe"))
            .or_else(bundled_presentmon)
            .map(CaptureTool::PresentMon)
            .ok_or_else(|| "PresentMon not found".into())
    } else {
        which("mangohud")
            .map(CaptureTool::MangoHudWrapper)
            .or_else(|| bundled_mangohud().map(CaptureTool::MangoHudBundled))
            .ok_or_else(|| "MangoHud not found".into())
    }
}

/// Whether frames can be measured: a tool is present and, on Windows, the
/// user may record performance events. Returns the tool path, or why not.
pub fn capture_tool_available(presentmon: Option<&Path>) -> Result<String, String> {
    let tool = find_capture_tool(presentmon)?;
    if !capture_permission_granted() {
        return Err("PresentMon needs the permission to record performance events".into());
    }
    Ok(tool.path().display().to_string())
}

#[cfg(windows)]
const CREATE_NO_WINDOW_FLAG: u32 = 0x0800_0000;

/// Windows: true when the process runs elevated or the user belongs to
/// "Performance Log Users" (SID S-1-5-32-559). Always true elsewhere.
pub fn capture_permission_granted() -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let out = Command::new("whoami")
            .args(["/groups", "/fo", "csv", "/nh"])
            .creation_flags(CREATE_NO_WINDOW_FLAG)
            .stderr(Stdio::null())
            .output();
        match out {
            Ok(o) => {
                let text = String::from_utf8_lossy(&o.stdout);
                // S-1-16-12288: high integrity level (elevated process).
                text.contains("S-1-5-32-559") || text.contains("S-1-16-12288")
            }
            Err(_) => false,
        }
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// Windows: adds the current user to "Performance Log Users", after a
/// Windows administrator prompt. Takes effect at the next sign-in.
pub fn request_capture_permission() -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let user = match (std::env::var("USERDOMAIN"), std::env::var("USERNAME")) {
            (Ok(d), Ok(u)) if !d.is_empty() => format!("{d}\\{u}"),
            (_, Ok(u)) => u,
            _ => return Err("unknown user name".into()),
        };
        let user = user.replace('\'', "''");
        // The inner command runs elevated; quotes are doubled for PowerShell.
        let inner = format!(
            "Add-LocalGroupMember -SID S-1-5-32-559 -Member '{user}' -ErrorAction SilentlyContinue"
        );
        let outer = format!(
            "$p = Start-Process powershell -Verb RunAs -Wait -PassThru -WindowStyle Hidden -ArgumentList '-NoProfile','-Command',\"{inner}\"; exit $p.ExitCode"
        );
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &outer,
            ])
            .creation_flags(CREATE_NO_WINDOW_FLAG)
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err("the administrator prompt was refused".into())
        }
    }
    #[cfg(not(windows))]
    {
        Ok(())
    }
}

/// Vulkan layer description pointing at the bundled MangoHud library.
fn write_mangohud_layer(lib_dir: &Path) -> std::io::Result<PathBuf> {
    let dir = std::env::temp_dir().join("console-launcher-mangohud-layer");
    std::fs::create_dir_all(&dir)?;
    let json = serde_json::json!({
        "file_format_version": "1.0.0",
        "layer": {
            "name": "VK_LAYER_MANGOHUD_overlay_x86_64",
            "type": "GLOBAL",
            "api_version": "1.3.0",
            "library_path": lib_dir.join("libMangoHud.so"),
            "implementation_version": "1",
            "description": "Vulkan Hud Overlay (bundled with Console Launcher)",
            "functions": {
                "vkNegotiateLoaderLayerInterfaceVersion": "vkNegotiateLoaderLayerInterfaceVersion"
            },
            "enable_environment": { "MANGOHUD": "1" },
            "disable_environment": { "DISABLE_MANGOHUD": "1" }
        }
    });
    std::fs::write(
        dir.join("MangoHud.x86_64.json"),
        serde_json::to_string_pretty(&json).unwrap_or_default(),
    )?;
    Ok(dir)
}

fn newest_csv(dir: &Path, since: SystemTime) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("csv"))
        })
        // MangoHud also writes a "_summary.csv"; keep the frame log.
        .filter(|e| !e.path().to_string_lossy().ends_with("_summary.csv"))
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .filter(|(t, _)| *t >= since)
        .max_by_key(|(t, _)| *t)
        .map(|(_, p)| p)
}

/// Builds the command for a plan, with MangoHud on Linux when a capture is
/// requested. Returned separately so it can be unit-tested.
pub fn build_command(
    plan: &LaunchPlan,
    capture: Option<&CaptureRequest>,
) -> (Command, Option<String>) {
    let mut note = None;
    let mangohud_config = |c: &CaptureRequest| {
        format!(
            "no_display,output_folder={},autostart_log={},log_duration={}",
            c.dir.display(),
            c.delay_seconds,
            c.seconds
        )
    };
    let mut cmd = match capture {
        Some(c) if !cfg!(windows) => match find_capture_tool(None) {
            Ok(CaptureTool::MangoHudWrapper(mangohud)) => {
                let mut cmd = Command::new(mangohud);
                cmd.arg(&plan.program);
                cmd.env("MANGOHUD_CONFIG", mangohud_config(c));
                cmd
            }
            Ok(CaptureTool::MangoHudBundled(dir)) => {
                let mut cmd = Command::new(&plan.program);
                cmd.env("MANGOHUD", "1");
                cmd.env("MANGOHUD_CONFIG", mangohud_config(c));
                // OpenGL: the shim loads libMangoHud_opengl.so from its folder.
                let shim = dir.join("libMangoHud_shim.so").display().to_string();
                let preload = match std::env::var("LD_PRELOAD") {
                    Ok(p) if !p.is_empty() => format!("{p}:{shim}"),
                    _ => shim,
                };
                cmd.env("LD_PRELOAD", preload);
                // Vulkan: implicit layer enabled by MANGOHUD=1.
                match write_mangohud_layer(&dir) {
                    Ok(layer_dir) => {
                        cmd.env("VK_ADD_IMPLICIT_LAYER_PATH", layer_dir);
                    }
                    Err(e) => log::warn!("MangoHud Vulkan layer: {e}"),
                }
                cmd
            }
            _ => {
                note = Some("MangoHud not found: session not measured".to_string());
                Command::new(&plan.program)
            }
        },
        _ => Command::new(&plan.program),
    };
    cmd.args(&plan.args);
    for (k, v) in &plan.env {
        cmd.env(k, v);
    }
    if let Some(dir) = plan.program.parent() {
        if !dir.as_os_str().is_empty() {
            cmd.current_dir(dir);
        }
    }
    cmd.stdin(Stdio::null());
    (cmd, note)
}

fn presentmon_command(
    exe: &Path,
    c: &CaptureRequest,
    file: &Path,
    process_id: u32,
    diagnostic_log: &Path,
) -> Command {
    let mut cmd = Command::new(exe);
    cmd.args([
        "--process_id",
        &process_id.to_string(),
        "--output_file",
        &file.display().to_string(),
        "--delay",
        &c.delay_seconds.to_string(),
        "--timed",
        &c.seconds.to_string(),
        "--terminate_after_timed",
        "--terminate_on_proc_exit",
        "--session_name",
        "ConsoleLauncher",
        "--stop_existing_session",
        "--v1_metrics",
        "--no_track_input",
        "--no_console_stats",
    ]);
    if let Ok(log) = File::create(diagnostic_log) {
        if let Ok(errors) = log.try_clone() {
            cmd.stdout(Stdio::from(log));
            cmd.stderr(Stdio::from(errors));
        }
    }
    cmd
}

fn start_presentmon(
    c: &CaptureRequest,
    file: &Path,
    process_id: u32,
    diagnostic_log: &Path,
) -> Result<Child, String> {
    let exe = match find_capture_tool(c.presentmon.as_deref()) {
        Ok(CaptureTool::PresentMon(p)) => p,
        _ => return Err("PresentMon not found: session not measured".into()),
    };
    if !capture_permission_granted() {
        return Err(
            "PresentMon has no permission to record: Settings › Smoothness check › Allow".into(),
        );
    }
    let mut cmd = presentmon_command(&exe, c, file, process_id, diagnostic_log);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW_FLAG);
    }
    cmd.spawn()
        .map_err(|e| format!("PresentMon could not start ({e}): session not measured"))
}

fn presentmon_diagnostic(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.is_empty() {
        return None;
    }
    const LIMIT: usize = 500;
    if clean.len() > LIMIT {
        clean.truncate(LIMIT);
        clean.push('…');
    }
    Some(clean)
}

// Private, short-lived session handoff to the native performance panel.
// No key input, game contents or network traffic is recorded here.
struct LiveSession {
    path: PathBuf,
}

impl LiveSession {
    fn publish_at(path: PathBuf, pid: u32, program: &Path) -> std::io::Result<Self> {
        let bytes = serde_json::to_vec(&serde_json::json!({
            "pid": pid,
            "program": program,
        }))?;
        std::fs::write(&path, bytes)?;
        Ok(Self { path })
    }

    fn publish(pid: u32, program: &Path) -> Option<Self> {
        let path = std::env::var_os("CONSOLE_LAUNCHER_LIVE_SESSION")?;
        match Self::publish_at(path.into(), pid, program) {
            Ok(session) => Some(session),
            Err(error) => {
                log::warn!("live performance panel unavailable: {error}");
                None
            }
        }
    }
}

impl Drop for LiveSession {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Runs the session and blocks until the emulator exits.
pub fn run(
    plan: &LaunchPlan,
    capture: Option<&CaptureRequest>,
) -> Result<SessionResult, SessionError> {
    let started_wall = SystemTime::now();
    let started = Instant::now();
    let mut result = SessionResult::default();

    if let Some(c) = capture {
        if let Err(e) = std::fs::create_dir_all(&c.dir) {
            result.capture_note = Some(format!("capture folder: {e}"));
        }
    }

    let (mut cmd, note) = build_command(plan, capture);
    result.capture_note = result.capture_note.or(note);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if plan.tweaks.high_priority {
            cmd.creation_flags(ABOVE_NORMAL_PRIORITY_CLASS);
            result.tweaks_applied.push("higher process priority".into());
        }
        if plan.tweaks.prefer_discrete_gpu {
            match prefer_discrete_gpu(&plan.program) {
                Ok(m) => result.tweaks_applied.push(m.into()),
                Err(e) => log::warn!("{e}"),
            }
        }
    }
    log::info!("launch: {:?}", cmd);
    let mut child = cmd.spawn().map_err(|source| SessionError::Spawn {
        program: plan.program.display().to_string(),
        source,
    })?;

    let _live_session = LiveSession::publish(child.id(), &plan.program);

    let mut presentmon = None;
    let pm_file = capture.map(|c| {
        c.dir.join(format!(
            "presentmon-{}.csv",
            started_wall
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        ))
    });
    let pm_log = pm_file.as_ref().map(|file| file.with_extension("log"));
    if cfg!(windows) {
        if let (Some(c), Some(f), Some(log)) = (capture, &pm_file, &pm_log) {
            match start_presentmon(c, f, child.id(), log) {
                Ok(p) => presentmon = Some(p),
                Err(e) => result.capture_note = Some(e),
            }
        }
    }

    let status = child.wait().map_err(SessionError::Wait)?;
    result.exit_code = status.code();
    result.played_seconds = started.elapsed().as_secs();

    let mut presentmon_exit = None;
    if let Some(mut pm) = presentmon {
        // PresentMon stops by itself after the timed window; if the game was
        // closed earlier, stop it.
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if let Ok(Some(status)) = pm.try_wait() {
                presentmon_exit = status.code();
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        if presentmon_exit.is_none() {
            let _ = pm.kill();
            presentmon_exit = pm.wait().ok().and_then(|status| status.code());
        }
        result.capture_file = pm_file.filter(|f| f.is_file());
    } else if let Some(c) = capture {
        result.capture_file = newest_csv(&c.dir, started_wall);
    }
    if let Some(c) =
        capture.filter(|_| result.capture_file.is_none() && result.capture_note.is_none())
    {
        let needed = u64::from(c.delay_seconds + c.seconds);
        result.capture_note = Some(
            if let Some(detail) = pm_log.as_deref().and_then(presentmon_diagnostic) {
                format!("PresentMon did not produce a capture: {detail}")
            } else if presentmon_exit.is_some_and(|code| code != 0) {
                format!(
                    "PresentMon exited with code {}: session not measured",
                    presentmon_exit.unwrap_or_default()
                )
            } else if result.played_seconds < needed {
                format!("session shorter than the measurement window ({needed} s): not measured")
            } else {
                "no capture file produced".into()
            },
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn runs_a_process_and_reports_exit_code() {
        let plan = LaunchPlan {
            program: "/bin/sh".into(),
            args: vec!["-c".into(), "exit 3".into()],
            process_name: "sh".into(),
            ..Default::default()
        };
        let r = run(&plan, None).unwrap();
        assert_eq!(r.exit_code, Some(3));
        assert!(r.capture_file.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn bundled_mangohud_is_injected() {
        let dir = tempfile::tempdir().unwrap();
        let tools = dir.path().join("tools/mangohud");
        std::fs::create_dir_all(&tools).unwrap();
        for f in [
            "libMangoHud.so",
            "libMangoHud_shim.so",
            "libMangoHud_opengl.so",
        ] {
            std::fs::write(tools.join(f), b"").unwrap();
        }
        std::env::set_var("CONSOLE_LAUNCHER_TOOLS", dir.path().join("tools"));
        let plan = LaunchPlan {
            program: "/bin/true".into(),
            ..Default::default()
        };
        let c = CaptureRequest {
            dir: dir.path().join("cap"),
            delay_seconds: 1,
            seconds: 2,
            presentmon: None,
        };
        if which("mangohud").is_none() {
            let (cmd, note) = build_command(&plan, Some(&c));
            assert!(note.is_none());
            let envs: std::collections::HashMap<_, _> = cmd
                .get_envs()
                .map(|(k, v)| {
                    (
                        k.to_string_lossy().into_owned(),
                        v.map(|v| v.to_string_lossy().into_owned()),
                    )
                })
                .collect();
            assert_eq!(envs["MANGOHUD"].as_deref(), Some("1"));
            assert!(envs["LD_PRELOAD"]
                .as_ref()
                .unwrap()
                .ends_with("libMangoHud_shim.so"));
            let layer =
                std::path::PathBuf::from(envs["VK_ADD_IMPLICIT_LAYER_PATH"].clone().unwrap());
            let json = std::fs::read_to_string(layer.join("MangoHud.x86_64.json")).unwrap();
            assert!(json.contains("libMangoHud.so"));
        }
        std::env::remove_var("CONSOLE_LAUNCHER_TOOLS");
    }

    #[test]
    fn live_handoff_contains_only_the_launched_process_and_is_removed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.json");
        let session =
            LiveSession::publish_at(path.clone(), 4242, Path::new("emulator.exe")).unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(value["pid"], 4242);
        assert_eq!(value["program"], "emulator.exe");
        assert_eq!(value.as_object().unwrap().len(), 2);
        drop(session);
        assert!(!path.exists());
    }
    #[test]
    fn missing_program_is_an_error() {
        let plan = LaunchPlan {
            program: "/definitely/not/here".into(),
            ..Default::default()
        };
        assert!(matches!(run(&plan, None), Err(SessionError::Spawn { .. })));
    }

    #[test]
    fn presentmon_targets_only_the_launched_process() {
        let dir = tempfile::tempdir().unwrap();
        let request = CaptureRequest {
            dir: dir.path().into(),
            delay_seconds: 20,
            seconds: 60,
            presentmon: None,
        };
        let command = presentmon_command(
            Path::new("PresentMon.exe"),
            &request,
            &dir.path().join("frames.csv"),
            4242,
            &dir.path().join("presentmon.log"),
        );
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert!(args.windows(2).any(|pair| pair == ["--process_id", "4242"]));
        assert!(args.contains(&"--terminate_on_proc_exit".into()));
        assert!(args.contains(&"--no_track_input".into()));
        assert!(!args.contains(&"--process_name".into()));
    }
}
