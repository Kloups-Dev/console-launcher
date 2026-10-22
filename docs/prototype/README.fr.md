# Console Launcher — prototype v0.1

*[English version](README.en.md)*

Lanceur d'émulation pour Windows et Linux. On choisit une console, puis un jeu, et la partie démarre directement, sans passer par les menus de l'émulateur. Le logiciel règle lui-même les émulateurs selon la puissance de la machine, mesure la fluidité pendant les parties et corrige ses réglages jusqu'à trouver le meilleur compromis. L'interface est un thème HTML/CSS/JS entièrement modifiable, et deux modèles sont fournis comme point de départ.

> Cette branche est un **prototype à tester**. Elle ne remplace rien sur `main`.

## Ce que fait la v0.1

| Fonction | État |
|---|---|
| Interface pilotable à la manette, au clavier et à la souris | ✅ Testée sous Linux (écran virtuel), pas encore sous Windows |
| Deux thèmes modèles (« Horizon », « Grid ») et API de thème | ✅ |
| Bibliothèque : dossiers `roms/<console>/`, jaquettes, favoris, jeux multi-disques (.m3u) | ✅ |
| 14 consoles reconnues, 3 émulateurs intégrés (RetroArch, Dolphin, PCSX2) | ✅ Commandes générées et testées, **pas encore essayées avec les vrais émulateurs** |
| Détection de la machine (CPU, RAM, GPU via Vulkan) et test CPU « simulation » | ✅ |
| Réglage de départ estimé, puis mesure réelle et ajustement automatique par jeu | ✅ Logique testée ; **seuils à calibrer** sur de vraies machines |
| Mesure en conditions réelles : MangoHud (Linux), PresentMon (Windows) | ⚠️ Intégré, pas encore vérifié sur une vraie partie |
| Mises à jour des émulateurs : auto / prévenir / désactivées, version épinglée, retour arrière | ✅ Pour PCSX2 (GitHub). Dolphin et RetroArch : installation manuelle pour l'instant |
| Respect des réglages de l'utilisateur | ✅ Les fichiers d'un émulateur installé par l'utilisateur ne sont jamais modifiés |

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
  cl-adapters         Un adaptateur par émulateur (détection, réglages, commande)
  cl-perf             Estimation, analyse des captures, ajustement par jeu
  cl-session          Lancement, capture des images, attente de fermeture
  cl-updater          Installations gérées : mises à jour, épinglage, retour arrière
  cl-engine           Assemble le tout pour l'interface et pour clctl
  cl-cli              clctl
```

L'interface ne connaît aucun émulateur : elle demande des consoles et des jeux, puis appelle `play`. Pour ajouter un émulateur, il suffit d'écrire un nouvel adaptateur dans `cl-adapters`, sans toucher aux thèmes.

## Ce qui n'est pas fourni, et ne le sera pas

Aucune ROM, aucun BIOS, aucun firmware ni aucune clé. Chacun utilise des fichiers extraits de son propre matériel. Les émulateurs sont téléchargés depuis leurs sources officielles, jamais redistribués avec le lanceur.

## Licence

Elle reste à choisir. Un point important : une licence qui **interdit** l'usage commercial (CC BY-NC, PolyForm Noncommercial…) n'est pas considérée comme « open source » au sens de l'OSI. Elle est aussi incompatible avec le code GPL, si un jour du code d'émulateur est intégré. La proposition est donc la **GPL-3.0-or-later** : le code reste libre, et toute version modifiée et distribuée doit publier ses sources. Les dépendances actuelles (MIT, Apache-2.0, MPL-2.0…) sont toutes compatibles avec elle.

## Suite proposée

1. Essais sur de vraies machines Windows et Linux avec les trois émulateurs, puis calibration des seuils ([PERFORMANCE.fr.md](PERFORMANCE.fr.md)).
2. Sources de mise à jour pour Dolphin et RetroArch, et installation des cores RetroArch manquants.
3. Écran de réglages complet : choix de l'émulateur par console, réglage manuel de la qualité, options du thème.
4. Récupération automatique des jaquettes (ScreenScraper, compte de l'utilisateur).
5. Nouveaux adaptateurs : DuckStation, PPSSPP, melonDS, Cemu, RPCS3…
6. Installateurs (NSIS pour Windows, AppImage et .deb pour Linux) via `cargo tauri build`.

Autres documents : [THEMES.fr.md](THEMES.fr.md) · [PERFORMANCE.fr.md](PERFORMANCE.fr.md) · [DEPENDENCIES.fr.md](DEPENDENCIES.fr.md) · [CHANGELOG](../../CHANGELOG.md)
