# Dépendances

*[English version](DEPENDENCIES.en.md)*

Tout est open source. Rien n'est propriétaire ni payant. Versions de la v0.3.

## Interface (C++)

| Bibliothèque | Version | Licence | Rôle |
|---|---|---|---|
| [Qt](https://www.qt.io) (Core, Gui, Qml, Quick, Quick Dialogs) | 6.8 LTS | LGPL-3.0 (liaison dynamique, DLL fournies à côté du lanceur) | Fenêtre, thèmes QML, rendu GPU, sélecteurs de fichiers |
| [SDL2](https://www.libsdl.org) | 2.32 | Zlib | Manettes (base GameController de SDL) |

## Moteur : bibliothèques Rust (compilées dans `console_launcher_engine`)

| Bibliothèque | Version | Licence | Rôle |
|---|---|---|---|
| [sysinfo](https://github.com/GuillaumeGomez/sysinfo) | 0.37 | MIT | Processeur, mémoire, système |
| [ash](https://github.com/ash-rs/ash) | 0.38 | MIT / Apache-2.0 | Cartes graphiques via Vulkan (chargé au lancement, facultatif) |
| [ureq](https://github.com/algesten/ureq) | 3.5 | MIT / Apache-2.0 | Vérification et téléchargement des mises à jour |
| [zip](https://github.com/zip-rs/zip2) | 4.6 | MIT | Archives .zip des émulateurs |
| [sevenz-rust2](https://github.com/hasenbanck/sevenz-rust2) | 0.18 | Apache-2.0 | Archives .7z des émulateurs |
| [serde](https://serde.rs), serde_json, [toml](https://github.com/toml-rs/toml) | 1.x / 0.8 | MIT / Apache-2.0 | Fichiers de réglages et de profils |
| [walkdir](https://github.com/BurntSushi/walkdir) | 2.5 | MIT / Unlicense | Parcours des dossiers de jeux |
| [directories](https://github.com/dirs-dev/directories-rs) | 6 | MIT / Apache-2.0 | Dossiers standard par utilisateur |
| [clap](https://github.com/clap-rs/clap) | 4.6 | MIT / Apache-2.0 | Ligne de commande de `clctl` |
| thiserror, anyhow, log, env_logger | — | MIT / Apache-2.0 | Erreurs et journaux |
| tempfile *(tests seulement)* | 3 | MIT / Apache-2.0 | Dossiers temporaires dans les tests |

L'arbre complet compte environ 500 paquets. Leurs licences sont MIT, Apache-2.0, BSD, ISC, Zlib, Unicode-3.0 ou MPL-2.0, toutes compatibles avec la GPL-3.0. Pour les lister : `cargo metadata --format-version 1`, ou `cargo install cargo-about` pour générer un fichier de crédits.

## Pour compiler

| Système | À installer |
|---|---|
| Tous | [Rust](https://rustup.rs) stable (testé avec la 1.97), [CMake](https://cmake.org) 3.21+, Qt 6.8 (installateur en ligne de Qt ou `aqtinstall`), SDL2 |
| Windows 10 / 11 | Visual Studio 2022 ou ses Build Tools (« Développement Desktop en C++ »), kit Qt `msvc2022_64`, archive SDL2 « devel VC » de [libsdl.org](https://github.com/libsdl-org/SDL/releases) |
| Debian / Ubuntu | `sudo apt install build-essential cmake libsdl2-dev libgl1-mesa-dev`, Qt 6.8 par l'installateur de Qt |
| Fedora | `sudo dnf install gcc-c++ cmake SDL2-devel mesa-libGL-devel qt6-qtdeclarative-devel` (si la distribution fournit Qt 6.8 ou plus récent) |
| Arch | `sudo pacman -S base-devel cmake sdl2 qt6-declarative` |

Commandes : [qt-prototype/README.fr.md](../../qt-prototype/README.fr.md). Les paquets sont produits par l'intégration continue (`.github/workflows/ci.yml`) : dossier Windows avec `windeployqt`, AppImage Linux avec `linuxdeploy`.

## À l'exécution

- **Windows** : rien à installer ; les DLL de Qt et de SDL2 sont dans le dossier.
- **Linux** : l'AppImage contient Qt ; un bureau X11 ou Wayland suffit.
- **Facultatif** : un pilote Vulkan pour détecter la carte graphique. Sans lui, le lanceur démarre quand même et choisit des réglages prudents.

## Outils de mesure (fournis avec les paquets)

| Outil | Version | Licence | Système | Usage |
|---|---|---|---|---|
| [PresentMon](https://github.com/GameTechDev/PresentMon) | 2.6.0 | MIT | Windows | Enregistre la durée de chaque image pendant le jeu. Dans `tools/presentmon/`. Nécessite les droits administrateur ou le groupe « Utilisateurs du journal de performances » : **Réglages › Mesure de fluidité › Autoriser la mesure** |
| [MangoHud](https://github.com/flightlessmango/MangoHud) | 0.8.4 | MIT | Linux | Même rôle. Dans `tools/mangohud/` à l'intérieur de l'AppImage. Une copie installée sur la machine est prioritaire |

Leurs fichiers de licence sont à côté d'eux. Sans mesure, les jeux se lancent normalement avec les réglages estimés, et le lanceur le signale.

## Émulateurs (non fournis, téléchargés depuis leurs sources officielles)

| Émulateur | Licence | Consoles |
|---|---|---|
| [RetroArch](https://www.retroarch.com) | GPL-3.0 | NES, SNES, N64, Game Boy, GBA, DS, Master System, Mega Drive, PlayStation, PSP |
| [Dolphin](https://dolphin-emu.org) | GPL-2.0-or-later | GameCube, Wii |
| [PCSX2](https://pcsx2.net) | GPL-3.0-or-later | PlayStation 2 |
| [mGBA](https://mgba.io) | MPL-2.0 | Game Boy, Game Boy Color, GBA |
| [DuckStation](https://www.duckstation.org) | CC-BY-NC-ND-4.0 (*) | PlayStation |
| [PPSSPP](https://www.ppsspp.org) | GPL-2.0-or-later | PSP |
| [melonDS](https://melonds.kuribo64.net) | GPL-3.0-or-later | Nintendo DS |
| [Cemu](https://cemu.info) | MPL-2.0 | Wii U |

Cores RetroArch utilisés par défaut : Mesen (GPL-3.0), Snes9x\*, Mupen64Plus-Next (GPL-2.0), Gambatte (GPL-2.0), mGBA (MPL-2.0), melonDS DS (GPL-3.0), Genesis Plus GX\*, Beetle PSX HW (GPL-2.0), PPSSPP (GPL-2.0).

(\*) Les versions récentes de DuckStation interdisent l'usage commercial et la redistribution de copies modifiées. Le lanceur ne fait que démarrer la copie installée par l'utilisateur.

\* Snes9x et Genesis Plus GX ont des licences qui **interdisent l'usage commercial**. Ce n'est pas un problème tant que le lanceur ne les redistribue pas, ce qui est le cas : l'utilisateur les installe depuis RetroArch.
