# Interface prototype comparison

*[Version française](COMPARAISON.fr.md)*

> **Outcome (2026-10-23):** Qt Quick was chosen for this branch. The Tauri interface was removed from it (it stays on `feat/rust-tauri-prototype`), and the Rust engine is now linked into the Qt application as a library instead of being called through `clctl`. The measurements below were taken before that change.

Goal: choose the project's technical base from real results. The criteria are very advanced design, customisable themes, Windows first then Linux, and minimal resource use while games run.

## The three tracks

| | WPF | Tauri | Qt Quick |
|---|---|---|---|
| Location | branch `feat/desktop-foundation` (PR #1) | `app/`, `themes/` | `qt-prototype/` |
| Language | C# / .NET 8 | Rust + HTML/CSS/JS | C++17 + QML |
| Rendering | DirectX (WPF) | WebView2 (Windows), WebKitGTK (Linux) | GPU scene graph: Direct3D 11 (Windows), OpenGL/Vulkan (Linux) |
| Themes | XAML compiled into the app | HTML/CSS/JS folders, reloadable (F5) | QML folders read at start-up |
| Controllers | XInput (Xbox and compatible pads) | gilrs (SDL database) | SDL2 (SDL database) |
| Engine | Specific to the prototype (Dolphin) | Built-in Rust engine | Same Rust engine, through `clctl` |
| Linux | No (Windows only) | Yes | Yes |
| Windows package | Needs .NET Desktop Runtime 8, or ~150 MB self-contained* | 18 MB (WebView2 provided by Windows) | 42 MB (Qt DLLs included) |

\* Usual order of magnitude for a self-contained WPF app, not measured on this prototype.

## Real measurements on Linux

Conditions: virtual machine with 2 Xeon cores at 2.8 GHz, virtual display without a graphics card (software rendering), Release builds, same 11-game library, fake emulator. 30 s measurement, the launcher and all its child processes counted, the emulator excluded (`tools/measure-linux.py`). Two runs per prototype.

| | Tauri | Qt Quick |
|---|---|---|
| Start-up (window visible) | 0.52 s (2.5 s on the first cold start) | 0.52 s |
| Memory in the menu (PSS) | **204–206 MB**, 3 processes | **51–52 MB**, 1 process |
| CPU in the menu | 1.6–2.9 % of one core | 1.4–1.5 % of one core |
| Memory during a game | 206–207 MB | 55–56 MB (with `clctl` waiting) |
| CPU during a game | 0.9–1.0 % of one core | **0.0 %** |

The WPF prototype cannot run on Linux and was therefore not measured.

Read with care: without a graphics card the CPU also does the rendering, which inflates the "menu" figures. The memory and in-game differences, however, come from the technology and should show up on Windows too, where WebView2 also runs several processes.

## Measurements to make on Windows (the real test)

The `tools/measure-launcher.ps1` script measures the three prototypes the same way: time to open, private memory, CPU and GPU, in the menu then during a game. It adds up all the launcher's processes and excludes the emulator.

```powershell
powershell -ExecutionPolicy Bypass -File measure-launcher.ps1 -Exe .\ConsoleLauncher-Tauri\console-launcher.exe -Name tauri
powershell -ExecutionPolicy Bypass -File measure-launcher.ps1 -Exe .\ConsoleLauncher-Qt\console-launcher-qt.exe -Name qt
powershell -ExecutionPolicy Bypass -File measure-launcher.ps1 -Exe <WPF folder>\ConsoleLauncher.exe -Name wpf
```

For a fair comparison: same PC, same screen, same game and same emulator, no other heavy program open. Run each measurement twice and keep the second.

## What the measurements do not tell

| Criterion | WPF | Tauri | Qt Quick |
|---|---|---|---|
| Design freedom | Very high (XAML, animations, styles) | Very high (the whole web: CSS, animations, WebGL) | Very high (GPU animations, shaders, particles) |
| Ease of making a theme | Medium: XAML, hot reload uncommon | **Highest**: HTML/CSS, known by many people | Good: QML is declarative and readable, but less known than the web |
| Animation smoothness | Good | Good on Windows; WebKitGTK more uneven on Linux | **Excellent** on both systems (native GPU rendering) |
| Linux later | Rewrite needed (Avalonia, for instance) | Yes, with a different web engine (WebKitGTK) | Yes, same rendering |
| Development difficulty | Low to medium | Medium (Rust + web) | Higher (C++, Qt tooling) |
| Licence | MIT (.NET) | MIT / Apache-2.0 | LGPL-3.0 (dynamic linking, compatible with open source) |
| Existing examples | Playnite (game launcher in WPF) | — | Pegasus Frontend (emulator launcher in QML, community themes) |

## Provisional reading

- **Engine**: the Rust engine (library, settings, optimisation, updates) already works with two different interfaces. It can serve as the base whichever interface is chosen.
- **Resource use**: Qt Quick is clearly ahead on Linux, with about 4 times less memory and nothing measurable during a game. That is the "use little while games run" criterion.
- **Themes**: Tauri remains the most accessible for theme authors, with ordinary HTML/CSS.
- **WPF**: it blocks the Linux goal without rewriting the interface.

The final choice will be made on the Windows figures for the three prototypes, measured with the script above.
