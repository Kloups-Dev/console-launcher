# Console Launcher — application Qt Quick

*[English version](README.en.md)*

L'interface du lanceur : consoles de gauche à droite, liste des jeux, détail du jeu, lancement direct, retour au menu, et un écran de réglages utilisable à la manette.

| Élément | Choix |
|---|---|
| Langage | C++17 |
| Interface | Qt Quick (QML), Qt 6.8 LTS ; rendu GPU via Direct3D 11 sous Windows, OpenGL/Vulkan sous Linux |
| Thèmes | Dossiers `themes/<nom>/Main.qml` lus au démarrage, modifiables sans recompiler ([THEMES.fr.md](../docs/prototype/THEMES.fr.md)) |
| Composants intégrés | `import ConsoleLauncher` : panneau de réglages (`ui/SettingsPanel.qml`) |
| Manettes | SDL2 (base GameController : Xbox, PlayStation, Switch, manettes génériques) |
| Moteur | Bibliothèque Rust `console_launcher_engine` (crate `cl-ffi`), chargée par l'application ; les appels tournent sur des fils de travail ([ENGINE_API.fr.md](../docs/prototype/ENGINE_API.fr.md)) |
| Pendant une partie | Fenêtre masquée (Qt Quick arrête de dessiner) et SDL complètement arrêté |

## Versions publiées

Un tag commençant par `v` (par exemple `v0.3.0-qt`) lance l'intégration continue et publie les deux paquets en **pré-version** sur GitHub, rattachée au commit du tag et jamais marquée comme dernière version. Le tag peut venir de `git push`, ou de la page GitHub (Releases › Draft a new release, nouveau tag, cible = la branche, « Set as a pre-release ») : l'intégration continue ajoute alors les paquets à cette version.

## Compiler

Prérequis : Rust (stable), CMake 3.21+, un compilateur C++17, Qt 6.8 ou plus récent (Core, Gui, Qml, Quick, Quick Dialogs), SDL2. Détail par système : [DEPENDENCIES.fr.md](../docs/prototype/DEPENDENCIES.fr.md).

```bash
cmake -S qt-prototype -B build-qt -DCMAKE_BUILD_TYPE=Release
cmake --build build-qt
./build-qt/console-launcher-qt
```

CMake lance lui-même `cargo build -p cl-ffi -p cl-cli`, puis copie la bibliothèque du moteur et les thèmes à côté de l'exécutable. `CL_ENGINE_PREBUILT_DIR` indique à CMake un moteur déjà compilé à utiliser à la place.

**Windows** (invite de commandes développeur de Visual Studio 2022, kit Qt `msvc2022_64`, archive SDL2 « devel VC ») :

```powershell
cmake -S qt-prototype -B build-qt -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH=C:\Qt\6.8.3\msvc2022_64 -DSDL2_DIR=C:\SDL2-2.32.10\cmake
cmake --build build-qt
windeployqt --release --qmldir qt-prototype/themes --qmldir qt-prototype/ui build-qt/console-launcher-qt.exe
copy C:\SDL2-2.32.10\lib\x64\SDL2.dll build-qt\
```

## Outils de mesure

Les paquets produits par l'intégration continue contiennent les outils de mesure de fluidité dans `tools/`, à côté de l'exécutable : `tools/presentmon/PresentMon-*.exe` sous Windows, `tools/mangohud/libMangoHud*.so` sous Linux. Pour une compilation locale, les y placer à la main, ou indiquer leur dossier dans la variable `CONSOLE_LAUNCHER_TOOLS`. Une copie installée sur la machine est prioritaire.

## Variables

| Variable | Usage |
|---|---|
| `CONSOLE_LAUNCHER_HOME` | Dossier de données (réglages, profils, captures) |
| `CONSOLE_LAUNCHER_QT_THEMES` | Dossier des thèmes, pour travailler sur un thème ailleurs |
| `CONSOLE_LAUNCHER_TOOLS` | Dossier des outils de mesure |
| `RUST_LOG` | Niveau de journal du moteur (`info` par défaut) |

## Journaux et test de démarrage

L'interface écrit `console-launcher.log` et le moteur `console-launcher-engine.log` à côté de l'exécutable (ou dans le dossier temporaire quand ce dossier est en lecture seule). Ils sont réécrits à chaque démarrage. Quand un thème ne peut pas être chargé, une fenêtre affiche les erreurs.

`console-launcher-qt --smoke-test` charge le thème, demande la bibliothèque, puis s'arrête : code 0 si le thème s'est chargé. L'intégration continue le lance sur les deux paquets.
