# Emulators

*[Version française](EMULATORS.fr.md)*

How the launcher finds emulators, which ones it optimises, how it respects settings changed by hand, and how to teach it a new emulator.

## Emulators recognised

| Emulator | Consoles | Automatic optimisation | How the settings are applied |
|---|---|---|---|
| RetroArch | NES, SNES, N64, Game Boy, GBA, DS, Master System, Mega Drive, PlayStation, PSP | Latency (threaded video, run-ahead), graphics back-end | Session override (`--appendconfig`), files untouched |
| Dolphin | GameCube, Wii | Internal resolution, shader compilation, back-end | Session override (`--config`), files untouched |
| PCSX2 | PlayStation 2 | Upscaling, back-end | `PCSX2.ini` edited |
| mGBA | Game Boy, Game Boy Color, GBA | Frame skipping (very slow machines only) | Session override (`-C`), files untouched |
| DuckStation | PlayStation | Internal resolution, back-end | `settings.ini` edited |
| PPSSPP | PSP | Rendering resolution | `ppsspp.ini` edited |
| melonDS | Nintendo DS | System optimisations only | — |
| Cemu | Wii U | System optimisations only | — |

The launcher never ships emulators, BIOS, firmware or keys. Emulators come from their official sources or are already installed by the user.

## Finding emulators

Without any setting, the launcher looks, in this order:

1. at the executable chosen in **Settings › Emulators** (an invalid path falls back to the automatic search);
2. in the launcher's own install, for emulators it manages (PCSX2);
3. in the **emulator folders**: those added in **Settings › Folders**, the `emulators` folder next to a `roms` folder (RetroBat layout), and on Windows `X:\RetroBat\emulators` on every drive;
4. in `PATH`, then in the usual install folders.

**Settings › Folders** sorts every path by itself: a game folder, a folder of emulators, or the executable of a known emulator. Paths added to the wrong list by an earlier version are moved at start-up.

## Which emulator runs which console

**Settings › Consoles and emulators** shows the emulator of each console and lets you change it with left and right. "Auto" takes the first **installed** emulator in the order of preference (for example GBA: mGBA, then RetroArch). A choice made by hand is kept until it is set back to "Auto".

## Settings changed by hand

The launcher tunes the emulator alone, until the user changes a setting inside the emulator:

- Before each launch, the launcher reads the emulator's own settings and compares each setting it manages with the last value it saw or wrote.
- A setting that changed was set by the user: the launcher no longer touches it, for any game, and tells so on the game screen.
- **Settings › Emulators › (emulator) › Re-enable automatic optimisation** gives these settings back to the launcher. It takes them back at the next launch.

When the launcher edits a settings file (PCSX2, DuckStation, PPSSPP), only the keys it manages change. The original file is copied once to `generated/backups/<emulator>/` in the data folder before the first change.

## System optimisations

They apply to every emulator, including those the launcher does not know (option **System optimisations**):

- **Windows**: the emulator starts with a higher process priority ("above normal").
- **Windows, machines with two GPUs** (integrated and dedicated): the emulator is set to use the high-performance GPU, unless a GPU was already chosen for it in the Windows graphics settings.

## Adding any emulator

**Settings › Emulators › Add an emulator** asks for its executable:

- If it is a recognised emulator (even one installed elsewhere), the launcher uses it with its automatic optimisation.
- Otherwise it becomes a **custom emulator**: choose the consoles it runs and, if needed, its arguments (`{rom}` stands for the game file; default: `{rom}`). It is launched directly, with the system optimisations and the smoothness check. The launcher does not change its own settings, because it does not know them.

## Teaching the launcher an emulator: profile files

A profile is a TOML file placed in the `emulator-profiles` folder of the data folder. It describes how to find the emulator, how to start a game, where its settings live and which settings make up each step of the quality ladder. Once a profile exists, the emulator is optimised like the bundled ones. A profile with the same `id` as a bundled one replaces it.

The bundled profiles are complete examples: `crates/cl-adapters/profiles/`.

```toml
id = "my-emulator"            # simple name, unique
name = "My Emulator"
license = "GPL-3.0-or-later"
homepage = "https://example.org"
systems = ["psx"]             # nes snes n64 gb gbc gba nds mastersystem megadrive psx ps2 psp gamecube wii wiiu
gpu_bound = true              # true: the ladder raises the resolution; false: it trades CPU for latency
args = ["-fullscreen", "{rom}"]

[exe]
windows = ["MyEmu.exe"]
linux = ["myemu"]

[settings]
method = "ini"                # "ini": file edited; "cli": session override; "none"
# Most specific first. Placeholders: {exe_dir} {home} {documents} {config} {data} {appdata} {localappdata} {programfiles}
files = ["{exe_dir}/settings.ini", "{config}/myemu/settings.ini"]
# For method = "cli": arguments added per setting, and the INI section where the emulator stores them
# cli = ["-C", "{key}={value}"]
# file_section = "general"

[settings.api]                # optional: value of the graphics back-end setting
key = "Video/Renderer"
vulkan = "Vulkan"
opengl = "OpenGL"

# Lightest first. Keys "Section/Key" for INI files.
[[ladder]]
label = "Native"
set = { "Video/Scale" = "1" }

[[ladder]]
label = "1080p"
set = { "Video/Scale" = "4" }
```

A file that cannot be read is listed in **Settings › Emulators**, and the other profiles still load.

## Known limits

- The launches and settings were checked with the real files of each emulator only in automated tests. Key names come from each project's documentation and default files, and must be confirmed on real installs.
- melonDS and Cemu: their settings are not tuned yet (melonDS changed its file format with version 1.0; Cemu sets resolution through per-game graphic packs).
- RetroBat rewrites some emulator settings each time it starts a game itself. The launcher then sees these changes as changes made by the user and leaves those settings alone, until they are given back with the re-enable button.
