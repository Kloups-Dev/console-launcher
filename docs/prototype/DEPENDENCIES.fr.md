# Dépendances

*[English version](DEPENDENCIES.en.md)*

Tout est open source. Rien n'est propriétaire ni payant. Versions relevées dans `Cargo.lock` au moment de la v0.1.

## Bibliothèques Rust (compilées dans le logiciel)

| Bibliothèque | Version | Licence | Rôle |
|---|---|---|---|
| [tauri](https://tauri.app) | 2.12 | MIT / Apache-2.0 | Fenêtre et pont entre l'interface web et le moteur |
| [gilrs](https://gitlab.com/gilrs-project/gilrs) | 0.11 | MIT / Apache-2.0 | Manettes (base SDL, XInput sous Windows, evdev sous Linux) |
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
| Tous | [Rust](https://rustup.rs) stable (testé avec 1.97) |
| Windows 10 / 11 | Microsoft C++ Build Tools (avec « Développement Desktop en C++ »). WebView2 est déjà présent sur Windows 10/11 à jour |
| Debian / Ubuntu | `sudo apt install libwebkit2gtk-4.1-dev libudev-dev librsvg2-dev libssl-dev pkg-config build-essential` |
| Fedora | `sudo dnf install webkit2gtk4.1-devel systemd-devel librsvg2-devel openssl-devel` |
| Arch | `sudo pacman -S webkit2gtk-4.1 base-devel librsvg openssl` |

Pour créer les installateurs : `cargo install tauri-cli --version "^2" --locked`, puis `cargo tauri build` depuis `app/src-tauri`.

## À l'exécution

- **Windows** : WebView2 (inclus dans Windows 10/11).
- **Linux** : WebKitGTK 4.1 (installé avec la plupart des bureaux).
- **Facultatif** : un pilote Vulkan pour la détection de la carte graphique. Sans lui, le lanceur démarre quand même et choisit des réglages prudents.

## Outils de mesure (facultatifs, non fournis)

| Outil | Licence | Système | Utilisation |
|---|---|---|---|
| [MangoHud](https://github.com/flightlessmango/MangoHud) | MIT | Linux | Enregistre le temps de chaque image pendant la partie. À installer avec le gestionnaire de paquets |
| [PresentMon](https://github.com/GameTechDev/PresentMon) | MIT | Windows | Même rôle. Indiquer son chemin dans `perf.presentmon_path`. Nécessite les droits administrateur ou le groupe « Utilisateurs du journal de performances » |

Sans ces outils, les jeux se lancent normalement avec le réglage estimé. Seule la mesure automatique est mise en pause, et le lanceur le signale.

## Émulateurs (non fournis, téléchargés depuis leurs sources officielles)

| Émulateur | Licence | Consoles |
|---|---|---|
| [RetroArch](https://www.retroarch.com) | GPL-3.0 | NES, SNES, N64, Game Boy, GBA, DS, Master System, Mega Drive, PlayStation, PSP |
| [Dolphin](https://dolphin-emu.org) | GPL-2.0-or-later | GameCube, Wii |
| [PCSX2](https://pcsx2.net) | GPL-3.0-or-later | PlayStation 2 |

Cores RetroArch utilisés par défaut : Mesen (GPL-3.0), Snes9x\*, Mupen64Plus-Next (GPL-2.0), Gambatte (GPL-2.0), mGBA (MPL-2.0), melonDS DS (GPL-3.0), Genesis Plus GX\*, Beetle PSX HW (GPL-2.0), PPSSPP (GPL-2.0).

\* Snes9x et Genesis Plus GX ont des licences qui **interdisent l'usage commercial**. Ce n'est pas un problème tant que le lanceur ne les redistribue pas, ce qui est le cas : l'utilisateur les installe depuis RetroArch.
