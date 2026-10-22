# Prototype C++ / Qt Quick

*[English version](README.en.md)*

Troisième piste d'interface, écrite pour comparer les technologies. Elle reprend le même parcours que les deux autres prototypes : consoles de gauche à droite, liste des jeux, fiche du jeu, lancement direct, retour au menu.

| Élément | Choix |
|---|---|
| Langage | C++17 |
| Interface | Qt Quick (QML), rendu GPU via Direct3D 11 sous Windows, OpenGL/Vulkan sous Linux |
| Thèmes | Dossiers `themes/<nom>/Main.qml` lus au démarrage. On les modifie sans recompiler |
| Manettes | SDL2 (base GameController : Xbox, PlayStation, Switch, génériques) |
| Moteur | `clctl`, le même moteur Rust que le prototype Tauri (bibliothèque, réglages, mesure, mises à jour) |
| Pendant un jeu | Fenêtre masquée (Qt Quick arrête de dessiner) et SDL complètement arrêté |

Le moteur n'est pas réécrit : le prototype appelle `clctl` pour chaque demande (scan, infos, lancement, favori). Cela montre aussi qu'une interface Qt peut s'appuyer sur le moteur Rust.

## Compiler

Prérequis : CMake 3.21+, un compilateur C++17, Qt 6.4 ou plus récent (Core, Gui, Qml, Quick), SDL2.

```bash
cmake -S qt-prototype -B build-qt -DCMAKE_BUILD_TYPE=Release
cmake --build build-qt
cargo build --release -p cl-cli          # moteur
cp target/release/clctl build-qt/        # clctl.exe sous Windows
./build-qt/console-launcher-qt
```

Sous Windows avec Qt installé (installateur en ligne de Qt, kit MinGW ou MSVC) : même commandes, puis `windeployqt --qmldir qt-prototype/themes build-qt/console-launcher-qt.exe` pour copier les DLL Qt à côté de l'exécutable.

## Thèmes

Un thème est un dossier contenant `Main.qml`, dont la racine est une `Window`. Il reçoit un seul objet, `Launcher` :

| Membre | Rôle |
|---|---|
| `systems`, `games` | Bibliothèque (mêmes données que `clctl --json scan`) |
| `nav(action)` (signal) | `up`, `down`, `left`, `right`, `confirm`, `back`, `menu`, `favorite`, `pageLeft`, `pageRight` |
| `requestInfo(id)` → `gameInfoReady(id, info)` | Émulateur, palier de qualité, notes |
| `play(id)` → `sessionEnded(report)` | Lance le jeu ; `running` vaut `true` pendant la partie |
| `toggleFavorite(id)` → `favoriteChanged(id, on)` | Favoris |
| `addRomDir(chemin)`, `setTheme(nom)`, `themes`, `theme` | Réglages |
| `fileUrl(chemin)`, `asset("img/fond.png")` | Images de la bibliothèque et du thème |
| `toggleFullscreen()`, `quit()`, `padName`, `lastError` | — |

Pour travailler sur un thème depuis un autre dossier : variable `CONSOLE_LAUNCHER_QT_THEMES`. Deux modèles sont fournis, `horizon` et `grid`.
