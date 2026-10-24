//! One entry point over the engine: `call(method, params)` with JSON in and
//! JSON out. The Qt interface reaches it through the C library `cl-ffi`;
//! the method list is documented in docs/prototype/ENGINE_API.en.md.
//!
//! Every method is short except `play`, which blocks until the emulator
//! closes. `play` uses its own engine instance, so the other methods stay
//! available while a game runs.

use cl_core::config::CustomEmulator;
use cl_core::paths::Paths;
use cl_core::{Game, SystemId};
use cl_engine::{Engine, EngineError};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

pub struct Api {
    paths: Paths,
    engine: Mutex<Engine>,
    session_running: AtomicBool,
}

type Res = Result<Value, String>;

fn arg<T: DeserializeOwned>(params: &Value, name: &str) -> Result<T, String> {
    serde_json::from_value(params.get(name).cloned().unwrap_or(Value::Null))
        .map_err(|e| format!("parameter '{name}': {e}"))
}

fn to_value<T: serde::Serialize>(v: T) -> Res {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Preferences {
    auto_tune: Option<bool>,
    system_tweaks: Option<bool>,
    /// Empty string clears the path.
    presentmon_path: Option<String>,
    language: Option<String>,
    theme: Option<String>,
}

impl Api {
    /// Opens the engine on a data folder (`None`: the standard per-user
    /// folder, or `CONSOLE_LAUNCHER_HOME`). Unreadable settings fall back to
    /// defaults in a temporary folder, so the interface still opens.
    pub fn open(data_dir: Option<PathBuf>) -> Api {
        let paths = data_dir.map(Paths::at).unwrap_or_else(Paths::discover);
        let engine = Engine::load(paths.clone()).unwrap_or_else(|e| {
            log::error!("settings unreadable ({e}); starting with defaults");
            Engine::load(Paths::at(
                std::env::temp_dir().join("console-launcher-fallback"),
            ))
            .expect("fallback engine")
        });
        Api {
            paths,
            engine: Mutex::new(engine),
            session_running: AtomicBool::new(false),
        }
    }

    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    /// Calls a method. `params` is a JSON object (or null).
    pub fn call(&self, method: &str, params: &Value) -> Res {
        match method {
            "play" => self.play(params),
            _ => {
                let mut engine = self.engine.lock().map_err(|e| e.to_string())?;
                let lang = engine.config.language.clone();
                let loc = |e: EngineError| e.localized(&lang);
                dispatch(&mut engine, &self.paths, method, params, &loc)
            }
        }
    }

    fn play(&self, params: &Value) -> Res {
        let id: String = arg(params, "id")?;
        if self.session_running.swap(true, Ordering::SeqCst) {
            return Err("a game is already running".into());
        }
        let result = (|| {
            let mut engine = Engine::load(self.paths.clone()).map_err(|e| e.to_string())?;
            let game = find_game(&engine, &id)?;
            let lang = engine.config.language.clone();
            let report = engine.play(&game).map_err(|e| e.localized(&lang))?;
            to_value(report)
        })();
        self.session_running.store(false, Ordering::SeqCst);
        result
    }
}

fn find_game(engine: &Engine, id: &str) -> Result<Game, String> {
    engine
        .games()
        .into_iter()
        .find(|g| g.id == id)
        .ok_or_else(|| format!("unknown game {id}"))
}

fn dispatch(
    engine: &mut Engine,
    paths: &Paths,
    method: &str,
    p: &Value,
    loc: &dyn Fn(EngineError) -> String,
) -> Res {
    match method {
        "version" => Ok(json!(env!("CARGO_PKG_VERSION"))),

        // Settings only (fast: no disk scan).
        "config" => to_value(&engine.config),

        "library" => {
            let games = engine.games();
            Ok(json!({ "systems": engine.systems(&games), "games": games }))
        }

        "game_info" => {
            let game = find_game(engine, &arg::<String>(p, "id")?)?;
            Ok(match engine.preview(&game) {
                Ok((pl, t)) => json!({
                    "ready": true,
                    "emulator": pl.emulator_name,
                    "emulatorId": pl.emulator,
                    "tunesSettings": pl.tunes_settings,
                    "step": pl.step,
                    "stepLabel": pl.step_label,
                    "ladder": pl.ladder,
                    "measure": pl.measure,
                    "settled": t.state.done,
                    "certified": t.state.certified,
                    "certification": t.history.last().map(|run| &run.certification),
                    "belowRequirements": t.state.below_requirements,
                    "notes": pl.notes,
                }),
                Err(e) => {
                    let emulator = match &e {
                        EngineError::EmulatorMissing { id, .. } => Some(id.clone()),
                        _ => None,
                    };
                    json!({
                        "ready": false,
                        "code": e.code(),
                        "emulatorId": emulator,
                        "error": loc(e),
                    })
                }
            })
        }

        "refresh_game_metadata" => {
            let id: String = arg(p, "id")?;
            engine
                .refresh_game_metadata(&id)
                .map_err(loc)
                .and_then(to_value)
        }

        "toggle_favorite" => {
            let on = engine
                .toggle_favorite(&arg::<String>(p, "id")?)
                .map_err(|e| e.to_string())?;
            Ok(json!(on))
        }

        "set_quality" => {
            let game = find_game(engine, &arg::<String>(p, "id")?)?;
            engine
                .set_step(&game, arg(p, "step")?)
                .map_err(loc)
                .map(|_| Value::Null)
        }

        "settings_overview" => {
            let games = engine.games();
            let capture =
                cl_session::find_capture_tool(engine.config.perf.presentmon_path.as_deref());
            let permission = cl_session::capture_permission_granted();
            Ok(json!({
                "config": engine.config,
                "emulators": engine.emulators_status(),
                "systems": engine.system_choices(&games),
                "emulatorDirsSearched": engine.emulator_search_dirs(),
                "capture": {
                    "available": capture.is_ok() && permission,
                    "tool": capture.as_ref().ok().map(|t| t.path().to_path_buf()),
                    "bundled": capture.as_ref().is_ok_and(|t| t.bundled()),
                    "permission": permission,
                    "detail": match &capture { Ok(t) => t.path().display().to_string(), Err(e) => e.clone() },
                },
                "profileErrors": engine.profile_errors,
                "dataDir": paths.root,
                "windows": cfg!(windows),
            }))
        }

        "set_system_emulator" => {
            let system: SystemId = arg(p, "system")?;
            engine
                .set_system_emulator(system, arg(p, "id")?)
                .map_err(loc)
                .map(|_| Value::Null)
        }

        "set_emulator_path" => {
            let id: String = arg(p, "id")?;
            let path: Option<String> = arg(p, "path")?;
            engine
                .set_emulator_path(&id, path.filter(|s| !s.is_empty()).map(Into::into))
                .map_err(loc)
                .map(|_| Value::Null)
        }

        "add_custom_emulator" => {
            let path: String = arg(p, "path")?;
            let name: Option<String> = arg(p, "name")?;
            let systems: Option<Vec<SystemId>> = arg(p, "systems")?;
            let args: Option<Vec<String>> = arg(p, "args")?;
            engine
                .add_custom_emulator(
                    path.into(),
                    name.unwrap_or_default(),
                    systems.unwrap_or_default(),
                    args.unwrap_or_default(),
                )
                .map_err(loc)
                .map(Value::String)
        }

        "update_custom_emulator" => {
            let id: String = arg(p, "id")?;
            let c: CustomEmulator = arg(p, "emulator")?;
            engine
                .update_custom_emulator(&id, c)
                .map_err(loc)
                .map(|_| Value::Null)
        }

        "remove_custom_emulator" => engine
            .remove_custom_emulator(&arg::<String>(p, "id")?)
            .map_err(loc)
            .map(|_| Value::Null),

        "reset_emulator_tuning" => engine
            .reset_emulator_tuning(&arg::<String>(p, "id")?)
            .map_err(loc)
            .map(|_| Value::Null),

        "open_emulator" => engine
            .open_emulator(&arg::<String>(p, "id")?)
            .map_err(loc)
            .map(|_| Value::Null),

        "install_emulator" => engine
            .install_emulator(&arg::<String>(p, "id")?)
            .map_err(loc)
            .and_then(to_value),

        "add_path" => {
            let path: String = arg(p, "path")?;
            engine.add_path(path.into()).map_err(loc).and_then(to_value)
        }

        "add_game" => {
            let path: String = arg(p, "path")?;
            let system: SystemId = arg(p, "system")?;
            engine
                .add_game(path.into(), system)
                .map_err(loc)
                .map(|_| Value::Null)
        }

        "remove_game" => {
            let path: String = arg(p, "path")?;
            engine
                .remove_game(std::path::Path::new(&path))
                .map_err(loc)
                .map(|_| Value::Null)
        }

        "remove_path" => {
            let path: String = arg(p, "path")?;
            engine
                .remove_path(std::path::Path::new(&path))
                .map_err(loc)
                .map(|_| Value::Null)
        }

        "update_preferences" => {
            let u: Preferences = serde_json::from_value(p.clone()).map_err(|e| e.to_string())?;
            if let Some(v) = u.auto_tune {
                engine.config.perf.auto_tune = v;
            }
            if let Some(v) = u.system_tweaks {
                engine.config.perf.system_tweaks = v;
            }
            if let Some(path) = u.presentmon_path {
                engine.config.perf.presentmon_path = (!path.is_empty()).then(|| path.into());
            }
            if let Some(l) = u.language {
                engine.config.language = l;
            }
            if let Some(t) = u.theme {
                engine.config.theme = t;
            }
            engine.save_config().map_err(loc).map(|_| Value::Null)
        }

        "allow_capture" => cl_session::request_capture_permission().map(|_| Value::Null),

        "hardware" => {
            let refresh: Option<bool> = arg(p, "refresh")?;
            let report = engine.hardware(refresh.unwrap_or(false)).clone();
            let main_gpu = report.main_gpu().cloned();
            let mut value = serde_json::to_value(report).map_err(|e| e.to_string())?;
            if let Value::Object(fields) = &mut value {
                fields.insert("main_gpu".into(), json!(main_gpu));
            }
            Ok(value)
        }

        "reload" => {
            *engine = Engine::load(paths.clone()).map_err(loc)?;
            Ok(Value::Null)
        }

        other => Err(format!("unknown method '{other}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_settings_and_errors() {
        let dir = tempfile::tempdir().unwrap();
        let roms = dir.path().join("roms/gba");
        std::fs::create_dir_all(&roms).unwrap();
        std::fs::write(roms.join("Game.gba"), b"").unwrap();
        let api = Api::open(Some(dir.path().join("data")));
        let none = Value::Null;

        api.call(
            "update_preferences",
            &json!({ "language": "fr", "systemTweaks": false }),
        )
        .unwrap();
        let kind = api
            .call("add_path", &json!({ "path": dir.path().join("roms") }))
            .unwrap();
        assert_eq!(kind["kind"], "games");

        let lib = api.call("library", &none).unwrap();
        assert_eq!(lib["games"].as_array().unwrap().len(), 1);
        let id = lib["games"][0]["id"].as_str().unwrap().to_owned();

        let info = api.call("game_info", &json!({ "id": id })).unwrap();
        assert_eq!(info["ready"], false);
        assert_eq!(info["code"], "emulator_missing");
        assert_eq!(info["emulatorId"], "mgba");
        assert!(info["error"].as_str().unwrap().contains("introuvable"));

        let s = api.call("settings_overview", &none).unwrap();
        assert_eq!(s["config"]["perf"]["system_tweaks"], false);
        assert_eq!(s["systems"].as_array().unwrap().len(), SystemId::ALL.len());
        assert!(s["emulators"].as_array().unwrap().len() >= 8);
        let retroarch = s["emulators"]
            .as_array()
            .unwrap()
            .iter()
            .find(|emulator| emulator["id"] == "retroarch")
            .unwrap();
        assert_eq!(retroarch["automatic_install"], false);
        assert!(retroarch["download_page"]
            .as_str()
            .unwrap()
            .starts_with("https://"));
        assert!(api
            .call("toggle_favorite", &json!({ "id": id }))
            .unwrap()
            .as_bool()
            .unwrap());

        assert!(api
            .call("nope", &none)
            .unwrap_err()
            .contains("unknown method"));
        assert!(api.call("play", &json!({ "id": id })).is_err());
        // The session flag is released after a failed launch.
        assert!(api
            .call("play", &json!({ "id": id }))
            .unwrap_err()
            .contains("introuvable"));
    }
}
