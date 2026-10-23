# Console Launcher — Qt Quick application

*[Version française](README.fr.md)*

The launcher's interface: consoles from left to right, game list, game details, direct launch, back to the menu, and a settings screen usable with a controller.

| Item | Choice |
|---|---|
| Language | C++17 |
| Interface | Qt Quick (QML), Qt 6.8 LTS; GPU rendering through Direct3D 11 on Windows, OpenGL/Vulkan on Linux |
| Themes | `themes/<name>/Main.qml` folders read at start-up, editable without rebuilding ([THEMES.en.md](../docs/prototype/THEMES.en.md)) |
| Built-in components | `import ConsoleLauncher`: settings panel (`ui/SettingsPanel.qml`) |
| Controllers | SDL2 (GameController database: Xbox, PlayStation, Switch, generic pads) |
| Engine | Rust library `console_launcher_engine` (crate `cl-ffi`), loaded by the application; calls run on worker threads ([ENGINE_API.en.md](../docs/prototype/ENGINE_API.en.md)) |
| During a game | Window hidden (Qt Quick stops drawing) and SDL fully shut down |

## Releases

A pre-release is published automatically, with both packages and this version's section of `CHANGELOG.md` as notes:

- on each push to `feat/qt-quick-comparison` whose version (`version` in `Cargo.toml`) has no release yet; the tag `v<version>-qt` is created on that commit. Publishing a new test build therefore only takes a version bump with its changelog entry;
- or from a tag starting with `v`, pushed with git or created from the GitHub page (Releases › Draft a new release).

Pre-releases are attached to their commit and never marked as the latest release.

## Building

Requirements: Rust (stable), CMake 3.21+, a C++17 compiler, Qt 6.8 or newer (Core, Gui, Qml, Quick, Quick Dialogs), SDL2. Details per system: [DEPENDENCIES.en.md](../docs/prototype/DEPENDENCIES.en.md).

```bash
cmake -S qt-prototype -B build-qt -DCMAKE_BUILD_TYPE=Release
cmake --build build-qt
./build-qt/console-launcher-qt
```

CMake runs `cargo build -p cl-ffi -p cl-cli` itself, then copies the engine library and the themes next to the executable. `CL_ENGINE_PREBUILT_DIR` points CMake at an already built engine instead.

**Windows** (Visual Studio 2022 developer prompt, Qt kit `msvc2022_64`, SDL2 "devel VC" archive):

```powershell
cmake -S qt-prototype -B build-qt -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH=C:\Qt\6.8.3\msvc2022_64 -DSDL2_DIR=C:\SDL2-2.32.10\cmake
cmake --build build-qt
windeployqt --release --qmldir qt-prototype/themes --qmldir qt-prototype/ui build-qt/console-launcher-qt.exe
copy C:\SDL2-2.32.10\lib\x64\SDL2.dll build-qt\
```

## Measurement tools

The packages built by the CI carry the smoothness measurement tools in `tools/` next to the executable: `tools/presentmon/PresentMon-*.exe` on Windows, `tools/mangohud/libMangoHud*.so` on Linux. For a local build, place them there by hand, or set the `CONSOLE_LAUNCHER_TOOLS` variable to their folder. A copy installed on the machine takes precedence.

## Variables

| Variable | Use |
|---|---|
| `CONSOLE_LAUNCHER_HOME` | Data folder (settings, profiles, captures) |
| `CONSOLE_LAUNCHER_QT_THEMES` | Themes folder, to work on a theme elsewhere |
| `CONSOLE_LAUNCHER_TOOLS` | Folder of the measurement tools |
| `RUST_LOG` | Engine log level (`info` by default) |

## Logs and start-up check

The interface writes `console-launcher.log` and the engine `console-launcher-engine.log` next to the executable (or in the temporary folder when that folder is read-only). They are rewritten at each start. When a theme cannot be loaded, a window lists the errors.

`console-launcher-qt --smoke-test` loads the theme, asks for the library, then exits: code 0 when the theme loaded. The CI runs it on both packages.
