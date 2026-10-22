# C++ / Qt Quick prototype

*[Version française](README.fr.md)*

Third interface track, written to compare technologies. It follows the same path as the other two prototypes: consoles from left to right, game list, game details, direct launch, back to the menu.

| Item | Choice |
|---|---|
| Language | C++17 |
| Interface | Qt Quick (QML), GPU rendering through Direct3D 11 on Windows, OpenGL/Vulkan on Linux |
| Themes | `themes/<name>/Main.qml` folders read at start-up. They can be edited without rebuilding |
| Controllers | SDL2 (GameController database: Xbox, PlayStation, Switch, generic pads) |
| Engine | `clctl`, the same Rust engine as the Tauri prototype (library, settings, measurement, updates) |
| During a game | Window hidden (Qt Quick stops drawing) and SDL fully shut down |

The engine is not rewritten: the prototype calls `clctl` for each request (scan, info, launch, favourite). This also shows that a Qt interface can sit on top of the Rust engine.

## Building

Requirements: CMake 3.21+, a C++17 compiler, Qt 6.4 or newer (Core, Gui, Qml, Quick), SDL2.

```bash
cmake -S qt-prototype -B build-qt -DCMAKE_BUILD_TYPE=Release
cmake --build build-qt
cargo build --release -p cl-cli          # engine
cp target/release/clctl build-qt/        # clctl.exe on Windows
./build-qt/console-launcher-qt
```

On Windows with Qt installed (Qt online installer, MinGW or MSVC kit): same commands, then `windeployqt --qmldir qt-prototype/themes build-qt/console-launcher-qt.exe` to copy the Qt DLLs next to the executable.

## Themes

A theme is a folder holding `Main.qml`, whose root is a `Window`. It receives a single object, `Launcher`:

| Member | Purpose |
|---|---|
| `systems`, `games` | Library (same data as `clctl --json scan`) |
| `nav(action)` (signal) | `up`, `down`, `left`, `right`, `confirm`, `back`, `menu`, `favorite`, `pageLeft`, `pageRight` |
| `requestInfo(id)` → `gameInfoReady(id, info)` | Emulator, quality rung, notes |
| `play(id)` → `sessionEnded(report)` | Starts the game; `running` is `true` during play |
| `toggleFavorite(id)` → `favoriteChanged(id, on)` | Favourites |
| `addRomDir(path)`, `setTheme(name)`, `themes`, `theme` | Settings |
| `fileUrl(path)`, `asset("img/bg.png")` | Library and theme images |
| `toggleFullscreen()`, `quit()`, `padName`, `lastError` | — |

To work on a theme from another folder: `CONSOLE_LAUNCHER_QT_THEMES` variable. Two templates are provided, `horizon` and `grid`.
