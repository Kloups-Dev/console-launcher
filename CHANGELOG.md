# Changelog

All notable changes are listed here, newest first. Format based on [Keep a Changelog](https://keepachangelog.com/), versions follow [Semantic Versioning](https://semver.org/).

Toutes les modifications notables sont listées ici, de la plus récente à la plus ancienne.

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

**Limites connues**
- Pas encore essayé avec les vrais émulateurs ni sur une machine Windows.
- Les seuils d'estimation et les critères de fluidité sont des valeurs de départ à calibrer.
- Pas encore de téléchargement automatique pour Dolphin et RetroArch.
