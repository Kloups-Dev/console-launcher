//! Desktop application: window, controller input, themes, and the commands
//! the theme SDK (`ui/sdk.js`) calls.

mod gamepad;
mod themes;

use cl_core::config::AppConfig;
use cl_core::paths::Paths;
use cl_core::Game;
use cl_engine::Engine;
use serde::Serialize;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

struct AppState {
    paths: Paths,
    themes_dir: std::path::PathBuf,
    engine: Mutex<Engine>,
    session_running: Arc<AtomicBool>,
}

type CmdResult<T> = Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

#[derive(Serialize)]
struct Library {
    systems: Vec<cl_engine::SystemEntry>,
    games: Vec<Game>,
}

fn find_game(engine: &Engine, id: &str) -> CmdResult<Game> {
    engine
        .games()
        .into_iter()
        .find(|g| g.id == id)
        .ok_or_else(|| format!("unknown game {id}"))
}

#[tauri::command]
fn get_library(state: State<AppState>) -> CmdResult<Library> {
    let engine = state.engine.lock().map_err(err)?;
    let games = engine.games();
    Ok(Library {
        systems: engine.systems(&games),
        games,
    })
}

#[tauri::command]
fn game_info(state: State<AppState>, id: String) -> CmdResult<Value> {
    let mut engine = state.engine.lock().map_err(err)?;
    let game = find_game(&engine, &id)?;
    match engine.prepare(&game) {
        Ok((p, t)) => Ok(serde_json::json!({
            "ready": true,
            "emulator": p.emulator,
            "step": p.step,
            "stepLabel": p.step_label,
            "ladder": p.ladder,
            "measure": p.measure,
            "settled": t.state.done,
            "belowRequirements": t.state.below_requirements,
            "notes": p.notes,
        })),
        Err(e) => Ok(serde_json::json!({ "ready": false, "error": e.to_string() })),
    }
}

#[tauri::command]
async fn play(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<cl_engine::PlayReport> {
    if state.session_running.swap(true, Ordering::SeqCst) {
        return Err("a game is already running".into());
    }
    let paths = state.paths.clone();
    let running = state.session_running.clone();
    let _ = app.emit(
        "session",
        serde_json::json!({ "state": "started", "gameId": id }),
    );
    let window = app.get_webview_window("main");
    if let Some(w) = &window {
        let _ = w.minimize();
    }

    // A separate engine instance: the session can last hours and must not
    // block the interface's other commands.
    let result =
        tauri::async_runtime::spawn_blocking(move || -> CmdResult<cl_engine::PlayReport> {
            let mut engine = Engine::load(paths).map_err(err)?;
            let game = find_game(&engine, &id)?;
            engine.play(&game).map_err(err)
        })
        .await
        .map_err(err)
        .and_then(|r| r);

    running.store(false, Ordering::SeqCst);
    if let Some(w) = &window {
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    let _ = app.emit(
        "session",
        match &result {
            Ok(r) => serde_json::json!({ "state": "ended", "report": r }),
            Err(e) => serde_json::json!({ "state": "failed", "error": e }),
        },
    );
    result
}

#[tauri::command]
fn toggle_favorite(state: State<AppState>, id: String) -> CmdResult<bool> {
    state
        .engine
        .lock()
        .map_err(err)?
        .toggle_favorite(&id)
        .map_err(err)
}

#[tauri::command]
fn set_quality(state: State<AppState>, id: String, step: usize) -> CmdResult<()> {
    let mut engine = state.engine.lock().map_err(err)?;
    let game = find_game(&engine, &id)?;
    engine.set_step(&game, step).map(|_| ()).map_err(err)
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> CmdResult<AppConfig> {
    Ok(state.engine.lock().map_err(err)?.config.clone())
}

#[tauri::command]
fn save_settings(state: State<AppState>, settings: AppConfig) -> CmdResult<()> {
    let mut engine = state.engine.lock().map_err(err)?;
    engine.config = settings;
    engine.save_config().map_err(err)
}

#[tauri::command]
fn hardware(state: State<AppState>, refresh: bool) -> CmdResult<Value> {
    let mut engine = state.engine.lock().map_err(err)?;
    let hw = engine.hardware(refresh).clone();
    serde_json::to_value(hw).map_err(err)
}

#[tauri::command]
fn list_themes(state: State<AppState>) -> Vec<Value> {
    themes::list(&state.themes_dir)
        .into_iter()
        .map(|(id, m)| serde_json::json!({ "id": id, "name": m.name, "author": m.author, "description": m.description }))
        .collect()
}

#[tauri::command]
fn load_theme(state: State<AppState>) -> CmdResult<themes::LoadedTheme> {
    let name = state.engine.lock().map_err(err)?.config.theme.clone();
    let dir = state.themes_dir.clone();
    themes::load(&dir, &name).or_else(|e| {
        log::warn!("theme '{name}' failed to load ({e}); falling back to 'horizon'");
        themes::load(&dir, "horizon")
    })
}

#[tauri::command]
fn set_theme(state: State<AppState>, name: String) -> CmdResult<()> {
    themes::load(&state.themes_dir, &name)?;
    let mut engine = state.engine.lock().map_err(err)?;
    engine.config.theme = name;
    engine.save_config().map_err(err)
}

#[tauri::command]
fn set_theme_option(state: State<AppState>, key: String, value: Value) -> CmdResult<()> {
    let name = state.engine.lock().map_err(err)?.config.theme.clone();
    themes::set_option(&state.themes_dir, &name, &key, value)
}

#[tauri::command]
fn quit(app: AppHandle) {
    app.exit(0);
}

pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let paths = Paths::discover();
    // Theme authors can point the launcher at their working folder.
    let themes_dir = std::env::var_os("CONSOLE_LAUNCHER_THEMES")
        .map(Into::into)
        .unwrap_or_else(|| paths.themes_dir());
    if let Err(e) = themes::install_builtins(&themes_dir) {
        log::error!("cannot install the built-in themes: {e}");
    }
    let engine = Engine::load(paths.clone()).unwrap_or_else(|e| {
        log::error!("settings unreadable ({e}); starting with defaults");
        Engine::load(Paths::at(
            std::env::temp_dir().join("console-launcher-fallback"),
        ))
        .expect("fallback engine")
    });
    let session_running = Arc::new(AtomicBool::new(false));
    let pad_pause = session_running.clone();

    tauri::Builder::default()
        .manage(AppState {
            paths,
            themes_dir,
            engine: Mutex::new(engine),
            session_running,
        })
        .setup(move |app| {
            gamepad::spawn(app.handle().clone(), pad_pause);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_library,
            game_info,
            play,
            toggle_favorite,
            set_quality,
            get_settings,
            save_settings,
            hardware,
            list_themes,
            load_theme,
            set_theme,
            set_theme_option,
            quit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the application");
}
