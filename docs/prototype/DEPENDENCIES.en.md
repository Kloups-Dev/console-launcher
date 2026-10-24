# Dependencies

*[Version française](DEPENDENCIES.fr.md)*

Everything is open source. Nothing is proprietary or paid. Versions as of v0.3.

## Interface (C++)

| Library | Version | Licence | Purpose |
|---|---|---|---|
| [Qt](https://www.qt.io) (Core, Gui, Qml, Quick, Quick Dialogs) | 6.8 LTS | LGPL-3.0 (dynamically linked, DLLs shipped next to the launcher) | Window, QML themes, GPU rendering, file pickers |
| [SDL2](https://www.libsdl.org) | 2.32 | Zlib | Controllers (SDL GameController database) |

## Engine: Rust libraries (compiled into `console_launcher_engine`)

| Library | Version | Licence | Purpose |
|---|---|---|---|
| [sysinfo](https://github.com/GuillaumeGomez/sysinfo) | 0.37 | MIT | Processor, memory, OS |
| [ash](https://github.com/ash-rs/ash) | 0.38 | MIT / Apache-2.0 | Graphics cards through Vulkan (loaded at run time, optional) |
| [ureq](https://github.com/algesten/ureq) | 3.5 | MIT / Apache-2.0 | Update checks and downloads |
| [zip](https://github.com/zip-rs/zip2) | 4.6 | MIT | Emulator .zip archives |
| [sevenz-rust2](https://github.com/hasenbanck/sevenz-rust2) | 0.18 | Apache-2.0 | Emulator .7z archives |
| [serde](https://serde.rs), serde_json, [toml](https://github.com/toml-rs/toml) | 1.x / 0.8 | MIT / Apache-2.0 | Settings and profile files |
| [walkdir](https://github.com/BurntSushi/walkdir) | 2.5 | MIT / Unlicense | Game folder scanning |
| [directories](https://github.com/dirs-dev/directories-rs) | 6 | MIT / Apache-2.0 | Standard per-user folders |
| [clap](https://github.com/clap-rs/clap) | 4.6 | MIT / Apache-2.0 | `clctl` command line |
| thiserror, anyhow, log, env_logger | — | MIT / Apache-2.0 | Errors and logs |
| tempfile *(tests only)* | 3 | MIT / Apache-2.0 | Temporary folders in tests |

The full tree has about 500 packages. Their licences are MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0 or MPL-2.0, all compatible with GPL-3.0. To list them: `cargo metadata --format-version 1`, or `cargo install cargo-about` to generate a credits file.

## To build

| System | Install |
|---|---|
| All | Stable [Rust](https://rustup.rs) (tested with 1.97), [CMake](https://cmake.org) 3.21+, Qt 6.8 (Qt online installer or `aqtinstall`), SDL2 |
| Windows 10 / 11 | Visual Studio 2022 or its Build Tools ("Desktop development with C++"), Qt kit `msvc2022_64`, SDL2 "devel VC" archive from [libsdl.org](https://github.com/libsdl-org/SDL/releases) |
| Debian / Ubuntu | `sudo apt install build-essential cmake libsdl2-dev libgl1-mesa-dev`, Qt 6.8 from the Qt installer |
| Fedora | `sudo dnf install gcc-c++ cmake SDL2-devel mesa-libGL-devel qt6-qtdeclarative-devel` (if the distribution ships Qt 6.8 or newer) |
| Arch | `sudo pacman -S base-devel cmake sdl2 qt6-declarative` |

Commands: [qt-prototype/README.en.md](../../qt-prototype/README.en.md). The packages are built by the CI (`.github/workflows/ci.yml`): Windows folder with `windeployqt`, Linux AppImage with `linuxdeploy`.

## At run time

- **Windows**: nothing to install; the Qt and SDL2 DLLs are in the folder.
- **Linux**: the AppImage carries Qt; an X11 or Wayland desktop is enough.
- **Optional**: a Vulkan driver for graphics card detection. Without it the launcher still starts and picks cautious settings.

## Measurement tools (shipped with the packages)

| Tool | Version | Licence | System | Use |
|---|---|---|---|---|
| [PresentMon](https://github.com/GameTechDev/PresentMon) | 2.6.0 | MIT | Windows | Records the time of every frame during play. In `tools/presentmon/`. Needs administrator rights or the "Performance Log Users" group: **Settings › Smoothness check › Allow the measurement** |
| [MangoHud](https://github.com/flightlessmango/MangoHud) | 0.8.4 | MIT | Linux | Same role. In `tools/mangohud/` inside the AppImage. A copy installed on the machine takes precedence |

Their licence files are next to them. Without measurement, games launch normally with the estimated settings, and the launcher says so.

## Emulators (not shipped, downloaded from their official sources)

| Emulator | Licence | Consoles |
|---|---|---|
| [RetroArch](https://www.retroarch.com) | GPL-3.0 | NES, SNES, N64, Game Boy, GBA, DS, Master System, Mega Drive, PlayStation, PSP |
| [Dolphin](https://dolphin-emu.org) | GPL-2.0-or-later | GameCube, Wii |
| [PCSX2](https://pcsx2.net) | GPL-3.0-or-later | PlayStation 2 |
| [mGBA](https://mgba.io) | MPL-2.0 | Game Boy, Game Boy Color, GBA |
| [DuckStation](https://www.duckstation.org) | CC-BY-NC-ND-4.0 (*) | PlayStation |
| [PPSSPP](https://www.ppsspp.org) | GPL-2.0-or-later | PSP |
| [melonDS](https://melonds.kuribo64.net) | GPL-3.0-or-later | Nintendo DS |
| [Cemu](https://cemu.info) | MPL-2.0 | Wii U |

Default RetroArch cores: Mesen (GPL-3.0), Snes9x\*, Mupen64Plus-Next (GPL-2.0), Gambatte (GPL-2.0), mGBA (MPL-2.0), melonDS DS (GPL-3.0), Genesis Plus GX\*, Beetle PSX HW (GPL-2.0), PPSSPP (GPL-2.0).

(\*) Recent DuckStation versions forbid commercial use and redistribution of modified copies. The launcher only starts the copy the user installed.

\* Snes9x and Genesis Plus GX have licences that **forbid commercial use**. That is not an issue as long as the launcher does not redistribute them, which it does not: users install them from RetroArch.
