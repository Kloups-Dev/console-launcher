# Automatic optimisation

*[Version française](PERFORMANCE.fr.md)*

Goal: every game runs at full speed, without stutter, at the best quality the machine allows, without the user having to adjust anything.

## The principle in three steps

1. **Simulation, before the first session.** The launcher detects the processor, memory and graphics card (through Vulkan), then runs a CPU test of about one second. The test runs a small code interpreter, the same kind of work as emulating a processor. From these data it picks a starting rung on the emulator's "quality ladder".
2. **Real conditions, while playing.** Until the right setting is found, sessions are measured: MangoHud (Linux) or PresentMon (Windows) records the duration of every frame for 60 seconds, after 20 seconds of loading.
3. **Adjustment.** If the session was smooth, the launcher tries the next rung up at the next session. If it stuttered, it goes back to the last smooth rung (or one rung down). It stops when a smooth rung sits right below a stuttering one. This takes at most as many sessions as the ladder has rungs (usually 5), and often 1 or 2.

The result is specific to each game and saved in `profiles/<id>.json`. If the machine changes (another processor or graphics card), or an incompatible automatic policy is corrected, the search starts again.

## Quality ladders (v0.1)

| Emulator | Rungs, lightest to heaviest | What changes |
|---|---|---|
| Dolphin (GameCube, Wii) | Native · 720p · 1080p · 1440p · 4K | Internal resolution (×1 to ×6); hybrid ubershaders above native to avoid compilation stutter |
| PCSX2 (PS2) | Native · 720p · 1080p · 1440p · 4K | Resolution multiplier (×1 to ×6) |
| RetroArch (light consoles) | Eco · Standard · Low latency · Lowest latency | Run-ahead (1 to 2 frames): less latency, for more CPU work |
| RetroArch (N64, DS, PSP) | Eco · Standard | Run-ahead off, too costly |

The graphics API is selected automatically for the emulator, operating system and GPU vendor. For Dolphin on Windows, Vulkan is preferred on AMD, D3D12 on Intel, and D3D11 on NVIDIA to avoid the documented Hybrid/Vulkan pipeline-swap stutter on that driver. An unknown vendor gets the conservative D3D11 choice. On Linux, the stability-first profile prefers OpenGL over Vulkan for Intel and NVIDIA; AMD keeps Vulkan.

Dolphin's native rung uses specialized shaders, which suit modest GPUs. Higher rungs use Hybrid Ubershaders. Skip Drawing is excluded from automatic choices because it saves time by not drawing objects whose shader is not ready. The launcher also leaves Dual Core to Dolphin, preserving its per-game compatibility exceptions.

## Smoothness criteria

A session is judged **smooth** when:
- the average reaches at least 97 % of the console's refresh rate (60 or 50 frames/s);
- the "1 % low" (the slowest frame among the best 99 %) reaches at least 85 % of that rate;
- fewer than 1 % of frames last more than 1.5 times the expected time.

During a measurement, the emulator presents every console refresh (Dolphin gets an option so it does not skip duplicate frames). Otherwise a game designed for 30 frames/s and a game running at half speed would give the same measurement.

On Windows, PresentMon targets only the process ID started by the launcher. It does not track keyboard or mouse input. If no CSV is produced, its short local log is included in the session report so the failure is explainable instead of silent.

These thresholds are **starting values**. They will be confirmed or corrected after the first tests.

## Respecting users' settings

- Emulator **installed by the user**: its files are never modified. Settings go through the command line and only apply to the session (RetroArch `--appendconfig`, Dolphin `--config`). PCSX2 has no such option, so it keeps its own settings.
- Emulator **managed by the launcher**: the INI file is edited key by key. The launcher remembers what it wrote. If the user changes a value by hand, the launcher keeps it from then on.
- `locked_keys` in `config.toml`: settings the launcher never touches.
- Picking a rung by hand (`clctl set-step` or `Launcher.setQuality`) stops the search for that game.

## Calibrating (to do with the first testers)

The estimate values (`crates/cl-perf/src/estimate.rs`) and the CPU test reference (`REFERENCE_NS` in `crates/cl-hwprobe/src/bench.rs`) have not been measured on real machines yet. Proposed protocol:

1. On each test machine: `clctl probe --json > machine.json`.
2. For 3 to 5 games per console: play with measurement on until the setting is found (`clctl plan` shows "settled").
3. Collect `hardware.json` and `profiles/*.json` (they hold the statistics of every session).
4. Compare the estimated rung with the final rung. Adjust `required_cpu` and `gpu_budget` so the estimate is right in most cases.
5. Record machines, games and results in a shared table before changing thresholds.

Measuring a capture made by hand:

```bash
mangohud --dlsym <emulator> ...       # Linux, log with Shift+F2
clctl analyse capture.csv --game "game name"
```

## To check on the real emulator versions

- Dolphin: confirm the `SkipPresentingDuplicateFrames` measurement key against the exact bundled release.
- PCSX2: `Renderer` numbers (14 = Vulkan, 3 = D3D11, 15 = D3D12, 12 = OpenGL) and launching with `-batch -nogui`.
- PresentMon 2.x: command-line options used (`--process_name`, `--timed`, `--terminate_after_timed`, `--v1_metrics`).
- MangoHud: `autostart_log` and `log_duration` options.

## Known limits

- A game's first launch may stutter while shaders compile. The first 60 frames are ignored and measurement starts after 20 s, but a game that compiles a lot may be judged too harshly the first time.
- Measurement only sees what reaches the screen. Audio-only slowdown is not detected.
- On Windows, PresentMon needs a permission: **Settings › Smoothness check › Allow the measurement**, then sign out of Windows and back in (see [DEPENDENCIES.en.md](DEPENDENCIES.en.md)).
