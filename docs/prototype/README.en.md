# Console Launcher — prototype v0.1

*[Version française](README.fr.md)*

An emulation launcher for Windows and Linux. Pick a console, pick a game, and it starts directly, without going through the emulator's menus. The launcher configures the emulators for the machine's power, measures smoothness while people play, and adjusts its settings until it finds the best balance. The interface is a fully editable HTML/CSS/JS theme, and two templates are provided as starting points.

> This branch is a **prototype for testing**. It does not replace anything on `main`.

## What v0.1 does

| Feature | Status |
|---|---|
| Interface driven by controller, keyboard or mouse | ✅ Tested on Linux (virtual display), not yet on Windows |
| Two template themes ("Horizon", "Grid") and theme API | ✅ |
| Library: `roms/<console>/` folders, covers, favourites, multi-disc games (.m3u) | ✅ |
| 14 consoles recognised, 3 emulators integrated (RetroArch, Dolphin, PCSX2) | ✅ Generated commands are tested, **not yet tried with the real emulators** |
| Hardware detection (CPU, RAM, GPU through Vulkan) and a "simulation" CPU test | ✅ |
| Estimated starting settings, then real measurement and automatic per-game adjustment | ✅ Logic tested; **thresholds still to calibrate** on real machines |
| Real-condition measurement: MangoHud (Linux), PresentMon (Windows) | ⚠️ Integrated, not yet checked on a real game session |
| Emulator updates: auto / notify / off, pinned version, rollback | ✅ For PCSX2 (GitHub). Dolphin and RetroArch: manual install for now |
| Users' own settings are respected | ✅ Files of an emulator installed by the user are never modified |

## Quick start

Requirements: stable [Rust](https://rustup.rs). On Linux, Tauri's system libraries are also needed (see [DEPENDENCIES.en.md](DEPENDENCIES.en.md)).

```bash
# Interface
cargo run -p console-launcher

# Command-line tool, to test the engine without the interface
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

Settings: `config.toml` in the data folder (`%APPDATA%\ConsoleLauncher\console-launcher\data` on Windows, `~/.local/share/console-launcher` on Linux, or the folder given by the `CONSOLE_LAUNCHER_HOME` variable). A commented example is in [config.example.toml](config.example.toml).

### Controls

| Controller | Keyboard | Action |
|---|---|---|
| D-pad / left stick | Arrows | Move |
| A (bottom) | Enter | Confirm / play |
| B (right) | Esc | Back |
| Y (top) | F | Favourite |
| Start / Select | Tab | Settings menu |
| LT / RT | Page Up / Down | Jump (10 games or one console) |

Every controller in the SDL database works: Xbox, PlayStation, Switch Pro and most generic pads.

## Architecture

```
app/                  Tauri application: window, controller, theme loading
  ui/sdk.js           `Launcher` API used by themes
themes/               Template themes (copied to the data folder on first start)
crates/
  cl-core             Shared model, config.toml, non-destructive INI editor
  cl-hwprobe          Hardware detection + CPU test (simulation)
  cl-library          Game scan, covers, favourites
  cl-adapters         One adapter per emulator (detection, settings, command)
  cl-perf             Estimate, capture analysis, per-game adjustment
  cl-session          Launch, frame capture, wait for exit
  cl-updater          Managed installs: updates, pinning, rollback
  cl-engine           Ties everything together for the interface and clctl
  cl-cli              clctl
```

The interface knows no emulator: it asks for consoles and games, then calls `play`. Adding an emulator only takes a new adapter in `cl-adapters`; themes do not change.

## What is not provided, and never will be

No ROMs, BIOS, firmware or keys. Everyone uses files dumped from their own hardware. Emulators are downloaded from their official sources and never redistributed with the launcher.

## Licence

Still to be chosen. One important point: a licence that **forbids** commercial use (CC BY-NC, PolyForm Noncommercial…) is not "open source" by the OSI definition. It is also incompatible with GPL code, should emulator code ever be included. The proposal is therefore **GPL-3.0-or-later**: the code stays free, and anyone distributing a modified version must publish its source. All current dependencies (MIT, Apache-2.0, MPL-2.0…) are compatible with it.

## Proposed next steps

1. Tests on real Windows and Linux machines with the three emulators, then threshold calibration ([PERFORMANCE.en.md](PERFORMANCE.en.md)).
2. Update sources for Dolphin and RetroArch, and installation of missing RetroArch cores.
3. Full settings screen: emulator per console, manual quality choice, theme options.
4. Automatic cover download (ScreenScraper, with the user's own account).
5. More adapters: DuckStation, PPSSPP, melonDS, Cemu, RPCS3…
6. Installers (NSIS for Windows, AppImage and .deb for Linux) with `cargo tauri build`.

Other documents: [THEMES.en.md](THEMES.en.md) · [PERFORMANCE.en.md](PERFORMANCE.en.md) · [DEPENDENCIES.en.md](DEPENDENCIES.en.md) · [CHANGELOG](../../CHANGELOG.md)
