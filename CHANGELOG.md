# Changelog

All notable changes are listed here, newest first. Format based on [Keep a Changelog](https://keepachangelog.com/), versions follow [Semantic Versioning](https://semver.org/).

Toutes les modifications notables sont listées ici, de la plus récente à la plus ancienne.

## [0.3.2] — 2026-10-24 — Library, metadata and desktop foundation

### English

**Added**
- Explicit metadata refresh now downloads exact GameCube/Wii, Nintendo DS and Wii U front covers from GameTDB into a validated offline PNG/JPEG cache. Regional editions are matched by their game id instead of blindly using the first link. The original URL and artwork provider remain recorded, and a missing cover never prevents the game details from being saved.
- Raw Wii U WUD images now expose their embedded four-character product id for exact catalogue matching; compressed WUX/WUA containers remain conservative filename matches until a dedicated reader is added.
- A provider-neutral game metadata foundation: local serial/CRC identity, an offline catalogue cache, rich provenance-aware fields for publisher/date/genres/artwork, and a documented safe model for revision-specific patches. The library already identifies raw GameCube/Wii discs, Nintendo DS files and small cartridge images without relying on filenames.
- A plain desktop interface rebuilt from the original functional workshop replaces the experimental presentation for now: one permanent library workspace with a console catalogue, visible game and folder actions, search, preparation and launch. One conventional Settings entry opens the full-window settings hierarchy; administrative pages no longer compete with the library as top-level tabs. Both theme entries use this same functional base until visual themes are designed later.
- An individual game file can be added from Settings; the launcher asks for its console when several systems share the same extension.
- Windows can install the clearly identified Dolphin 2609a release directly from Dolphin's official server. The archive's SHA-256 digest is checked before extraction and the managed copy is selected automatically.

**Changed**
- The interface never renders a remote artwork URL directly. Covers are shown only after the engine has validated and cached them; a clear retry message replaces the silent empty-cover state when caching fails.
- GameTDB edition selection now understands four-character product codes embedded in longer retail box serials and pairs them with the correct six-character artwork id, preventing mixed region/id URLs such as an Australian folder with a US disc id.
- The Performance page now displays the GPU actually selected for emulation (discrete GPU first) instead of whichever graphics adapter happened to be enumerated first.
- Windows smoothness checks now target the exact emulator process ID instead of every process with the same name, stop when that process exits, and disable unneeded keyboard/mouse input tracking. PresentMon output is retained as a short local diagnostic when no frame capture is produced.
- Dolphin's automatic profile is now stability-first: it never uses Skip Drawing (which can hide objects), no longer forces Dual Core over Dolphin's per-game compatibility choices, and selects a conservative initial graphics backend for Intel, AMD and NVIDIA GPUs. Existing Dolphin tuning profiles are safely recalculated once under this new policy.
- Selecting a game without cached details now starts its metadata and cover lookup automatically after a short delay. Only the selected game is queried; the manual button remains as Refresh/Retry and startup still does not bulk-download the library.
- The left catalogue always shows all 15 supported consoles, whether or not games or emulators are detected. Selecting one now shows its emulator status and the relevant install/download action directly in the right-hand workspace. Emulator installation is no longer presented as a general application setting.
- Settings subpages now have a permanent mouse-accessible Back arrow in the header, and every settings page has a permanent Close button. Keyboard and controller navigation remain unchanged.
- The desktop interface now starts from a separate clean Qt `ApplicationWindow` foundation. Qt owns the menu bar, toolbar, resizable split workspace and status bar, preventing the overlaps and unstable menu placement caused by the former manually-positioned shell. The previous workshop shell is kept only as an unreferenced local fallback.
- The main window now follows the conventional desktop structure: File/Edit/View/Help menu bar, focused command toolbar, library work area and status bar. Duplicate commands were removed from the content, while selection-specific actions stay next to the selected system or game.
- Emulator detection also reuses compatible installations from the previous C# prototype's local data folder.
- The launcher now loads only its QML interface compiled into the executable. External `Main.qml` files are no longer executed or packaged; the engine bridge is private to the built-in interface, and stale asynchronous library responses are ignored.
- Settings now follow a stable hierarchy: application-wide settings (general, controls, library, performance), then consoles, then each console's emulator and tailored behaviour. Diagnostics and emulator management have dedicated sections, and direct links open the relevant console page.

### Français

**Ajouts**
- L'actualisation explicite d'une fiche télécharge aussi la jaquette avant exacte GameCube/Wii, Nintendo DS et Wii U depuis GameTDB dans un cache PNG/JPEG hors ligne vérifié. L'édition régionale est choisie par son identifiant de jeu au lieu de prendre aveuglément le premier lien. L'URL et la source de l'illustration restent enregistrées ; une jaquette absente n'empêche jamais de sauvegarder la fiche.
- Les images disque Wii U brutes au format WUD exposent maintenant leur identifiant produit de quatre caractères pour une correspondance exacte avec le catalogue ; les conteneurs compressés WUX/WUA restent prudemment basés sur le nom jusqu'à l'ajout d'un lecteur dédié.
- Un socle de métadonnées indépendant des fournisseurs : identité locale par série/CRC, cache de catalogue hors ligne, champs riches avec provenance pour éditeur/date/genres/illustrations, et modèle documenté pour des patches sûrs liés à une révision précise. La bibliothèque identifie déjà les disques GameCube/Wii bruts, les fichiers Nintendo DS et les petites cartouches sans dépendre du nom de fichier.
- Une interface de bureau simple, reconstruite à partir de l'atelier fonctionnel d'origine, remplace provisoirement la présentation expérimentale : un espace Bibliothèque permanent avec catalogue des consoles, actions visibles pour les jeux et dossiers, recherche, préparation et lancement. Une seule entrée Réglages ouvre la hiérarchie en plein écran ; les pages administratives ne concurrencent plus la bibliothèque sous forme d'onglets principaux. Les deux entrées de thème utilisent cette même base fonctionnelle jusqu'à la création ultérieure des thèmes visuels.
- Un fichier de jeu précis peut être ajouté depuis les réglages ; le launcher demande sa console quand plusieurs systèmes utilisent la même extension.
- Sous Windows, Dolphin 2609a, clairement identifié, peut être installé directement depuis le serveur officiel de Dolphin. L'empreinte SHA-256 de l'archive est vérifiée avant l'extraction et la copie gérée est sélectionnée automatiquement.

**Modifications**
- L'interface n'affiche jamais directement une adresse d'illustration distante. Une jaquette n'apparaît qu'après validation et mise en cache par le moteur ; si cette étape échoue, un message clair propose de réessayer au lieu de laisser un cadre vide sans explication.
- La sélection d'édition GameTDB reconnaît maintenant les codes produit de quatre caractères inclus dans les longues séries des boîtes et les relie au bon identifiant d'illustration à six caractères. Elle évite ainsi de mélanger, par exemple, un dossier australien avec un identifiant de disque américain.
- La page Performance affiche maintenant le GPU réellement retenu pour l'émulation (carte dédiée en priorité), au lieu du premier adaptateur graphique trouvé dans un ordre technique quelconque.
- Sous Windows, la mesure de fluidité cible maintenant l'identifiant exact du processus d'émulation au lieu de tous les processus portant le même nom, s'arrête avec ce processus et désactive le suivi inutile des entrées clavier/souris. La sortie de PresentMon reste disponible comme court diagnostic local si aucune capture d'images n'est produite.
- Le profil automatique de Dolphin privilégie désormais la stabilité : il n'utilise jamais « Skip Drawing » (qui peut masquer des objets), ne force plus le double cœur par-dessus les choix de compatibilité par jeu de Dolphin et choisit un moteur graphique initial prudent selon le GPU Intel, AMD ou NVIDIA. Les anciens profils Dolphin sont recalculés une fois avec cette nouvelle politique.
- La sélection d'un jeu sans fiche en cache déclenche maintenant automatiquement sa recherche et celle de sa jaquette après un court délai. Seul le jeu sélectionné est interrogé ; le bouton manuel reste disponible pour actualiser/réessayer et le démarrage ne télécharge pas toute la bibliothèque.
- Le catalogue de gauche affiche toujours les 15 consoles prises en charge, même sans jeu ni émulateur détecté. La sélection d’une console affiche désormais à droite l’état de son émulateur et l’action d’installation ou de téléchargement adaptée. L’installation des émulateurs n’est plus présentée comme un réglage général de l’application.
- Les sous-pages des réglages disposent maintenant d'une flèche Retour permanente et utilisable à la souris dans leur en-tête. Toutes les pages des réglages ont aussi un bouton Fermer permanent ; la navigation clavier et manette reste inchangée.
- L'interface de bureau repart sur une fondation Qt `ApplicationWindow` séparée et propre. Qt gère directement la barre de menus, la barre d'outils, l'espace redimensionnable et la barre d'état, ce qui évite les superpositions et le placement instable des menus de l'ancienne coque positionnée à la main. L'ancien atelier reste seulement comme sauvegarde locale non chargée.
- La fenêtre principale suit maintenant la structure classique d'un logiciel de bureau : menus Fichier/Modifier/Affichage/Aide, barre d'outils limitée aux commandes fréquentes, espace Bibliothèque et barre d'état. Les commandes dupliquées ont été retirées du contenu, tandis que les actions liées à une sélection restent près de la console ou du jeu concerné.
- La détection des émulateurs réutilise aussi les installations compatibles présentes dans le dossier local de l'ancien prototype C#.
- Le lanceur charge désormais uniquement son interface QML compilée dans l'exécutable. Les fichiers `Main.qml` externes ne sont plus exécutés ni inclus dans le paquet ; la passerelle du moteur est privée à l'interface intégrée et les réponses asynchrones périmées de la bibliothèque sont ignorées.
- Les réglages suivent maintenant une hiérarchie stable : réglages communs à l'application (général, contrôles, bibliothèque, performance), puis consoles, puis émulateur et comportement adapté à chaque console. Le diagnostic et la gestion des émulateurs ont leurs propres sections, et les raccourcis ouvrent directement la fiche de la console concernée.

## [0.3.1] — 2026-10-23 — Windows start-up fix

### English

**Fixed**
- Windows (and Linux AppImage): the launcher started without opening any window. The built-in QML components (`import ConsoleLauncher`, settings panel) were not shipped with the packages; they are now compiled into the program.

**Added**
- Logs: `console-launcher.log` (interface) and `console-launcher-engine.log` (engine) next to the program, or in the temporary folder when that folder is read-only.
- When a theme cannot be loaded, a window lists the errors instead of a program running with no window.
- CI: both packages are started on Windows and Linux with `--smoke-test` before being published.
- CI: a pre-release `v<version>-qt` is published automatically, with the changelog as notes, when the branch's version has no release yet; no tag to create by hand.

**Changed**
- Controller support (SDL) starts after the window, on its own thread, so a slow device never delays the interface.

### Français

**Corrections**
- Windows (et AppImage Linux) : le lanceur démarrait sans ouvrir de fenêtre. Les composants QML intégrés (`import ConsoleLauncher`, panneau de réglages) n'étaient pas fournis dans les paquets ; ils sont maintenant compilés dans le programme.

**Ajouts**
- Journaux : `console-launcher.log` (interface) et `console-launcher-engine.log` (moteur) à côté du programme, ou dans le dossier temporaire quand ce dossier est en lecture seule.
- Quand un thème ne peut pas être chargé, une fenêtre affiche les erreurs au lieu d'un programme qui tourne sans fenêtre.
- Intégration continue : les deux paquets sont démarrés sous Windows et Linux avec `--smoke-test` avant d'être publiés.
- Intégration continue : une pré-version `v<version>-qt` est publiée automatiquement, avec le changelog en notes, quand la version de la branche n'a pas encore de release ; plus de tag à créer à la main.

**Modifications**
- La prise en charge des manettes (SDL) démarre après la fenêtre, sur son propre fil, pour qu'un périphérique lent ne retarde jamais l'interface.

## [0.3.0] — 2026-10-23 — Qt Quick interface

### English

**Added**
- The Qt Quick application (`qt-prototype/`) is now the interface, with the settings screen of 0.2.0 rebuilt in QML as a built-in component (`import ConsoleLauncher`, `SettingsPanel`) shared by every theme: emulator per console, emulators, game and emulator folders, smoothness check, system optimisations, language, theme.
- Engine linked into the application: new `cl-api` crate (one JSON entry point, documented in [ENGINE_API.en.md](docs/prototype/ENGINE_API.en.md)) and `cl-ffi` crate (C library `console_launcher_engine` with its header). Calls run on worker threads; `clctl` is no longer started for each action.
- Measurement tools shipped with the packages: PresentMon 2.6.0 on Windows, MangoHud 0.8.4 on Linux (injected through `LD_PRELOAD` and a Vulkan layer). A copy installed on the machine still takes precedence.
- Windows: **Allow the measurement** button, which adds the user to the "Performance Log Users" group after an administrator prompt.
- Native file and folder dialogs; pressing A on a game whose emulator is missing opens that emulator's settings page.
- Packages built by the CI: Windows folder (Qt, SDL2, engine, PresentMon) and Linux AppImage (Qt, engine, MangoHud). A `v…` tag publishes them as a GitHub pre-release.
- Guides [ENGINE_API.en.md](docs/prototype/ENGINE_API.en.md), [THEMES.en.md](docs/prototype/THEMES.en.md) rewritten for QML.

**Changed**
- Qt 6.8 (latest long-term support series) is the minimum version.
- Theme and language are stored in `config.toml`; themes follow the language chosen in the settings.
- The Tauri interface (`app/`, `themes/` in HTML) is removed from this branch; it stays on `feat/rust-tauri-prototype`.

### Français

**Ajouts**
- L'application Qt Quick (`qt-prototype/`) devient l'interface, avec l'écran de réglages de la 0.2.0 refait en QML sous forme de composant intégré (`import ConsoleLauncher`, `SettingsPanel`) partagé par tous les thèmes : émulateur par console, émulateurs, dossiers de jeux et d'émulateurs, mesure de fluidité, optimisations système, langue, thème.
- Moteur intégré à l'application : nouvelle crate `cl-api` (un point d'entrée JSON unique, décrit dans [ENGINE_API.fr.md](docs/prototype/ENGINE_API.fr.md)) et crate `cl-ffi` (bibliothèque C `console_launcher_engine` avec son en-tête). Les appels tournent sur des fils de travail ; `clctl` n'est plus lancé à chaque action.
- Outils de mesure fournis avec les paquets : PresentMon 2.6.0 sous Windows, MangoHud 0.8.4 sous Linux (injecté via `LD_PRELOAD` et une couche Vulkan). Une copie installée sur la machine reste prioritaire.
- Windows : bouton **Autoriser la mesure**, qui ajoute l'utilisateur au groupe « Utilisateurs du journal de performances » après une fenêtre administrateur.
- Sélecteurs natifs de fichiers et de dossiers ; appuyer sur A sur un jeu dont l'émulateur manque ouvre la page de réglages de cet émulateur.
- Paquets produits par l'intégration continue : dossier Windows (Qt, SDL2, moteur, PresentMon) et AppImage Linux (Qt, moteur, MangoHud). Un tag `v…` les publie en pré-version sur GitHub.
- Guides [ENGINE_API.fr.md](docs/prototype/ENGINE_API.fr.md) et [THEMES.fr.md](docs/prototype/THEMES.fr.md) réécrits pour QML.

**Modifications**
- Qt 6.8 (dernière série à support long terme) est la version minimale.
- Le thème et la langue sont enregistrés dans `config.toml` ; les thèmes suivent la langue choisie dans les réglages.
- L'interface Tauri (`app/`, `themes/` en HTML) est retirée de cette branche ; elle reste sur `feat/rust-tauri-prototype`.

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
