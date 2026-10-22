//! Game sessions.
//!
//! Starts the command prepared by an adapter, optionally records frame
//! times, and waits for the emulator to close so the interface can come
//! back to the front.
//!
//! Frame capture tools (not shipped, detected on the machine):
//! - Linux: MangoHud (MIT), run as a wrapper with logging enabled and the
//!   overlay hidden.
//! - Windows: PresentMon (MIT), started next to the emulator and watching
//!   its process name. PresentMon needs administrator rights or membership
//!   of the "Performance Log Users" group.

use cl_adapters::LaunchPlan;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

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

/// Whether a capture tool is available on this machine.
pub fn capture_tool_available(presentmon: Option<&Path>) -> Result<String, String> {
    if cfg!(windows) {
        match presentmon
            .map(Path::to_path_buf)
            .or_else(|| which("PresentMon.exe"))
        {
            Some(p) if p.is_file() => Ok(p.display().to_string()),
            _ => Err("PresentMon not found: set perf.presentmon_path in config.toml".into()),
        }
    } else {
        which("mangohud")
            .map(|p| p.display().to_string())
            .ok_or_else(|| "MangoHud not installed".into())
    }
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

/// Builds the command for a plan, with MangoHud wrapping on Linux when a
/// capture is requested. Returned separately so it can be unit-tested.
pub fn build_command(
    plan: &LaunchPlan,
    capture: Option<&CaptureRequest>,
) -> (Command, Option<String>) {
    let mut note = None;
    let mut cmd = match capture {
        Some(c) if !cfg!(windows) => match which("mangohud") {
            Some(mangohud) => {
                let mut cmd = Command::new(mangohud);
                cmd.arg(&plan.program);
                cmd.env(
                    "MANGOHUD_CONFIG",
                    format!(
                        "no_display,output_folder={},autostart_log={},log_duration={}",
                        c.dir.display(),
                        c.delay_seconds,
                        c.seconds
                    ),
                );
                cmd
            }
            None => {
                note = Some("MangoHud not installed: session not measured".to_string());
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

fn start_presentmon(plan: &LaunchPlan, c: &CaptureRequest, file: &Path) -> Result<Child, String> {
    let exe = c
        .presentmon
        .clone()
        .or_else(|| which("PresentMon.exe"))
        .ok_or("PresentMon not found: session not measured")?;
    Command::new(exe)
        .args([
            "--process_name",
            &plan.process_name,
            "--output_file",
            &file.display().to_string(),
            "--delay",
            &c.delay_seconds.to_string(),
            "--timed",
            &c.seconds.to_string(),
            "--terminate_after_timed",
            "--session_name",
            "ConsoleLauncher",
            "--stop_existing_session",
            "--v1_metrics",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("PresentMon could not start ({e}): session not measured"))
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
    if cfg!(windows) {
        if let (Some(c), Some(f)) = (capture, &pm_file) {
            match start_presentmon(plan, c, f) {
                Ok(p) => presentmon = Some(p),
                Err(e) => result.capture_note = Some(e),
            }
        }
    }

    let status = child.wait().map_err(SessionError::Wait)?;
    result.exit_code = status.code();
    result.played_seconds = started.elapsed().as_secs();

    if let Some(mut pm) = presentmon {
        // PresentMon stops by itself after the timed window; if the game was
        // closed earlier, stop it.
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline {
            if let Ok(Some(_)) = pm.try_wait() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = pm.kill();
        result.capture_file = pm_file.filter(|f| f.is_file());
    } else if let Some(c) = capture {
        result.capture_file = newest_csv(&c.dir, started_wall);
    }
    if let Some(c) =
        capture.filter(|_| result.capture_file.is_none() && result.capture_note.is_none())
    {
        let needed = u64::from(c.delay_seconds + c.seconds);
        result.capture_note = Some(if result.played_seconds < needed {
            format!("session shorter than the measurement window ({needed} s): not measured")
        } else {
            "no capture file produced".into()
        });
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

    #[test]
    fn missing_program_is_an_error() {
        let plan = LaunchPlan {
            program: "/definitely/not/here".into(),
            ..Default::default()
        };
        assert!(matches!(run(&plan, None), Err(SessionError::Spawn { .. })));
    }
}
