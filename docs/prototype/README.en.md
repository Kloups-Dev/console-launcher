# Console Launcher — prototype v0.3

*[Version française](README.fr.md)*

An emulation launcher for Windows and Linux. Pick a console, pick a game, and it starts directly, without going through the emulator's menus. The launcher configures the emulators for the machine's power, measures smoothness while people play, and adjusts its settings until it finds the best balance. The interface is made of QML themes that anyone can edit; two templates are provided.

> This branch is a **prototype for testing**. It does not replace anything on `main`.

The interface is written in C++ / Qt Quick (Qt 6.8 LTS). The engine is written in Rust and linked into the application as a library. An earlier interface written with Tauri remains on the `feat/rust-tauri-prototype` branch; the comparison that led to Qt Quick is in [docs/comparison](../comparison/COMPARISON.en.md).

## What v0.3 does

| Feature | Status |
|---|---|
| Interface driven by controller, keyboard or mouse | ✅ Tested on Linux (virtual display), not yet on Windows |
| Two template themes ("Horizon", "Grid") in QML | ✅ |
| Library: `roms/<console>/` folders, covers, favourites, multi-disc games (.m3u) | ✅ |
| 15 consoles recognised, 8 emulators recognised (RetroArch, Dolphin, PCSX2, mGBA, DuckStation, PPSSPP, melonDS, Cemu), any other emulator can be added | ✅ Generated commands and settings are tested, **not yet tried with the real emulators** |
| Emulators found by themselves, including RetroBat's `emulators` folder; emulator chosen per console | ✅ |
| Full settings screen, usable with a controller: consoles, emulators, folders, smoothness check | ✅ Tested on Linux (virtual display) |
| Hardware detection (CPU, RAM, GPU through Vulkan) and a "simulation" CPU test | ✅ |
| Estimated starting settings, then real measurement and automatic per-game adjustment | ✅ Logic tested; **thresholds still to calibrate** on real machines |
| Real-condition measurement, tool shipped with the launcher: PresentMon (Windows), MangoHud (Linux) | ⚠️ Integrated, not yet checked on a real game session |
| Emulator updates: auto / notify / off, pinned version, rollback | ✅ For PCSX2 (GitHub). Dolphin and RetroArch: manual install for now |
| Settings changed by hand in an emulator are left alone, with a button to give them back to the launcher | ✅ Original files backed up before the first change ([EMULATORS.en.md](EMULATORS.en.md)) |
| System optimisations for every emulator (priority, high-performance GPU on Windows) | ✅ Not yet checked on Windows |

## Download a test build

Every push to GitHub builds ready-to-run packages. On the repository's **Actions** tab, open the latest **CI** run of the branch, then download the artifact at the bottom of the page (a GitHub account is required):

- `console-launcher-qt-windows-x64`: unzip, then run `ConsoleLauncher\console-launcher-qt.exe`. Everything needed is in the folder, including PresentMon for the smoothness check.
- `console-launcher-qt-linux-x64`: unzip, make `ConsoleLauncher-x86_64.AppImage` executable, then run it. MangoHud is included.

**Windows, first smoothness check**: PresentMon needs a Windows permission. In **Settings › Smoothness check**, choose **Allow the measurement** (an administrator prompt opens), then sign out of Windows and back in. Without it, games still launch with the settings estimated for the machine.

## Building

See [qt-prototype/README.en.md](../../qt-prototype/README.en.md). In short: Rust, CMake, Qt 6.8 and SDL2, then:

```bash
cmake -S qt-prototype -B build-qt -DCMAKE_BUILD_TYPE=Release
cmake --build build-qt
./build-qt/console-launcher-qt
```

The `clctl` command-line tool drives the same engine without the interface:

```bash
cargo run -p cl-cli -- probe               # hardware + CPU test
cargo run -p cl-cli -- add-roms D:/Games    # game folder
cargo run -p cl-cli -- scan                 # consoles and games found
cargo run -p cl-cli -- plan "game name"     # what will be launched, with which settings
cargo run -p cl-cli -- play "game name"     # play, measure, adjust
cargo run -p cl-cli -- analyse capture.csv  # analyse a MangoHud / PresentMon capture
cargo run -p cl-cli -- emulators            # emulators detected
cargo run -p cl-cli -- update pcsx2         # install / update PCSX2 (official release)
```

Game layout (the same as RetroBat, Batocera and EmulationStation, so an existing collection works as it is):

```
Games/
  snes/  My game.sfc
  snes/media/covers/My game.png     (cover, optional)
  ps2/   Other game.iso
  gc/    One more.rvz
```

Settings: `config.toml` in the data folder (`%APPDATA%\ConsoleLauncher\console-launcher\data` on Windows, `~/.local/share/console-launcher` on Linux, or the folder given by the `CONSOLE_LAUNCHER_HOME` variable). They are edited from the settings screen; a commented example is in [config.example.toml](config.example.toml).

### Controls

| Controller | Keyboard | Action |
|---|---|---|
| D-pad / left stick | Arrows | Move |
| A (bottom) | Enter | Confirm / play |
| B (right) | Esc | Back |
| Y (top) | F | Favourite |
| Start / Select | Tab | Settings |
| LB / RB | Page Up / Down | Jump (10 games or one console) |
| — | F11 | Fullscreen |

Every controller in the SDL database works: Xbox, PlayStation, Switch Pro and most generic pads.

## Architecture

```
qt-prototype/          C++ / Qt Quick application
  src/                 window, controller (SDL2), bridge to the engine
  ui/                  built-in QML components (settings panel)
  themes/              QML themes, copied next to the executable
crates/
  cl-core              Shared model, config.toml, non-destructive INI editor, change detection
  cl-hwprobe           Hardware detection + CPU test (simulation)
  cl-library           Game scan, covers, favourites
  cl-adapters          Emulator adapters and profiles (detection, settings, command)
    profiles/          Bundled emulator profiles (mGBA, DuckStation, PPSSPP, melonDS, Cemu)
  cl-perf              Estimate, capture analysis, per-game adjustment
  cl-session           Launch, frame capture, system optimisations, wait for exit
  cl-updater           Managed installs: updates, pinning, rollback
  cl-engine            Ties everything together
  cl-api               One JSON entry point over the engine
  cl-ffi               C library loaded by the application (console_launcher_engine)
  cl-cli               clctl
```

The interface knows no emulator: it asks for consoles and games, then calls `play`. The engine methods are listed in [ENGINE_API.en.md](ENGINE_API.en.md). Adding an emulator takes a profile file (TOML) or, for special cases, an adapter in `cl-adapters`; themes do not change. See [EMULATORS.en.md](EMULATORS.en.md).

## What is not provided, and never will be

No ROMs, BIOS, firmware or keys. Everyone uses files dumped from their own hardware. Emulators are downloaded from their official sources and never redistributed with the launcher. Only the two measurement tools, PresentMon and MangoHud (MIT), are shipped with the packages.

## Licence

Still to be chosen. One important point: a licence that **forbids** commercial use (CC BY-NC, PolyForm Noncommercial…) is not "open source" by the OSI definition. It is also incompatible with GPL code, should emulator code ever be included. The proposal is therefore **GPL-3.0-or-later**: the code stays free, and anyone distributing a modified version must publish its source. All current dependencies (Qt under LGPL-3.0, MIT, Apache-2.0, MPL-2.0, Zlib…) are compatible with it.

## Proposed next steps

1. Tests on real Windows and Linux machines with the real emulators, then threshold calibration ([PERFORMANCE.en.md](PERFORMANCE.en.md)).
2. Settings tuning for melonDS (1.x format) and Cemu (graphic packs).
3. Update sources for Dolphin and RetroArch, and installation of missing RetroArch cores.
4. Manual quality choice per game and theme options in the settings screen.
5. Automatic cover download (ScreenScraper, with the user's own account).
6. Windows installer.

Other documents: [EMULATORS.en.md](EMULATORS.en.md) · [THEMES.en.md](THEMES.en.md) · [ENGINE_API.en.md](ENGINE_API.en.md) · [PERFORMANCE.en.md](PERFORMANCE.en.md) · [DEPENDENCIES.en.md](DEPENDENCIES.en.md) · [CHANGELOG](../../CHANGELOG.md)
