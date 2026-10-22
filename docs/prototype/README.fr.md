# Console Launcher — prototype v0.2

*[English version](README.en.md)*

Lanceur d'émulation pour Windows et Linux. On choisit une console, puis un jeu, et la partie démarre directement, sans passer par les menus de l'émulateur. Le logiciel règle lui-même les émulateurs selon la puissance de la machine, mesure la fluidité pendant les parties et corrige ses réglages jusqu'à trouver le meilleur compromis. L'interface est un thème HTML/CSS/JS entièrement modifiable, et deux modèles sont fournis comme point de départ.

> Cette branche est un **prototype à tester**. Elle ne remplace rien sur `main`.

## Ce que fait la v0.2

| Fonction | État |
|---|---|
| Interface pilotable à la manette, au clavier et à la souris | ✅ Testée sous Linux (écran virtuel), pas encore sous Windows |
| Deux thèmes modèles (« Horizon », « Grid ») et API de thème | ✅ |
| Bibliothèque : dossiers `roms/<console>/`, jaquettes, favoris, jeux multi-disques (.m3u) | ✅ |
| 15 consoles reconnues, 8 émulateurs reconnus (RetroArch, Dolphin, PCSX2, mGBA, DuckStation, PPSSPP, melonDS, Cemu), n'importe quel autre émulateur peut être ajouté | ✅ Commandes et réglages générés et testés, **pas encore essayés avec les vrais émulateurs** |
| Émulateurs trouvés tout seuls, y compris dans le dossier `emulators` de RetroBat ; émulateur choisi par console | ✅ |
| Écran de réglages complet, utilisable à la manette : consoles, émulateurs, dossiers, mesure de fluidité | ✅ Testé sous Linux (écran virtuel) |
| Détection de la machine (CPU, RAM, GPU via Vulkan) et test CPU « simulation » | ✅ |
| Réglage de départ estimé, puis mesure réelle et ajustement automatique par jeu | ✅ Logique testée ; **seuils à calibrer** sur de vraies machines |
| Mesure en conditions réelles : MangoHud (Linux), PresentMon (Windows) | ⚠️ Intégré, pas encore vérifié sur une vraie partie |
| Mises à jour des émulateurs : auto / prévenir / désactivées, version épinglée, retour arrière | ✅ Pour PCSX2 (GitHub). Dolphin et RetroArch : installation manuelle pour l'instant |
| Les réglages modifiés à la main dans un émulateur ne sont plus touchés, avec un bouton pour les rendre au lanceur | ✅ Fichiers d'origine sauvegardés avant la première modification ([EMULATORS.fr.md](EMULATORS.fr.md)) |
| Optimisations système pour tous les émulateurs (priorité, GPU hautes performances sous Windows) | ✅ Pas encore vérifiées sous Windows |

## Télécharger une version de test

Chaque envoi sur GitHub compile des exécutables prêts à l'emploi. Dans l'onglet **Actions** du dépôt, ouvrir la dernière exécution **CI** de la branche, puis télécharger l'artefact en bas de la page (un compte GitHub est nécessaire) :

- `console-launcher-windows-x64` : décompresser, puis lancer `console-launcher.exe`. WebView2 est nécessaire (déjà présent dans Windows 10 et 11).
- `console-launcher-linux-x64` : décompresser, rendre `console-launcher` exécutable, puis le lancer. Les bibliothèques WebKitGTK listées dans [DEPENDENCIES.fr.md](DEPENDENCIES.fr.md) sont nécessaires.

L'outil en ligne de commande `clctl` est dans la même archive.

## Démarrage rapide

Prérequis : [Rust](https://rustup.rs) stable. Sous Linux, il faut aussi les bibliothèques de Tauri (voir [DEPENDENCIES.fr.md](DEPENDENCIES.fr.md)).

```bash
# Interface
cargo run -p console-launcher

# Outil en ligne de commande, pour tester le moteur sans l'interface
cargo run -p cl-cli -- probe               # machine + test CPU
cargo run -p cl-cli -- add-roms D:/Jeux     # dossier de jeux
cargo run -p cl-cli -- scan                 # consoles et jeux trouvés
cargo run -p cl-cli -- plan "nom du jeu"    # ce qui sera lancé, avec quels réglages
cargo run -p cl-cli -- play "nom du jeu"    # jouer, mesurer, ajuster
cargo run -p cl-cli -- analyse capture.csv  # analyser une capture MangoHud / PresentMon
cargo run -p cl-cli -- emulators            # émulateurs détectés
cargo run -p cl-cli -- update pcsx2         # installer / mettre à jour PCSX2 (version officielle)
```

Organisation des jeux (la même que RetroBat, Batocera et EmulationStation, donc une collection existante fonctionne telle quelle) :

```
Jeux/
  snes/  Mon jeu.sfc
  snes/media/covers/Mon jeu.png     (jaquette, facultative)
  ps2/   Autre jeu.iso
  gc/    Encore un.rvz
```

Réglages : `config.toml` dans le dossier de données (`%APPDATA%\ConsoleLauncher\console-launcher\data` sous Windows, `~/.local/share/console-launcher` sous Linux, ou le dossier indiqué par la variable `CONSOLE_LAUNCHER_HOME`). Un exemple commenté se trouve dans [config.example.toml](config.example.toml).

### Commandes manette

| Manette | Clavier | Action |
|---|---|---|
| Croix / stick gauche | Flèches | Naviguer |
| A (bas) | Entrée | Valider / jouer |
| B (droite) | Échap | Retour |
| Y (haut) | F | Favori |
| Start / Select | Tab | Menu des réglages |
| LT / RT | Page préc. / suiv. | Sauter (10 jeux ou une console) |

Toutes les manettes reconnues par la base SDL fonctionnent : Xbox, PlayStation, Switch Pro et la plupart des manettes génériques.

## Architecture

```
app/                  Application Tauri : fenêtre, manette, chargement des thèmes
  ui/sdk.js           API `Launcher` utilisée par les thèmes
themes/               Thèmes modèles (copiés dans le dossier de données au 1er lancement)
crates/
  cl-core             Modèle commun, config.toml, éditeur INI non destructif
  cl-hwprobe          Détection matériel + test CPU (simulation)
  cl-library          Scan des jeux, jaquettes, favoris
  cl-adapters         Adaptateurs et profils d'émulateurs (détection, réglages, commande)
    profiles/         Profils fournis (mGBA, DuckStation, PPSSPP, melonDS, Cemu)
  cl-perf             Estimation, analyse des captures, ajustement par jeu
  cl-session          Lancement, capture des images, attente de fermeture
  cl-updater          Installations gérées : mises à jour, épinglage, retour arrière
  cl-engine           Assemble le tout pour l'interface et pour clctl
  cl-cli              clctl
```

L'interface ne connaît aucun émulateur : elle demande des consoles et des jeux, puis appelle `play`. Pour ajouter un émulateur, il suffit d'un fichier de profil (TOML) ou, pour les cas particuliers, d'un adaptateur dans `cl-adapters`, sans toucher aux thèmes. Voir [EMULATORS.fr.md](EMULATORS.fr.md).

## Ce qui n'est pas fourni, et ne le sera pas

Aucune ROM, aucun BIOS, aucun firmware ni aucune clé. Chacun utilise des fichiers extraits de son propre matériel. Les émulateurs sont téléchargés depuis leurs sources officielles, jamais redistribués avec le lanceur.

## Licence

Elle reste à choisir. Un point important : une licence qui **interdit** l'usage commercial (CC BY-NC, PolyForm Noncommercial…) n'est pas considérée comme « open source » au sens de l'OSI. Elle est aussi incompatible avec le code GPL, si un jour du code d'émulateur est intégré. La proposition est donc la **GPL-3.0-or-later** : le code reste libre, et toute version modifiée et distribuée doit publier ses sources. Les dépendances actuelles (MIT, Apache-2.0, MPL-2.0…) sont toutes compatibles avec elle.

## Suite proposée

1. Essais sur de vraies machines Windows et Linux avec les vrais émulateurs, puis calibration des seuils ([PERFORMANCE.fr.md](PERFORMANCE.fr.md)).
2. Optimisation des réglages de melonDS (format 1.x) et de Cemu (graphic packs).
3. Sources de mise à jour pour Dolphin et RetroArch, et installation des cores RetroArch manquants.
4. Choix manuel de la qualité par jeu et options du thème dans l'écran de réglages.
5. Récupération automatique des jaquettes (ScreenScraper, compte de l'utilisateur).
6. Installateurs (NSIS pour Windows, AppImage et .deb pour Linux) via `cargo tauri build`.

Autres documents : [EMULATORS.fr.md](EMULATORS.fr.md) · [THEMES.fr.md](THEMES.fr.md) · [PERFORMANCE.fr.md](PERFORMANCE.fr.md) · [DEPENDENCIES.fr.md](DEPENDENCIES.fr.md) · [CHANGELOG](../../CHANGELOG.md)
