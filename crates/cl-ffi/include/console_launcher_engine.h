/*
 * Console Launcher engine — C interface.
 *
 * The engine (Rust) is a shared library: console_launcher_engine.dll on
 * Windows, libconsole_launcher_engine.so on Linux. Requests and answers are
 * UTF-8 JSON. Every answer is {"ok": true, "result": ...} or
 * {"ok": false, "error": "..."}. Methods: docs/prototype/ENGINE_API.en.md.
 */
#ifndef CONSOLE_LAUNCHER_ENGINE_H
#define CONSOLE_LAUNCHER_ENGINE_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct ClEngine ClEngine;

/* Opens the engine. data_dir: NULL for the standard per-user folder. */
ClEngine *cl_engine_open(const char *data_dir);

/* Calls a method; params_json may be NULL. Thread-safe. Calls are
 * serialised, except "play", which blocks until the emulator closes and
 * runs alongside the others. Free the answer with cl_string_free. */
char *cl_engine_call(ClEngine *engine, const char *method, const char *params_json);

void cl_string_free(char *s);

/* Closes the engine. No call may be running. */
void cl_engine_close(ClEngine *engine);

#ifdef __cplusplus
}
#endif

#endif
