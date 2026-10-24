# Engine API

*[Version française](ENGINE_API.fr.md)*

The engine (Rust) is a shared library, `console_launcher_engine.dll` on Windows and `libconsole_launcher_engine.so` on Linux, built from the `cl-ffi` crate. The Qt interface loads it directly; any other program can use it through the C header [`crates/cl-ffi/include/console_launcher_engine.h`](../../crates/cl-ffi/include/console_launcher_engine.h).

```c
ClEngine *e = cl_engine_open(NULL);                  /* NULL: standard data folder */
char *answer = cl_engine_call(e, "library", NULL);  /* JSON answer */
cl_string_free(answer);
cl_engine_close(e);
```

- Requests and answers are UTF-8 JSON. Every answer is `{"ok": true, "result": …}` or `{"ok": false, "error": "…"}`.
- `cl_engine_call` is thread-safe. Calls are handled one at a time, except `play`, which blocks until the emulator closes and runs alongside the others.
- Error messages follow the language chosen in the settings (`fr` or `en`).
- An internal error never crashes the caller: it comes back as `{"ok": false}`.

The built-in QML interface uses a private `Launcher.internalRequest(...)` bridge. It is not part of the future theme contract and no external QML is executed.

## Methods

| Method | Parameters | Result |
|---|---|---|
| `version` | — | Engine version |
| `config` | — | Contents of `config.toml` (fast: no disk scan) |
| `library` | — | `{systems, games}` |
| `add_game` | `system`, `path` | Adds an individual game file after checking its extension |
| `remove_game` | `path` | Removes an individually added game without deleting the file |
| `game_info` | `id` | What a launch would do: emulator, quality rung, notes. Writes nothing. When the game cannot start: `{ready: false, code, emulatorId, error}` |
| `refresh_game_metadata` | `id` | Explicitly looks up the Title Screen record and the GameTDB cover for GameCube/Wii, Nintendo DS and Wii U. It caches the record and validated PNG/JPEG, then returns metadata or `null`. Normal library scans never use the network. |
| `play` | `id` | Starts the game, waits for the emulator to close, measures and adjusts. Report: smoothness, next rung, notes |
| `toggle_favorite` | `id` | New state (`true` / `false`) |
| `set_quality` | `id`, `step` | Fixes a rung by hand; the automatic search stops for that game |
| `settings_overview` | — | Everything the settings screen shows: `config`, `emulators` (state, path, settings kept by the user), `systems` (emulator choice per console), `capture` (measurement tool, permission), `profileErrors`, `dataDir` |
| `set_system_emulator` | `system`, `id` (null: automatic) | — |
| `set_emulator_path` | `id`, `path` (null: automatic detection) | — |
| `add_custom_emulator` | `path`, `name`, `systems`, `args` | Emulator id. A known executable gives the known emulator |
| `update_custom_emulator` | `id`, `emulator` (`{name, path, args, systems}`) | — |
| `remove_custom_emulator` | `id` | — |
| `reset_emulator_tuning` | `id` | Gives the settings changed by hand back to the automatic optimisation |
| `open_emulator` | `id` | Opens the emulator without a game |
| `install_emulator` | `id` | Downloads, verifies and activates an available managed version (Dolphin 2609a on Windows) |
| `add_path` | `path` | `{kind: "games" \| "emulators" \| "emulator" \| "already_added"}` |
| `remove_path` | `path` | — |
| `update_preferences` | any of `autoTune`, `systemTweaks`, `presentmonPath` (`""` clears it), `language`, `theme` | — |
| `allow_capture` | — | Windows: adds the user to "Performance Log Users" after an administrator prompt (effective at the next sign-in) |
| `hardware` | `refresh` | Detected machine; `true` runs the test again |
| `reload` | — | Reads `config.toml` and the profile files again |
