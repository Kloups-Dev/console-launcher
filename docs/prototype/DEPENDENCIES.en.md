# Dependencies

*[Version française](DEPENDENCIES.fr.md)*

Everything is open source. Nothing is proprietary or paid. Versions taken from `Cargo.lock` at v0.1.

## Rust libraries (compiled into the software)

| Library | Version | Licence | Purpose |
|---|---|---|---|
| [tauri](https://tauri.app) | 2.12 | MIT / Apache-2.0 | Window and bridge between the web interface and the engine |
| [gilrs](https://gitlab.com/gilrs-project/gilrs) | 0.11 | MIT / Apache-2.0 | Controllers (SDL database, XInput on Windows, evdev on Linux) |
| [sysinfo](https://github.com/GuillaumeGomez/sysinfo) | 0.37 | MIT | Processor, memory, OS |
| [ash](https://github.com/ash-rs/ash) | 0.38 | MIT / Apache-2.0 | Graphics cards through Vulkan (loaded at run time, optional) |
| [ureq](https://github.com/algesten/ureq) | 3.5 | MIT / Apache-2.0 | Update checks and downloads |
| [zip](https://github.com/zip-rs/zip2) | 4.6 | MIT | Emulator .zip archives |
| [sevenz-rust2](https://github.com/hasenbanck/sevenz-rust2) | 0.18 | Apache-2.0 | Emulator .7z archives |
| [serde](https://serde.rs), serde_json, [toml](https://github.com/toml-rs/toml) | 1.x / 0.8 | MIT / Apache-2.0 | Settings and profile files |
| [walkdir](https://github.com/BurntSushi/walkdir) | 2.5 | MIT / Unlicense | Game folder scanning |
| [directories](https://github.com/dirs-dev/directories-rs) | 6 | MIT / Apache-2.0 | Standard per-user folders |
| [clap](https://github.com/clap-rs/clap) | 4.6 | MIT / Apache-2.0 | `clctl` command line |
| [tauri-plugin-dialog](https://github.com/tauri-apps/plugins-workspace) | 2 | MIT / Apache-2.0 | Native file and folder picker |
| thiserror, anyhow, log, env_logger | — | MIT / Apache-2.0 | Errors and logs |
| tempfile *(tests only)* | 3 | MIT / Apache-2.0 | Temporary folders in tests |

The full tree has about 500 packages. Their licences are MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0 or MPL-2.0, all compatible with GPL-3.0. To list them: `cargo metadata --format-version 1`, or `cargo install cargo-about` to generate a credits file.

## To build

| System | Install |
|---|---|
| All | Stable [Rust](https://rustup.rs) (tested with 1.97) |
| Windows 10 / 11 | Microsoft C++ Build Tools ("Desktop development with C++"). WebView2 is already present on up-to-date Windows 10/11 |
| Debian / Ubuntu | `sudo apt install libwebkit2gtk-4.1-dev libudev-dev librsvg2-dev libssl-dev pkg-config build-essential` |
| Fedora | `sudo dnf install webkit2gtk4.1-devel systemd-devel librsvg2-devel openssl-devel` |
| Arch | `sudo pacman -S webkit2gtk-4.1 base-devel librsvg openssl` |

To create the installers: `cargo install tauri-cli --version "^2" --locked`, then `cargo tauri build` from `app/src-tauri`.

## At run time

- **Windows**: WebView2 (included in Windows 10/11).
- **Linux**: WebKitGTK 4.1 (installed with most desktops).
- **Optional**: a Vulkan driver for graphics card detection. Without it the launcher still starts and picks cautious settings.

## Measurement tools (optional, not shipped)

| Tool | Licence | System | Use |
|---|---|---|---|
| [MangoHud](https://github.com/flightlessmango/MangoHud) | MIT | Linux | Records the time of every frame during play. Install it with the package manager |
| [PresentMon](https://github.com/GameTechDev/PresentMon) | MIT | Windows | Same role. Choose it in Settings › Smoothness check (or `perf.presentmon_path`). Needs administrator rights or the "Performance Log Users" group |

Without these tools, games launch normally with the estimated settings. Only automatic measurement is paused, and the launcher says so.

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
