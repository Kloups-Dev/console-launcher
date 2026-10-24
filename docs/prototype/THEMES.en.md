# Creating a theme

> **Current status: external themes are disabled.** The application only loads
> the QML interface compiled into the executable. The old `Main.qml` entry
> points were removed and are no longer executed. The future
> Theme v1 format will be a validated data pack (manifest, colours, fonts,
> images and sounds), with the built-in functional interface as its fallback.
> This prevents a theme from calling the engine or breaking start-up. The rest
> of this document describes the old prototype and is no longer a supported API.

*[Version française](THEMES.fr.md)*

A theme is a folder holding a `Main.qml` file written in QML, Qt Quick's declarative language. It decides **everything** on screen: layout, colours, animations, sounds, menu order. The launcher provides the data (consoles, games, covers, settings) and the controller actions. The theme never needs to know which emulator is used.

Nobody has to create a theme to play: "Horizon" is active by default.

## Where themes live

In the `themes` folder next to the executable (`ConsoleLauncher\themes` in the Windows package). Each sub-folder holding a `Main.qml` is a theme.

```
themes/my-theme/
  Main.qml       the theme; its root is a Window
  ...            images, fonts, sounds, other QML files: anything
```

To work from a development folder, set the `CONSOLE_LAUNCHER_QT_THEMES` variable to the folder holding the themes.

## Starting from a template

1. Copy `themes/grid` (the shortest, about 120 lines) or `themes/horizon` (more complete).
2. Rename the folder.
3. Open the settings (Tab / Start) → Theme, until the new one shows.
4. Edit `Main.qml`, then switch theme back and forth to reload it. **F11** toggles fullscreen.

QML errors are printed in the console; the settings stay reachable with Tab.

## Minimal theme

```qml
import QtQuick
import QtQuick.Window
import ConsoleLauncher            // built-in components (settings panel)

Window {
    visible: true
    width: 1280; height: 720
    color: "#101010"

    property int sel: 0

    Connections {
        target: Launcher
        function onNav(a) {
            if (a === "down") sel = Math.min(Launcher.games.length - 1, sel + 1);
            else if (a === "up") sel = Math.max(0, sel - 1);
            else if (a === "confirm") Launcher.play(Launcher.games[sel].id);
            else if (a === "menu") Launcher.openSettings();
        }
    }

    ListView {
        anchors.fill: parent; anchors.margins: 40
        model: Launcher.games
        delegate: Text { text: modelData.title; color: index === sel ? "gold" : "white"; font.pixelSize: 22 }
    }

    SettingsPanel { anchors.fill: parent }   // opened by Launcher.openSettings()
}
```

## The `Launcher` object

### Properties

| Property | Content |
|---|---|
| `systems` | Consoles that have games (details below) |
| `games` | Every game found |
| `running` | `true` while a game runs (the window is hidden meanwhile) |
| `padName` | Name of the connected controller, or empty |
| `themes`, `theme` | Themes available, active theme |
| `language` | `fr` or `en`, chosen in the settings |
| `settingsOpen` | `true` while the settings panel is open |
| `lastError` | Last error message, or empty |
| `version` | Launcher version |

### Signals

| Signal | When |
|---|---|
| `nav(action)` | Controller or keyboard: `up`, `down`, `left`, `right`, `confirm`, `back`, `menu`, `favorite`, `pageLeft`, `pageRight`. Directions repeat while held. Not sent while the settings panel is open |
| `libraryChanged()` | `systems` and `games` were reloaded |
| `gameInfoReady(id, info)` | Answer to `requestInfo(id)` |
| `favoriteChanged(id, on)` | Answer to `toggleFavorite(id)` |
| `sessionEnded(report)` | The emulator closed; `report` holds the measured smoothness and the next setting, or `error` |

### Functions

| Function | Purpose |
|---|---|
| `requestInfo(id)` | Asks for a game's details → `gameInfoReady`. `info` is `{ready, emulator, emulatorId, tunesSettings, stepLabel, ladder, settled, measure, belowRequirements, notes}` or `{ready: false, code, emulatorId, error}` (`code`: `emulator_missing`, `no_emulator`…) |
| `play(id)` | Starts the game |
| `toggleFavorite(id)` | Favourite on / off |
| `openSettings(target)` | Opens the built-in settings panel. `target` (optional): `{emulator: id}` opens that emulator's page, `{page: "systems" \| "emulators" \| "folders" \| "capture"}` a page. Use it when `info.ready` is false to send the player straight to the fix |
| `request(method, params, callback)` | Calls any engine method ([ENGINE_API.en.md](ENGINE_API.en.md)); `callback(ok, resultOrError)` |
| `reload()` | Reloads the library |
| `setTheme(name)` | Switches theme |
| `fileUrl(path)` | URL of a local file (covers) |
| `asset("img/bg.png")` | URL of a file in the theme folder |
| `toggleFullscreen()`, `quit()` | — |

### Data

```js
systems: [{
  info: { id: "snes", name: "Super Nintendo", manufacturer: "Nintendo", year: 1990, cost: 1, ... },
  games: 42,                 // number of games
  emulator: "retroarch",     // emulator id
  emulator_name: "RetroArch",
  emulator_found: true
}]
games: [{
  id: "6edaa4f992fd1b98",    // stable: favourites and profiles are tied to it
  system: "snes",
  title: "My game",          // cleaned: "(Europe)", "[!]"… removed
  path: "...", cover: "...", favorite: false
}]
```

## Built-in components

`import ConsoleLauncher` gives access to:

| Component | Purpose |
|---|---|
| `SettingsPanel` | The whole settings screen (consoles, emulators, folders, smoothness check, language…), usable with a controller. Place it last in the window with `anchors.fill: parent`. Its colours can be changed with the `text`, `accent` and `panelColor` properties |

## Good practice

- React only to `nav`, never read the controller directly: the theme then works with every controller and with the keyboard.
- Use `ListView` / `GridView` for game lists: they only create the visible items and stay smooth with thousands of games.
- Ask for `requestInfo` after a short delay while scrolling (see Horizon).
- Load nothing from the Internet: the launcher must work offline.
- A theme's QML has the same rights as the interface. Only install themes from people you trust.
