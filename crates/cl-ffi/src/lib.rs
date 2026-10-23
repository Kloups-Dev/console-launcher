//! C interface of the engine (see `include/console_launcher_engine.h`).
//!
//! Requests and answers are UTF-8 JSON strings. Every answer is an object:
//! `{"ok": true, "result": …}` or `{"ok": false, "error": "…"}`.
//! A panic inside the engine is turned into an error answer and never
//! crosses into the caller.

use cl_api::Api;
use serde_json::{json, Value};
use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;

/// Opaque handle given to the caller.
pub struct ClEngine {
    api: Api,
}

unsafe fn str_arg(p: *const c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    // SAFETY: the caller passes a NUL-terminated string (see the header).
    Some(unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
}

fn answer(v: Value) -> *mut c_char {
    let text = v.to_string();
    // JSON never contains a raw NUL byte (it is escaped as \u0000).
    CString::new(text)
        .unwrap_or_else(|_| CString::new("{\"ok\":false,\"error\":\"encoding\"}").expect("static"))
        .into_raw()
}

/// Opens the engine. `data_dir` may be NULL (standard per-user folder).
/// Returns NULL only if the engine panicked while starting.
///
/// # Safety
/// `data_dir` is NULL or a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn cl_engine_open(data_dir: *const c_char) -> *mut ClEngine {
    let mut logger =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    // The application may ask for a log file (it has no console on Windows).
    if let Some(path) = std::env::var_os("CONSOLE_LAUNCHER_ENGINE_LOG") {
        if let Ok(file) = std::fs::File::create(path) {
            logger.target(env_logger::Target::Pipe(Box::new(file)));
        }
    }
    let _ = logger.try_init();
    log::info!("engine {} starting", env!("CARGO_PKG_VERSION"));
    let dir = unsafe { str_arg(data_dir) }
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    match catch_unwind(|| Api::open(dir)) {
        Ok(api) => Box::into_raw(Box::new(ClEngine { api })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Calls a method. Thread-safe: may be called from several threads; calls
/// are serialised except `play`, which runs alongside the others.
/// The returned string must be freed with `cl_string_free`.
///
/// # Safety
/// `engine` comes from `cl_engine_open` and is not closed; `method` is a
/// NUL-terminated string; `params_json` is NULL or a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn cl_engine_call(
    engine: *mut ClEngine,
    method: *const c_char,
    params_json: *const c_char,
) -> *mut c_char {
    if engine.is_null() {
        return answer(json!({ "ok": false, "error": "engine not open" }));
    }
    // SAFETY: checked non-null; the caller keeps it alive during the call.
    let engine = unsafe { &*engine };
    let Some(method) = (unsafe { str_arg(method) }) else {
        return answer(json!({ "ok": false, "error": "no method" }));
    };
    let params = match unsafe { str_arg(params_json) } {
        None => Value::Null,
        Some(t) if t.trim().is_empty() => Value::Null,
        Some(t) => match serde_json::from_str(&t) {
            Ok(v) => v,
            Err(e) => return answer(json!({ "ok": false, "error": format!("invalid JSON: {e}") })),
        },
    };
    let result = catch_unwind(AssertUnwindSafe(|| engine.api.call(&method, &params)));
    answer(match result {
        Ok(Ok(v)) => json!({ "ok": true, "result": v }),
        Ok(Err(e)) => json!({ "ok": false, "error": e }),
        Err(_) => {
            log::error!("engine panic in '{method}'");
            json!({ "ok": false, "error": format!("internal error in '{method}'") })
        }
    })
}

/// Frees a string returned by `cl_engine_call`.
///
/// # Safety
/// `s` is NULL or a string returned by `cl_engine_call`, freed only once.
#[no_mangle]
pub unsafe extern "C" fn cl_string_free(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: created by CString::into_raw in `answer`.
        drop(unsafe { CString::from_raw(s) });
    }
}

/// Closes the engine. No call may be running.
///
/// # Safety
/// `engine` is NULL or comes from `cl_engine_open`, closed only once.
#[no_mangle]
pub unsafe extern "C" fn cl_engine_close(engine: *mut ClEngine) {
    if !engine.is_null() {
        // SAFETY: created by Box::into_raw in `cl_engine_open`.
        drop(unsafe { Box::from_raw(engine) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(e: *mut ClEngine, m: &str, p: &str) -> Value {
        let m = CString::new(m).unwrap();
        let p = CString::new(p).unwrap();
        unsafe {
            let r = cl_engine_call(e, m.as_ptr(), p.as_ptr());
            let v = serde_json::from_str(CStr::from_ptr(r).to_str().unwrap()).unwrap();
            cl_string_free(r);
            v
        }
    }

    #[test]
    fn round_trip_through_the_c_interface() {
        let dir = tempfile_dir();
        let d = CString::new(dir.to_str().unwrap()).unwrap();
        let e = unsafe { cl_engine_open(d.as_ptr()) };
        assert!(!e.is_null());
        let v = call(e, "library", "");
        assert_eq!(v["ok"], true);
        assert!(v["result"]["games"].as_array().unwrap().is_empty());
        let v = call(e, "game_info", "{not json");
        assert_eq!(v["ok"], false);
        let v = call(e, "nope", "{}");
        assert!(v["error"].as_str().unwrap().contains("unknown method"));
        unsafe { cl_engine_close(e) };
        let _ = std::fs::remove_dir_all(dir);
    }

    fn tempfile_dir() -> PathBuf {
        let d = std::env::temp_dir().join(format!("cl-ffi-test-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }
}
