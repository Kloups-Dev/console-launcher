//! Desktop application: window, controller input, themes, and the commands
//! the theme SDK (`ui/sdk.js`) calls.

mod gamepad;
mod themes;

use cl_core::config::{AppConfig, CustomEmulator};
use cl_core::paths::Paths;
use cl_core::{Game, SystemId};
use cl_engine::{Engine, EngineError};
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

/// Engine errors in the interface language.
fn eng_err(engine: &Engine) -> impl Fn(EngineError) -> String + '_ {
    move |e| e.localized(&engine.config.language)
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
    match engine.preview(&game) {
        Ok((p, t)) => Ok(serde_json::json!({
            "ready": true,
            "emulator": p.emulator_name,
            "emulatorId": p.emulator,
            "tunesSettings": p.tunes_settings,
            "step": p.step,
            "stepLabel": p.step_label,
            "ladder": p.ladder,
            "measure": p.measure,
            "settled": t.state.done,
            "belowRequirements": t.state.below_requirements,
            "notes": p.notes,
        })),
        Err(e) => {
            let emulator = match &e {
                EngineError::EmulatorMissing { id, .. } => Some(id.clone()),
                _ => None,
            };
            Ok(serde_json::json!({
                "ready": false,
                "code": e.code(),
                "emulatorId": emulator,
                "error": e.localized(&engine.config.language),
            }))
        }
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
            let lang = engine.config.language.clone();
            engine.play(&game).map_err(|e| e.localized(&lang))
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

/// Everything the settings screen shows.
#[tauri::command]
fn settings_overview(state: State<AppState>) -> CmdResult<Value> {
    let engine = state.engine.lock().map_err(err)?;
    let games = engine.games();
    let capture = cl_session::capture_tool_available(engine.config.perf.presentmon_path.as_deref());
    Ok(serde_json::json!({
        "config": engine.config,
        "emulators": engine.emulators_status(),
        "systems": engine.system_choices(&games),
        "emulatorDirsSearched": engine.emulator_search_dirs(),
        "capture": { "available": capture.is_ok(), "detail": capture.unwrap_or_else(|e| e) },
        "profileErrors": engine.profile_errors,
        "dataDir": state.paths.root,
        "windows": cfg!(windows),
    }))
}

#[tauri::command]
fn set_system_emulator(
    state: State<AppState>,
    system: SystemId,
    id: Option<String>,
) -> CmdResult<()> {
    let mut engine = state.engine.lock().map_err(err)?;
    let r = engine.set_system_emulator(system, id);
    r.map_err(eng_err(&engine))
}

#[tauri::command]
fn set_emulator_path(state: State<AppState>, id: String, path: Option<String>) -> CmdResult<()> {
    let mut engine = state.engine.lock().map_err(err)?;
    let r = engine.set_emulator_path(&id, path.map(Into::into));
    r.map_err(eng_err(&engine))
}

#[tauri::command]
fn add_custom_emulator(
    state: State<AppState>,
    path: String,
    name: String,
    systems: Vec<SystemId>,
    args: Vec<String>,
) -> CmdResult<String> {
    let mut engine = state.engine.lock().map_err(err)?;
    let r = engine.add_custom_emulator(path.into(), name, systems, args);
    r.map_err(eng_err(&engine))
}

#[tauri::command]
fn update_custom_emulator(
    state: State<AppState>,
    id: String,
    emulator: CustomEmulator,
) -> CmdResult<()> {
    let mut engine = state.engine.lock().map_err(err)?;
    let r = engine.update_custom_emulator(&id, emulator);
    r.map_err(eng_err(&engine))
}

#[tauri::command]
fn remove_custom_emulator(state: State<AppState>, id: String) -> CmdResult<()> {
    let mut engine = state.engine.lock().map_err(err)?;
    let r = engine.remove_custom_emulator(&id);
    r.map_err(eng_err(&engine))
}

#[tauri::command]
fn reset_emulator_tuning(state: State<AppState>, id: String) -> CmdResult<()> {
    let mut engine = state.engine.lock().map_err(err)?;
    let r = engine.reset_emulator_tuning(&id);
    r.map_err(eng_err(&engine))
}

#[tauri::command]
fn open_emulator(state: State<AppState>, id: String) -> CmdResult<()> {
    let engine = state.engine.lock().map_err(err)?;
    engine.open_emulator(&id).map_err(eng_err(&engine))
}

/// Adds a folder or an executable where it belongs (games, emulators).
#[tauri::command]
fn add_path(state: State<AppState>, path: String) -> CmdResult<cl_engine::FolderKind> {
    let mut engine = state.engine.lock().map_err(err)?;
    let r = engine.add_path(path.into());
    r.map_err(eng_err(&engine))
}

#[tauri::command]
fn remove_path(state: State<AppState>, path: String) -> CmdResult<()> {
    let mut engine = state.engine.lock().map_err(err)?;
    let r = engine.remove_path(std::path::Path::new(&path));
    r.map_err(eng_err(&engine))
}

#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct PerfUpdate {
    auto_tune: Option<bool>,
    system_tweaks: Option<bool>,
    /// Empty string clears the path.
    presentmon_path: Option<String>,
    language: Option<String>,
}

#[tauri::command]
fn update_preferences(state: State<AppState>, update: PerfUpdate) -> CmdResult<()> {
    let mut engine = state.engine.lock().map_err(err)?;
    if let Some(v) = update.auto_tune {
        engine.config.perf.auto_tune = v;
    }
    if let Some(v) = update.system_tweaks {
        engine.config.perf.system_tweaks = v;
    }
    if let Some(p) = update.presentmon_path {
        engine.config.perf.presentmon_path = (!p.is_empty()).then(|| p.into());
    }
    if let Some(l) = update.language {
        engine.config.language = l;
    }
    engine.save_config().map_err(err)
}

/// Native file / folder picker. `kind`: "folder", "file" or "exe".
#[tauri::command]
async fn pick_path(
    app: AppHandle,
    kind: String,
    title: Option<String>,
) -> CmdResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        let mut d = app.dialog().file();
        if let Some(t) = title {
            d = d.set_title(t);
        }
        if kind == "exe" && cfg!(windows) {
            d = d.add_filter("Programme / Program", &["exe"]);
        }
        let picked = if kind == "folder" {
            d.blocking_pick_folder()
        } else {
            d.blocking_pick_file()
        };
        picked
            .and_then(|p| p.into_path().ok())
            .map(|p| p.display().to_string())
    })
    .await
    .map_err(err)
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
        .plugin(tauri_plugin_dialog::init())
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
            settings_overview,
            set_system_emulator,
            set_emulator_path,
            add_custom_emulator,
            update_custom_emulator,
            remove_custom_emulator,
            reset_emulator_tuning,
            open_emulator,
            add_path,
            remove_path,
            update_preferences,
            pick_path,
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
