# Console Launcher — prototype v0.3

*[English version](README.en.md)*

Lanceur d'émulation pour Windows et Linux. On choisit une console, puis un jeu, et la partie démarre directement, sans passer par les menus de l'émulateur. Le logiciel règle lui-même les émulateurs selon la puissance de la machine, mesure la fluidité pendant les parties et corrige ses réglages jusqu'à trouver le meilleur compromis. L'interface est faite de thèmes QML que chacun peut modifier ; deux modèles sont fournis.

> Cette branche est un **prototype à tester**. Elle ne remplace rien sur `main`.

L'interface est écrite en C++ / Qt Quick (Qt 6.8 LTS). Le moteur est écrit en Rust et intégré à l'application sous forme de bibliothèque. Une interface précédente, écrite avec Tauri, reste sur la branche `feat/rust-tauri-prototype` ; la comparaison qui a mené à Qt Quick se trouve dans [docs/comparison](../comparison/COMPARAISON.fr.md).

## Ce que fait la v0.3

| Fonction | État |
|---|---|
| Interface pilotable à la manette, au clavier et à la souris | ✅ Testée sous Linux (écran virtuel), pas encore sous Windows |
| Deux thèmes modèles (« Horizon », « Grid ») en QML | ✅ |
| Bibliothèque : dossiers `roms/<console>/`, jaquettes, favoris, jeux multi-disques (.m3u) | ✅ |
| 15 consoles reconnues, 8 émulateurs reconnus (RetroArch, Dolphin, PCSX2, mGBA, DuckStation, PPSSPP, melonDS, Cemu), n'importe quel autre émulateur peut être ajouté | ✅ Commandes et réglages générés et testés, **pas encore essayés avec les vrais émulateurs** |
| Émulateurs trouvés tout seuls, y compris dans le dossier `emulators` de RetroBat ; émulateur choisi par console | ✅ |
| Écran de réglages complet, utilisable à la manette : consoles, émulateurs, dossiers, mesure de fluidité | ✅ Testé sous Linux (écran virtuel) |
| Détection de la machine (CPU, RAM, GPU via Vulkan) et test CPU « simulation » | ✅ |
| Réglage de départ estimé, puis mesure réelle et ajustement automatique par jeu | ✅ Logique testée ; **seuils à calibrer** sur de vraies machines |
| Mesure en conditions réelles, outil fourni avec le lanceur : PresentMon (Windows), MangoHud (Linux) | ⚠️ Intégré, pas encore vérifié sur une vraie partie |
| Mises à jour des émulateurs : auto / prévenir / désactivées, version épinglée, retour arrière | ✅ Pour PCSX2 (GitHub). Dolphin et RetroArch : installation manuelle pour l'instant |
| Les réglages modifiés à la main dans un émulateur ne sont plus touchés, avec un bouton pour les rendre au lanceur | ✅ Fichiers d'origine sauvegardés avant la première modification ([EMULATORS.fr.md](EMULATORS.fr.md)) |
| Optimisations système pour tous les émulateurs (priorité, GPU hautes performances sous Windows) | ✅ Pas encore vérifiées sous Windows |

## Télécharger une version de test

Chaque envoi sur GitHub compile des paquets prêts à l'emploi. Dans l'onglet **Actions** du dépôt, ouvrir la dernière exécution **CI** de la branche, puis télécharger l'artefact en bas de la page (un compte GitHub est nécessaire) :

- `console-launcher-qt-windows-x64` : décompresser, puis lancer `ConsoleLauncher\console-launcher-qt.exe`. Tout le nécessaire est dans le dossier, y compris PresentMon pour la mesure de fluidité.
- `console-launcher-qt-linux-x64` : décompresser, rendre `ConsoleLauncher-x86_64.AppImage` exécutable, puis le lancer. MangoHud est inclus.

**Windows, première mesure de fluidité** : PresentMon a besoin d'une autorisation de Windows. Dans **Réglages › Mesure de fluidité**, choisir **Autoriser la mesure** (une fenêtre administrateur s'ouvre), puis fermer et rouvrir la session Windows. Sans cette autorisation, les jeux se lancent quand même avec les réglages estimés pour la machine.

## Compiler

Voir [qt-prototype/README.fr.md](../../qt-prototype/README.fr.md). En bref : Rust, CMake, Qt 6.8 et SDL2, puis :

```bash
cmake -S qt-prototype -B build-qt -DCMAKE_BUILD_TYPE=Release
cmake --build build-qt
./build-qt/console-launcher-qt
```

L'outil en ligne de commande `clctl` pilote le même moteur sans l'interface :

```bash
cargo run -p cl-cli -- probe               # machine + test CPU
cargo run -p cl-cli -- add-roms D:/Jeux     # dossier de jeux
cargo run -p cl-cli -- scan                 # consoles et jeux trouvés
cargo run -p cl-cli -- plan "nom du jeu"    # ce qui sera lancé, avec quels réglages
cargo run -p cl-cli -- play "nom du jeu"    # jouer, mesurer, ajuster
cargo run -p cl-cli -- analyse capture.csv  # analyser une capture MangoHud / PresentMon
cargo run -p cl-cli -- emulators            # émulateurs détectés
cargo run -p cl-cli -- update pcsx2         # installer / mettre à jour PCSX2 (version officielle)
```

Organisation des jeux (la même que RetroBat, Batocera et EmulationStation, une collection existante fonctionne donc telle quelle) :

```
Jeux/
  snes/  Mon jeu.sfc
  snes/media/covers/Mon jeu.png     (jaquette, facultative)
  ps2/   Autre jeu.iso
  gc/    Encore un.rvz
```

Réglages : `config.toml` dans le dossier de données (`%APPDATA%\ConsoleLauncher\console-launcher\data` sous Windows, `~/.local/share/console-launcher` sous Linux, ou le dossier indiqué par la variable `CONSOLE_LAUNCHER_HOME`). Ils se modifient depuis l'écran de réglages ; un exemple commenté se trouve dans [config.example.toml](config.example.toml).

### Commandes

| Manette | Clavier | Action |
|---|---|---|
| Croix / stick gauche | Flèches | Se déplacer |
| A (bas) | Entrée | Valider / jouer |
| B (droite) | Échap | Retour |
| Y (haut) | F | Favori |
| Start / Select | Tab | Réglages |
| LB / RB | Page préc. / suiv. | Sauter (10 jeux ou une console) |
| — | F11 | Plein écran |

Toutes les manettes de la base SDL fonctionnent : Xbox, PlayStation, Switch Pro et la plupart des manettes génériques.

## Architecture

```
qt-prototype/          Application C++ / Qt Quick
  src/                 fenêtre, manettes (SDL2), lien avec le moteur
  ui/                  composants QML intégrés (panneau de réglages)
  themes/              thèmes QML, copiés à côté de l'exécutable
crates/
  cl-core              Modèle commun, config.toml, éditeur INI non destructif, détection des modifications
  cl-hwprobe           Détection de la machine + test CPU (simulation)
  cl-library           Scan des jeux, jaquettes, favoris
  cl-adapters          Adaptateurs et profils d'émulateurs (détection, réglages, commande)
    profiles/          Profils fournis (mGBA, DuckStation, PPSSPP, melonDS, Cemu)
  cl-perf              Estimation, analyse des captures, ajustement par jeu
  cl-session           Lancement, capture des images, optimisations système, attente de fermeture
  cl-updater           Installations gérées : mises à jour, épinglage, retour arrière
  cl-engine            Assemble le tout
  cl-api               Un point d'entrée JSON unique sur le moteur
  cl-ffi               Bibliothèque C chargée par l'application (console_launcher_engine)
  cl-cli               clctl
```

L'interface ne connaît aucun émulateur : elle demande des consoles et des jeux, puis appelle `play`. Les méthodes du moteur sont listées dans [ENGINE_API.fr.md](ENGINE_API.fr.md). Pour ajouter un émulateur, il suffit d'un fichier de profil (TOML) ou, pour les cas particuliers, d'un adaptateur dans `cl-adapters`, sans toucher aux thèmes. Voir [EMULATORS.fr.md](EMULATORS.fr.md).

## Ce qui n'est pas fourni, et ne le sera pas

Aucune ROM, aucun BIOS, aucun firmware ni aucune clé. Chacun utilise des fichiers extraits de son propre matériel. Les émulateurs sont téléchargés depuis leurs sources officielles, jamais redistribués avec le lanceur. Seuls les deux outils de mesure, PresentMon et MangoHud (MIT), sont fournis avec les paquets.

## Licence

Elle reste à choisir. Un point important : une licence qui **interdit** l'usage commercial (CC BY-NC, PolyForm Noncommercial…) n'est pas considérée comme « open source » au sens de l'OSI. Elle est aussi incompatible avec le code GPL, si un jour du code d'émulateur est intégré. La proposition est donc la **GPL-3.0-or-later** : le code reste libre, et toute version modifiée et distribuée doit publier ses sources. Les dépendances actuelles (Qt sous LGPL-3.0, MIT, Apache-2.0, MPL-2.0, Zlib…) sont toutes compatibles avec elle.

## Suite proposée

1. Essais sur de vraies machines Windows et Linux avec les vrais émulateurs, puis calibration des seuils ([PERFORMANCE.fr.md](PERFORMANCE.fr.md)).
2. Optimisation des réglages de melonDS (format 1.x) et de Cemu (graphic packs).
3. Sources de mise à jour pour Dolphin et RetroArch, et installation des cores RetroArch manquants.
4. Choix manuel de la qualité par jeu et options du thème dans l'écran de réglages.
5. Récupération automatique des jaquettes (ScreenScraper, compte de l'utilisateur).
6. Installateur Windows.

Autres documents : [EMULATORS.fr.md](EMULATORS.fr.md) · [THEMES.fr.md](THEMES.fr.md) · [ENGINE_API.fr.md](ENGINE_API.fr.md) · [PERFORMANCE.fr.md](PERFORMANCE.fr.md) · [DEPENDENCIES.fr.md](DEPENDENCIES.fr.md) · [CHANGELOG](../../CHANGELOG.md)
