# Changelog

All notable changes are listed here, newest first. Format based on [Keep a Changelog](https://keepachangelog.com/), versions follow [Semantic Versioning](https://semver.org/).

Toutes les modifications notables sont listées ici, de la plus récente à la plus ancienne.

## [0.2.1] — 2026-10-23 — Interface comparison

### English

**Added**
- Third interface prototype in C++17 / Qt Quick (`qt-prototype/`): QML themes loaded from disk ("horizon", "grid"), SDL2 controller input, same Rust engine through `clctl`, SDL shut down and window hidden during games.
- `clctl info` and `clctl favorite` commands, used by the Qt interface.
- Measurement tools: `tools/measure-launcher.ps1` (Windows) and `tools/measure-linux.py` (Linux), counting the launcher and its child processes, not the emulator.
- Comparison of the WPF, Tauri and Qt Quick prototypes with real Linux measurements (`docs/comparison/`).

**Changed**
- CPU test reference recalibrated on a release build (a 2.8 GHz cloud Xeon core now scores about 600).
- Tauri prototype: controller polling slowed down 25 times while a game runs.

### Français

**Ajouts**
- Troisième prototype d'interface en C++17 / Qt Quick (`qt-prototype/`) : thèmes QML lus depuis le disque (« horizon », « grid »), manettes via SDL2, même moteur Rust via `clctl`, SDL arrêté et fenêtre masquée pendant les jeux.
- Commandes `clctl info` et `clctl favorite`, utilisées par l'interface Qt.
- Outils de mesure : `tools/measure-launcher.ps1` (Windows) et `tools/measure-linux.py` (Linux), qui comptent le lanceur et ses processus enfants, pas l'émulateur.
- Comparaison des prototypes WPF, Tauri et Qt Quick avec des mesures réelles sous Linux (`docs/comparison/`).

**Modifications**
- Référence du test CPU recalibrée sur une compilation Release (un cœur Xeon cloud à 2,8 GHz obtient environ 600).
- Prototype Tauri : interrogation de la manette 25 fois moins fréquente pendant un jeu.

## [0.2.0] — 2026-10-22 — Settings and emulators

### English

**Added**
- Full settings screen, usable with a controller: emulator per console (automatic or chosen), emulators (executable, detection, opening the emulator to set it up), game and emulator folders, smoothness check tool, system optimisations, message language.
- Emulators recognised: mGBA, DuckStation, PPSSPP, melonDS and Cemu, in addition to RetroArch, Dolphin and PCSX2. New console: Wii U.
- Custom emulators: any executable, with its consoles and arguments.
- Emulator profiles (TOML) in the data folder, to teach the launcher a new emulator; the new emulators are bundled as profiles.
- Emulator folders: RetroBat's `emulators` folder is searched automatically, and any folder can be added.
- Settings changed by hand inside an emulator are detected and left alone; a button gives them back to the automatic optimisation.
- System optimisations for every emulator: higher process priority and, on Windows machines with two GPUs, the high-performance GPU.
- Native file and folder picker.
- Messages in French or English.
- Guide [EMULATORS.en.md](docs/prototype/EMULATORS.en.md).

**Changed**
- The automatic choice per console takes the first installed emulator in order of preference (for example GBA: mGBA, then RetroArch).
- PCSX2 installed by the user is now optimised too: only the managed keys of `PCSX2.ini` change, and the original file is backed up once.
- The game screen no longer writes any file: settings are applied only when a game starts.
- Built-in themes are updated with the launcher; files edited by the user are kept.
- Pressing A on a game whose emulator is missing opens its settings page.

**Fixed**
- Game scan: emulator, BIOS and save folders are skipped, and loose files are only read directly in a game folder (a RetroBat `emulators` folder used to add hundreds of false PlayStation 2 games).
- Emulator executables or folders added as game folders are moved to the right place at start-up.

### Français

**Ajouts**
- Écran de réglages complet, utilisable à la manette : émulateur par console (automatique ou choisi), émulateurs (exécutable, détection, ouverture de l'émulateur pour le configurer), dossiers de jeux et d'émulateurs, outil de mesure de fluidité, optimisations système, langue des messages.
- Émulateurs reconnus : mGBA, DuckStation, PPSSPP, melonDS et Cemu, en plus de RetroArch, Dolphin et PCSX2. Nouvelle console : Wii U.
- Émulateurs personnalisés : n'importe quel exécutable, avec ses consoles et ses arguments.
- Profils d'émulateurs (TOML) dans le dossier de données, pour apprendre un nouvel émulateur au lanceur ; les nouveaux émulateurs sont fournis sous forme de profils.
- Dossiers d'émulateurs : le dossier `emulators` de RetroBat est cherché automatiquement, et n'importe quel dossier peut être ajouté.
- Les réglages modifiés à la main dans un émulateur sont repérés et laissés tels quels ; un bouton les rend à l'optimisation automatique.
- Optimisations système pour tous les émulateurs : priorité plus élevée et, sous Windows avec deux GPU, le GPU hautes performances.
- Sélecteur natif de fichiers et de dossiers.
- Messages en français ou en anglais.
- Guide [EMULATORS.fr.md](docs/prototype/EMULATORS.fr.md).

**Modifications**
- Le choix automatique par console prend le premier émulateur installé dans l'ordre de préférence (par exemple GBA : mGBA, puis RetroArch).
- PCSX2 installé par l'utilisateur est maintenant optimisé aussi : seules les clés gérées de `PCSX2.ini` changent, et le fichier d'origine est sauvegardé une fois.
- La fiche d'un jeu n'écrit plus aucun fichier : les réglages ne s'appliquent qu'au lancement d'une partie.
- Les thèmes intégrés sont mis à jour avec le lanceur ; les fichiers modifiés par l'utilisateur sont conservés.
- Appuyer sur A sur un jeu dont l'émulateur manque ouvre sa page de réglages.

**Corrections**
- Scan des jeux : les dossiers d'émulateurs, de BIOS et de sauvegardes sont ignorés, et les fichiers isolés ne sont lus que directement dans un dossier de jeux (un dossier `emulators` de RetroBat ajoutait des centaines de faux jeux PlayStation 2).
- Les exécutables ou dossiers d'émulateurs ajoutés comme dossiers de jeux sont remis à la bonne place au démarrage.

## [0.1.0] — 2026-10-22 — Prototype

### English

**Added**
- Desktop application (Tauri 2) for Windows and Linux, usable with a controller, keyboard or mouse.
- Theme system: HTML/CSS/JS folders, `Launcher` API, options declared in `theme.json`, built-in settings panel. Two templates: "Horizon" (default, consoles left to right) and "Grid".
- Controller input through gilrs (SDL database): navigation actions, auto-repeat, input paused while a game runs.
- Game library: EmulationStation-style folders, 14 consoles, title cleaning, covers, favourites, multi-disc `.m3u`.
- Adapters for RetroArch, Dolphin and PCSX2: detection, quality ladders, direct launch.
- Hardware detection (CPU, RAM, GPU through Vulkan) and single-thread CPU test.
- Automatic optimisation: starting estimate, frame-time capture (MangoHud / PresentMon), smoothness verdict and per-game search for the best smooth setting.
- Non-destructive settings: command-line overrides for external installs, INI ownership ledger for managed installs, locked keys.
- Managed emulator updates: auto / notify / off policy, pinned version, two versions kept, rollback; PCSX2 from its official GitHub releases.
- `clctl` command-line tool: probe, add-roms, scan, plan, play, analyse, set-step, emulators, update, rollback.
- Bilingual documentation in `docs/prototype/`, CI on Windows and Linux.
- Downloadable test builds for Windows and Linux, produced by the CI on every push.

**Fixed**
- Windows lint: the settings parse error is boxed, so `cargo clippy -D warnings` passes on Windows too.

**Known limitations**
- Not yet tried with the real emulators or on Windows hardware.
- Estimation thresholds and smoothness criteria are initial values to calibrate.
- No automatic download for Dolphin and RetroArch yet.

### Français

**Ajouts**
- Application de bureau (Tauri 2) pour Windows et Linux, utilisable à la manette, au clavier ou à la souris.
- Système de thèmes : dossiers HTML/CSS/JS, API `Launcher`, options déclarées dans `theme.json`, panneau de réglages intégré. Deux modèles : « Horizon » (par défaut, consoles de gauche à droite) et « Grid ».
- Manettes via gilrs (base SDL) : actions de navigation, répétition automatique, commandes ignorées pendant une partie.
- Bibliothèque : dossiers au format EmulationStation, 14 consoles, titres nettoyés, jaquettes, favoris, multi-disques `.m3u`.
- Adaptateurs RetroArch, Dolphin et PCSX2 : détection, échelles de qualité, lancement direct.
- Détection de la machine (CPU, RAM, GPU via Vulkan) et test CPU monocœur.
- Optimisation automatique : estimation de départ, capture du temps de chaque image (MangoHud / PresentMon), verdict de fluidité et recherche, jeu par jeu, du meilleur réglage fluide.
- Réglages non destructifs : options en ligne de commande pour les installations externes, suivi des clés INI pour les installations gérées, clés verrouillées.
- Mises à jour des émulateurs gérés : politique auto / prévenir / désactivée, version épinglée, deux versions conservées, retour arrière ; PCSX2 depuis ses versions officielles sur GitHub.
- Outil en ligne de commande `clctl` : probe, add-roms, scan, plan, play, analyse, set-step, emulators, update, rollback.
- Documentation bilingue dans `docs/prototype/`, intégration continue sous Windows et Linux.
- Versions de test téléchargeables pour Windows et Linux, compilées par l'intégration continue à chaque envoi.

**Corrections**
- Lint sous Windows : l'erreur de lecture des réglages est mise en boîte, `cargo clippy -D warnings` passe donc aussi sous Windows.

**Limites connues**
- Pas encore essayé avec les vrais émulateurs ni sur une machine Windows.
- Les seuils d'estimation et les critères de fluidité sont des valeurs de départ à calibrer.
- Pas encore de téléchargement automatique pour Dolphin et RetroArch.
