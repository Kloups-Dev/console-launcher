# Creating a theme

*[Version française](THEMES.fr.md)*

A theme is a folder with 4 files: HTML, CSS, JavaScript and a manifest. It decides **everything** on screen: layout, colours, animations, sounds, menu order. The launcher provides the data (consoles, games, covers, settings) and the controller actions. The theme never needs to know which emulator is used.

Nobody has to create a theme to play: "Horizon" is active by default.

## Where themes live

`<data folder>/themes/<name>/`. The two templates are copied there on first start and never overwritten afterwards, so they can be edited directly.

```
themes/my-theme/
  theme.json    manifest (name, adjustable options)
  index.html    page structure
  style.css     look
  theme.js      behaviour (receives the Launcher object)
  ...           images, fonts, sounds: anything
```

The folder name may only contain letters, digits, `-` and `_`.

## Starting from a template

1. Copy `themes/grid` (the shortest, about 80 lines of JS) or `themes/horizon` (more complete).
2. Rename the folder and the `name` field in `theme.json`.
3. Open the menu (Tab / Start) → Theme, until the new one shows.
4. Edit, then press **F5** to reload. **F11** toggles fullscreen.

To work straight from a development folder:

```bash
CONSOLE_LAUNCHER_THEMES=/path/to/my-themes cargo run -p console-launcher
```

JavaScript errors are shown at the bottom of the screen, and the menu stays reachable with Tab to switch theme.

## The `theme.json` manifest

```json
{
  "name": "My theme",
  "author": "Nickname",
  "version": "1.0.0",
  "description": "One line",
  "options": [
    { "id": "accent", "type": "color", "label": "Accent", "default": "#ffc440" },
    { "id": "background", "type": "select", "label": "Background", "choices": ["aurora", "plain"], "default": "aurora" },
    { "id": "clock", "type": "bool", "label": "Clock", "default": true }
  ]
}
```

Each option becomes:
- a CSS variable `--opt-<id>` (colours, numbers, text);
- a `data-opt-<id>` attribute on `<html>`, handy for choices: `html[data-opt-clock="false"] .clock { display: none }`;
- a value readable in JS with `Launcher.option("accent")`.

Values chosen by the user are saved in `user-options.json`, next to the theme.

## `Launcher` API

Every asynchronous function returns a promise.

| Function | Purpose |
|---|---|
| `onNav(cb)` | Receives actions: `up`, `down`, `left`, `right`, `confirm`, `back`, `menu`, `favorite`, `pageLeft`, `pageRight`. They come from the controller, the keyboard, or auto-repeat while a direction is held |
| `onSession(cb)` | `{state: "started"}` when a game starts, then `{state: "ended", report}` or `{state: "failed", error}` |
| `onPad(cb)` | Controller connected or disconnected: `{name, connected}` |
| `getLibrary()` | `{systems, games}` (details below) |
| `gameInfo(id)` | `{ready, emulator, stepLabel, ladder, settled, measure, belowRequirements, notes}` or `{ready: false, error}` |
| `play(id)` | Starts the game. The promise resolves when the emulator closes, with the report (measured smoothness, next setting) |
| `toggleFavorite(id)` | Returns the new state |
| `setQuality(id, step)` | Fixes a rung by hand. Automatic optimisation stops for that game |
| `getSettings()` / `saveSettings(s)` | Reads or writes `config.toml` |
| `hardware(refresh)` | Detected machine. `true` runs the test again |
| `listThemes()` / `setTheme(id)` | Switch theme (the page reloads) |
| `option(id)` / `setOption(id, value)` | Manifest options |
| `asset("img/bg.png")` | URL of a file in the theme folder |
| `openSettings()` | Opens the built-in settings panel, so a theme does not have to code one |
| `toggleFullscreen()`, `quit()` | — |
| `escapeHtml(text)` | Use it to put game titles into HTML |

DOM event: `launcher:library-changed`, fired when the user adds a game folder from the built-in panel. The library should then be reloaded.

### `getLibrary()` data

```js
systems: [{
  info: { id: "snes", name: "Super Nintendo", manufacturer: "Nintendo", year: 1990, cost: 1, ... },
  games: 42,              // number of games
  emulator: "retroarch",
  emulator_found: true
}]
games: [{
  id: "6edaa4f992fd1b98", // stable: favourites and profiles are tied to it
  system: "snes",
  title: "My game",       // cleaned: "(Europe)", "[!]"… removed
  path: "...", cover: "...", coverUrl: "...", favorite: false
}]
```

## Good practice

- React only to `onNav` actions, never read the controller directly: the theme then works with every controller and with the keyboard.
- Render long lists as a window (about ten items around the selection), like the templates, to stay smooth with thousands of games.
- Load `gameInfo` after a short delay while scrolling (see Horizon).
- Load nothing from the Internet: the launcher must work offline.
- A theme's JavaScript has the same rights as the interface. Only install themes from people you trust.
