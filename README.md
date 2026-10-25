# Console Launcher

[![CI](https://github.com/Kloups-Dev/console-launcher/actions/workflows/ci.yml/badge.svg)](https://github.com/Kloups-Dev/console-launcher/actions/workflows/ci.yml)
[![Latest pre-release](https://img.shields.io/github/v/release/Kloups-Dev/console-launcher?include_prereleases&label=release)](https://github.com/Kloups-Dev/console-launcher/releases)
[![Status: prototype](https://img.shields.io/badge/status-prototype-orange)](https://github.com/Kloups-Dev/console-launcher)

**Console Launcher** is a desktop front-end for emulation on Windows and
Linux. It gives games, consoles, emulators and performance settings one
consistent interface, while continuing to use the emulators that already
support each system.

The intended path is simple:

**add your games → choose a console → select a game → play → return to the
library**

Console Launcher does not emulate consoles itself. It discovers, configures
and launches existing emulators; it does not include games, BIOS files,
firmware, encryption keys or copyrighted content.

> **Project status:** Console Launcher 0.3.2 is a working prototype and
> pre-release. The application can be downloaded from
> [GitHub Releases](https://github.com/Kloups-Dev/console-launcher/releases).
> It is not yet a finished 1.0 product, and real hardware, controller, audio
> and emulator coverage is still being expanded.

## What works today

The current Qt Quick application provides:

- a conventional desktop library with console catalogue, search, favourites,
  game details and direct launch;
- support for 15 systems: NES, SNES, Nintendo 64, Game Boy, Game Boy Color,
  Game Boy Advance, Nintendo DS, Master System, Mega Drive, PlayStation,
  PlayStation 2, PSP, GameCube, Wii and Wii U;
- detection of RetroArch, Dolphin, PCSX2, mGBA, DuckStation, PPSSPP, melonDS
  and Cemu, plus user-defined emulators;
- game and emulator folder management, including individual game import;
- local game identity, metadata and artwork caches;
- regional GameTDB artwork for GameCube/Wii, Nintendo DS and Wii U where the
  game identity is available;
- conservative hardware detection and emulator profiles;
- Dolphin installation on Windows from the official release with archive
  checksum verification;
- user-protected emulator settings: automatic changes are backed up and
  existing manual choices are not silently overwritten;
- local logs and optional frame-time measurement through PresentMon on Windows
  or MangoHud on Linux.

The launcher keeps its data local by default. It does not upload games,
controller input, logs or performance captures automatically.

## Download and run

Download the latest package from
[Releases](https://github.com/Kloups-Dev/console-launcher/releases).

### Windows

1. Download and extract the Windows package.
2. Open the `ConsoleLauncher` directory.
3. Run `console-launcher-qt.exe`.

### Linux

1. Download `ConsoleLauncher-x86_64.AppImage`.
2. Make it executable:

   ```bash
   chmod +x ConsoleLauncher-x86_64.AppImage
   ```

3. Run it:

   ```bash
   ./ConsoleLauncher-x86_64.AppImage
   ```

The packages do not contain games or BIOS files. Add your own game folders
from the launcher settings.

## Build from source

The repository contains a Rust engine and a Qt Quick/C++ application.

### Requirements

- Rust stable with Rust 1.80 or newer;
- CMake 3.21 or newer;
- a C++17 compiler;
- Qt 6.8 or newer with Core, Gui, Qml, Quick and Quick Dialogs;
- SDL2 development files.

Platform-specific dependency details are documented in
[`qt-prototype/README.en.md`](qt-prototype/README.en.md) and
[`qt-prototype/README.fr.md`](qt-prototype/README.fr.md).

### Rust engine and CLI

```bash
cargo test --workspace --all-targets
cargo run -p cl-cli -- --help
```

The command-line diagnostic tool is called `clctl`. For example:

```bash
cargo run -p cl-cli -- scan
cargo run -p cl-cli -- emulators
```

### Qt application

```bash
cmake -S qt-prototype -B build-qt -DCMAKE_BUILD_TYPE=Release
cmake --build build-qt
./build-qt/console-launcher-qt
```

The Qt build invokes Cargo for the Rust engine unless an already-built engine
is supplied with `CL_ENGINE_PREBUILT_DIR`.

To validate that the packaged interface loads without opening a full session:

```bash
./build-qt/console-launcher-qt --smoke-test
```

## Configuration and local data

The launcher never moves or copies game files. Its own data is stored in:

| Platform | Default data directory |
|---|---|
| Windows | `%APPDATA%\ConsoleLauncher\console-launcher\data` |
| Linux | `~/.local/share/console-launcher` |

Useful environment variables:

| Variable | Purpose |
|---|---|
| `CONSOLE_LAUNCHER_HOME` | Override the launcher data directory |
| `CONSOLE_LAUNCHER_TOOLS` | Override the folder containing PresentMon or MangoHud |
| `RUST_LOG` | Set the Rust engine log level, for example `RUST_LOG=debug` |

The launcher writes local interface and engine logs next to the executable,
or in a temporary directory when that location is not writable.

## Performance and diagnostics

The project follows a stability-first policy:

- an automatic profile is a conservative starting point, not a guarantee;
- a measured session can refine the profile for that game and machine;
- a session that exits with an error is recorded as failed and is not used as
  evidence that a profile is stable;
- frame pacing, FPS and process resources can be measured when the platform
  and capture tool support it.

The current diagnostic path does **not** measure or guarantee emulation speed,
audio correctness, GPU load or temperature. A successful build or automated
test is not proof that a particular game, controller or emulator works
correctly on every machine.

## Architecture

| Component | Responsibility |
|---|---|
| `qt-prototype/` | Qt Quick desktop application, SDL2 controller input and native dialogs |
| `crates/cl-engine` | Application orchestration and launch decisions |
| `crates/cl-library` | Game scanning, identity and metadata |
| `crates/cl-adapters` | Emulator detection, profiles and launch plans |
| `crates/cl-hwprobe` / `crates/cl-perf` | Hardware detection and frame-time analysis |
| `crates/cl-session` | Process launching, capture and session records |
| `crates/cl-updater` | Managed emulator installation and updates |
| `crates/cl-api` / `crates/cl-ffi` | Rust-to-Qt engine boundary |
| `crates/cl-cli` | `clctl`, the command-line diagnostic tool |

The interface presents common concepts. Emulator-specific behavior belongs in
the Rust adapters rather than in QML.

## Project direction

- [`ROADMAP.md`](ROADMAP.md) describes the product vision, priorities and
  criteria for considering future work complete. It is not a list of features
  already available.
- [`CHANGELOG.md`](CHANGELOG.md) records published changes by version.
- [`qt-prototype/README.en.md`](qt-prototype/README.en.md) and
  [`qt-prototype/README.fr.md`](qt-prototype/README.fr.md) contain detailed
  build and packaging notes.

The current priorities are reliable controller mapping, a consistent
diagnostic session, real-world validation with multiple machines and
emulators, and automatic tuning that remains explainable and reversible.

## Scope and legal notice

Console Launcher is an interface and launcher for software installed by the
user. It does not provide or encourage the distribution of ROMs, BIOS files,
firmware, keys or circumvention tools. Users are responsible for complying
with the laws and licences applicable to their games, firmware and emulators.

The project's final licence has not been selected yet.
