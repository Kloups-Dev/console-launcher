# Console Launcher — contexte et suivi commun

Ce document est la référence commune pour les humains et leurs IA.

## Le projet

Un logiciel d'émulation à installer sur PC, avec un menu original, joli et fluide, utilisable à la manette. Consoles accessibles de gauche à droite, puis sélection des jeux. La PS3 est un exemple de simplicité de navigation, pas une interface à reproduire.

Parcours : **menu → console → jeu → partie → retour au menu**. Les menus et bibliothèques des émulateurs ne sont pas une étape visible. Les réglages restent accessibles et compréhensibles : il ne s'agit pas de supprimer tous les paramètres.

Réutiliser les émulateurs existants. Intention de publier en open source ; nom définitif et licence à choisir. Aucun jeu ou BIOS fourni. Pas d'objectif de réseau social demandé. Aucun gain de performance encore démontré.

## Fonctionnement entre IA

- Au début de chaque nouvelle phase de code, synchroniser le dépôt, lire ce document et consulter les changements des autres branches utiles. Signaler si l'accès réseau empêche d'actualiser le contexte.
- Chaque IA est indépendante dans le périmètre fixé par les humains. Aucune IA ne dirige l'autre, ne lui attribue de tâche ou de fichiers, ni ne lui impose une technologie ou une voie de développement.
- Les choix existants décrivent le travail réalisé ; ils peuvent être améliorés ou remis en question. Préserver les changements d'autrui et signaler les conflits.
- Après chaque push de code, mettre à jour l'état et ajouter une courte entrée au journal. Envoyer cette mise à jour dans un petit commit de suivi. Un push de suivi documentaire ne demande pas une nouvelle entrée : pas de boucle.
- Indiquer date, branche, commit de code, changements, vérifications réellement faites et limites. Distinguer travail envoyé, fusionné et encore local. Ne pas écrire de secrets ici.
- Garder l'état actuel court ; le détail historique reste dans Git et les pull requests. Si deux IA changent le journal, conserver les deux contributions lors de la résolution du conflit.

## État actuel

- Dépôt : https://github.com/Kloups-Dev/console-launcher. Cykalos est administrateur du dépôt.
- Prototype proposé dans la [PR n°1](https://github.com/Kloups-Dev/console-launcher/pull/1), branche `feat/desktop-foundation`, pas encore fusionné à cette mise à jour.
- Prototype : Windows, C#/.NET 8 et WPF ; cœur indépendant de l'interface. Menu horizontal, bibliothèque JSON, navigation clavier/XInput, sélection de Dolphin et ajout d'un fichier GameCube pour lancement direct puis retour au menu.
- Limites : présentation provisoire ; PS2/SNES en aperçu ; titres fictifs. Dolphin doit être installé et ses contrôles configurés. Dialogues encore dépendants du clavier/souris. Pas d'installateur autonome.
- Vérifications : compilation Release sans erreur ni avertissement, 23 contrôles locaux réussis, contrôles GitHub réussis avant simplification documentaire, rendu WPF inspecté. Aucun essai avec vrai jeu/manette physique, ni benchmark.
- Second prototype sur la branche `feat/rust-tauri-prototype` (Windows et Linux, Rust + Tauri 2, v0.2.0) dans `crates/`, `app/` et `themes/`, sans fichier commun avec la PR n°1 : thèmes HTML/CSS/JS modifiables, écran de réglages complet à la manette, 8 émulateurs reconnus (RetroArch, Dolphin, PCSX2, mGBA, DuckStation, PPSSPP, melonDS, Cemu) et émulateurs personnalisés, optimisation automatique par jeu (estimation puis mesure de fluidité) qui laisse tranquilles les réglages modifiés à la main, mises à jour des émulateurs avec retour arrière. Détails : `docs/prototype/README.fr.md`. Les deux voies techniques restent à comparer avant d'en fusionner une sur `main`.

## Essayer le prototype

Ces commandes concernent la branche du prototype. Prérequis : Windows et SDK .NET 8, version 8.0.300 ou ultérieure dans la série 8.

```powershell
dotnet run --project src/Launcher.Desktop/Launcher.Desktop.csproj
dotnet build src/Launcher.Desktop/Launcher.Desktop.csproj --configuration Release
dotnet run --project tests/Launcher.Checks/Launcher.Checks.csproj --configuration Release
```

Flèches/XInput pour naviguer ; Entrée/A pour choisir ; Échap/B pour revenir ; F11 pour le plein écran. Dans Réglages, sélectionner Dolphin.exe, ajouter son fichier GameCube puis jouer. Fermer la partie dans Dolphin pour revenir.

Bibliothèque : `%LOCALAPPDATA%/ConsoleLauncher/library.json`. Les jeux ne sont ni copiés ni déplacés ; les réglages et sauvegardes Dolphin existants sont utilisés. Le dossier portable nécessite .NET Desktop Runtime 8.

## Journal partagé

### 2026-10-22 — Premier socle publié

- Code : `7b52097`, branche `feat/desktop-foundation`, PR n°1 ouverte.
- Fait : prototype, navigation, stockage et premier adaptateur Dolphin ; vérifications et limites ci-dessus.
- Suivi : `fc67ebb` retire les attributions de rôles. Les choix techniques restent révisables.
- Ouvert : essais réels, contrôles en jeu, présentation et mesures de fluidité. Aucune attribution à une IA.

### 2026-10-22 — Suivi documentaire simplifié

- Fait : un seul document général, suppression des anciens guides et consignes redondantes.
- Vérifié : renvois documentaires et cohérence du contenu ; aucun code applicatif modifié.
- Publication : simplification fusionnée sur `main` par la PR n°3. Le prototype a été aligné dans `a2f635d` ; sa PR n°1 reste ouverte.

### 2026-10-22 — Second prototype (Rust + Tauri) sur une branche dédiée

- Code : branche `feat/rust-tauri-prototype`, à comparer avec la PR n°1 ; `main` n'est pas modifié.
- Fait : application Tauri, moteur en crates Rust, deux thèmes modèles, outil `clctl`, CI Windows/Linux avec exécutables de test téléchargeables, documentation FR/EN dans `docs/prototype/`, `CHANGELOG.md`.
- Vérifié : 47 tests, `cargo clippy -D warnings` et `cargo fmt` sans erreur sous Linux ; interface essayée sur écran virtuel Linux avec de faux émulateurs (navigation, menu, thèmes, lancement).
- Limites : aucun essai sous Windows, avec un vrai émulateur, une vraie manette ou une vraie mesure de fluidité ; seuils d'optimisation à calibrer. Aucun fichier de la PR n°1 modifié ; seul `.gitignore` a reçu deux lignes (`/target/`, `/app/src-tauri/gen/`).

### 2026-10-22 — Correction du lint Windows (Rust + Tauri)

- Code : `c642e1e`, branche `feat/rust-tauri-prototype`, envoyé.
- Fait : l'erreur de lecture de `config.toml` est mise en boîte ; `clippy::result_large_err` bloquait la CI Windows.
- Vérifié : `cargo fmt`, `cargo clippy -D warnings` et tests de `cl-core` sous Linux ; résultat Windows donné par la CI.
- Limites : l'exécutable de test Windows a été lancé par un humain ; RetroArch absent, donc aucune partie lancée.

### 2026-10-22 — Réglages complets et nouveaux émulateurs (Rust + Tauri, v0.2.0)

- Code : `3cf24ae`, branche `feat/rust-tauri-prototype`, envoyé.
- Fait : écran de réglages à la manette (émulateur par console, émulateurs, dossiers, mesure, optimisations système) ; profils TOML (mGBA, DuckStation, PPSSPP, melonDS, Cemu) et émulateurs personnalisés ; console Wii U ; dossiers d'émulateurs dont celui de RetroBat ; détection des réglages modifiés à la main avec bouton de réactivation et sauvegarde des INI ; priorité et GPU dédié sous Windows ; scan qui ignore les dossiers d'émulateurs ; doc FR/EN `docs/prototype/EMULATORS.*.md`.
- Vérifié : 64 tests, `cargo clippy -D warnings` et `cargo fmt` sous Linux ; CI Windows réussie (compilation, lint, tests, paquet) ; parcours essayés sur écran virtuel Linux avec une fausse installation RetroBat (migration des chemins, choix automatique, modification à la main puis réactivation). Le job Linux de la CI était bloqué sur l'installation des paquets de GitHub au moment de cette entrée.
- Limites : aucun essai avec les vrais émulateurs ni sous Windows réel ; noms de clés des nouveaux émulateurs à confirmer ; melonDS et Cemu sans réglages optimisés ; PresentMon à installer à part pour la mesure sous Windows.
