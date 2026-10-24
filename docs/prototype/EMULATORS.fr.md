# Émulateurs

*[English version](EMULATORS.en.md)*

Comment le lanceur trouve les émulateurs, lesquels il optimise, comment il respecte les réglages modifiés à la main, et comment lui apprendre un nouvel émulateur.

## Émulateurs reconnus

| Émulateur | Consoles | Optimisation automatique | Application des réglages |
|---|---|---|---|
| RetroArch | NES, SNES, N64, Game Boy, GBA, DS, Master System, Mega Drive, PlayStation, PSP | Latence (vidéo sur fil séparé, run-ahead), moteur graphique | Surcharge de session (`--appendconfig`), fichiers intacts |
| Dolphin | GameCube, Wii | Résolution interne, compilation des shaders, moteur graphique | Surcharge de session (`--config`), fichiers intacts |
| PCSX2 | PlayStation 2 | Mise à l'échelle, moteur graphique | `PCSX2.ini` modifié |
| mGBA | Game Boy, Game Boy Color, GBA | Saut d'images (machines très lentes seulement) | Surcharge de session (`-C`), fichiers intacts |
| DuckStation | PlayStation | Résolution interne, moteur graphique | `settings.ini` modifié |
| PPSSPP | PSP | Résolution de rendu | `ppsspp.ini` modifié |
| melonDS | Nintendo DS | Optimisations système seulement | — |
| Cemu | Wii U | Optimisations système seulement | — |

Le lanceur ne fournit jamais d'émulateur, de BIOS, de firmware ni de clé. Les émulateurs viennent de leurs sources officielles, ou sont déjà installés par l'utilisateur.

## Trouver les émulateurs

Sans aucun réglage, le lanceur cherche, dans cet ordre :

1. l'exécutable choisi dans **Réglages › Émulateurs** (un chemin invalide renvoie à la recherche automatique) ;
2. sa propre installation, pour les émulateurs qu'il gère (PCSX2) ;
3. les **dossiers d'émulateurs** : ceux ajoutés dans **Réglages › Dossiers**, le dossier `emulators` à côté d'un dossier `roms` (organisation de RetroBat), et sous Windows `X:\RetroBat\emulators` sur chaque lecteur ;
4. le `PATH`, puis les dossiers d'installation habituels.

**Réglages › Dossiers** range chaque chemin tout seul : dossier de jeux, dossier d'émulateurs ou exécutable d'un émulateur connu. Les chemins ajoutés dans la mauvaise liste par une version précédente sont déplacés au démarrage.

## Quel émulateur pour quelle console

**Réglages › Consoles et émulateurs** montre l'émulateur de chaque console et permet d'en changer avec gauche et droite. « Auto » prend le premier émulateur **installé** dans l'ordre de préférence (par exemple pour la GBA : mGBA, puis RetroArch). Un choix fait à la main est gardé jusqu'à ce qu'on revienne sur « Auto ».

## Réglages modifiés à la main

Le lanceur règle l'émulateur tout seul, jusqu'à ce que l'utilisateur change un réglage dans l'émulateur :

- Avant chaque lancement, le lanceur lit les réglages propres à l'émulateur. Il compare chaque réglage qu'il gère avec la dernière valeur qu'il a vue ou écrite.
- Un réglage qui a changé a été modifié par l'utilisateur : le lanceur n'y touche plus, quel que soit le jeu, et le signale sur la fiche du jeu.
- **Réglages › Émulateurs › (émulateur) › Réactiver l'optimisation automatique** rend ces réglages au lanceur, qui les reprend au lancement suivant.

Quand le lanceur modifie un fichier de réglages (PCSX2, DuckStation, PPSSPP), seules les clés qu'il gère changent. Le fichier d'origine est copié une fois dans `generated/backups/<émulateur>/` du dossier de données, avant la première modification.

## Optimisations système

Elles s'appliquent à tous les émulateurs, y compris ceux que le lanceur ne connaît pas (option **Optimisations système**) :

- **Windows** : l'émulateur démarre avec une priorité plus élevée (« supérieure à la normale »).
- **Windows, machines à deux GPU** (intégré et dédié) : l'émulateur est réglé pour utiliser le GPU hautes performances, sauf si un GPU a déjà été choisi pour lui dans les paramètres graphiques de Windows.

## Ajouter n'importe quel émulateur

**Réglages › Émulateurs › Ajouter un émulateur** demande son exécutable :

- Si c'est un émulateur reconnu (même installé ailleurs), le lanceur l'utilise avec son optimisation automatique.
- Sinon, il devient un **émulateur personnalisé** : on choisit les consoles qu'il lance et, si besoin, ses arguments (`{rom}` représente le fichier du jeu ; par défaut : `{rom}`). Il est lancé directement, avec les optimisations système et la mesure de fluidité. Le lanceur ne change pas ses réglages, car il ne les connaît pas.

## Apprendre un émulateur au lanceur : les fichiers de profil

Un profil est un fichier TOML placé dans le dossier `emulator-profiles` du dossier de données. Il décrit comment trouver l'émulateur, comment lancer un jeu, où sont ses réglages et quels réglages forment chaque palier de qualité. Dès qu'un profil existe, l'émulateur est optimisé comme ceux fournis. Un profil qui porte le même `id` qu'un profil fourni le remplace.

Les profils fournis sont des exemples complets : `crates/cl-adapters/profiles/`.

```toml
id = "mon-emulateur"          # nom simple et unique
name = "Mon Émulateur"
license = "GPL-3.0-or-later"
homepage = "https://example.org"
systems = ["psx"]             # nes snes n64 gb gbc gba nds mastersystem megadrive psx ps2 psp gamecube wii wiiu
gpu_bound = true              # true : les paliers montent la résolution ; false : ils échangent du CPU contre de la latence
args = ["-fullscreen", "{rom}"]

[exe]
windows = ["MonEmu.exe"]
linux = ["monemu"]

[settings]
method = "ini"                # "ini" : fichier modifié ; "cli" : surcharge de session ; "none"
# Du plus précis au plus général. Repères : {exe_dir} {home} {documents} {config} {data} {appdata} {localappdata} {programfiles}
files = ["{exe_dir}/settings.ini", "{config}/monemu/settings.ini"]
# Pour method = "cli" : arguments ajoutés pour chaque réglage, et section INI où l'émulateur les range
# cli = ["-C", "{key}={value}"]
# file_section = "general"

[settings.api]                # facultatif : valeur du réglage de moteur graphique
key = "Video/Renderer"
vulkan = "Vulkan"
opengl = "OpenGL"

# Du plus léger au plus lourd. Clés « Section/Clé » pour les fichiers INI.
[[ladder]]
label = "Native"
set = { "Video/Scale" = "1" }

[[ladder]]
label = "1080p"
set = { "Video/Scale" = "4" }
```

Un fichier illisible est signalé dans **Réglages › Émulateurs**, et les autres profils se chargent quand même.

## Limites connues

- Les lancements et les réglages n'ont été vérifiés que par des tests automatiques, sans les vrais émulateurs. Les noms de clés viennent de la documentation et des fichiers par défaut de chaque projet. Ils restent à confirmer sur de vraies installations.
- melonDS et Cemu : leurs réglages ne sont pas encore optimisés. melonDS a changé de format de fichier avec sa version 1.0, et Cemu règle la résolution par jeu, avec des « graphic packs ».
- RetroBat réécrit certains réglages des émulateurs à chaque fois qu'il lance lui-même un jeu. Le lanceur prend alors ces changements pour des modifications de l'utilisateur et n'y touche plus, jusqu'à ce qu'on les lui rende avec le bouton de réactivation.
