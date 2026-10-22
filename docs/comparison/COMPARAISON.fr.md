# Comparaison des prototypes d'interface

*[English version](COMPARISON.en.md)*

But : choisir la base technique du projet sur des résultats réels. Les critères sont un design très poussé, des thèmes personnalisables, Windows d'abord puis Linux, et une consommation minimale pendant les jeux.

## Les trois pistes

| | WPF | Tauri | Qt Quick |
|---|---|---|---|
| Emplacement | branche `feat/desktop-foundation` (PR n°1) | `app/`, `themes/` | `qt-prototype/` |
| Langage | C# / .NET 8 | Rust + HTML/CSS/JS | C++17 + QML |
| Rendu | DirectX (WPF) | WebView2 (Windows), WebKitGTK (Linux) | Scene graph GPU : Direct3D 11 (Windows), OpenGL/Vulkan (Linux) |
| Thèmes | XAML compilé dans l'application | Dossiers HTML/CSS/JS, rechargeables (F5) | Dossiers QML lus au démarrage |
| Manettes | XInput (manettes Xbox et compatibles) | gilrs (base SDL) | SDL2 (base SDL) |
| Moteur | Propre au prototype (Dolphin) | Moteur Rust intégré | Même moteur Rust, via `clctl` |
| Linux | Non (Windows uniquement) | Oui | Oui |
| Paquet Windows | Nécessite le .NET Desktop Runtime 8, ou ~150 Mo en autonome* | 18 Mo (WebView2 fourni par Windows) | 42 Mo (DLL Qt incluses) |

\* Ordre de grandeur habituel d'une application WPF autonome, non mesuré sur ce prototype.

## Mesures réelles sous Linux

Conditions : machine virtuelle à 2 cœurs Xeon 2,8 GHz, écran virtuel sans carte graphique (rendu logiciel), compilation Release, même bibliothèque de 11 jeux, émulateur factice. Mesure sur 30 s, le lanceur et tous ses processus enfants comptés, l'émulateur exclu (`tools/measure-linux.py`). Deux séries par prototype.

| | Tauri | Qt Quick |
|---|---|---|
| Démarrage (fenêtre visible) | 0,52 s (2,5 s au premier lancement à froid) | 0,52 s |
| Mémoire au menu (PSS) | **204–206 Mo**, 3 processus | **51–52 Mo**, 1 processus |
| Processeur au menu | 1,6–2,9 % d'un cœur | 1,4–1,5 % d'un cœur |
| Mémoire pendant un jeu | 206–207 Mo | 55–56 Mo (avec `clctl` en attente) |
| Processeur pendant un jeu | 0,9–1,0 % d'un cœur | **0,0 %** |

Le prototype WPF ne peut pas tourner sous Linux et n'a donc pas été mesuré.

À lire avec prudence : sans carte graphique, le processeur fait aussi le rendu, ce qui gonfle les chiffres « au menu ». Les écarts de mémoire et de consommation pendant un jeu, eux, viennent de la technologie et devraient se retrouver sous Windows, où WebView2 lance lui aussi plusieurs processus.

## Mesures à faire sous Windows (le vrai test)

Le script `tools/measure-launcher.ps1` mesure les trois prototypes de la même façon : temps d'ouverture, mémoire privée, processeur et carte graphique, au menu puis pendant un jeu. Il additionne tous les processus du lanceur et exclut l'émulateur.

```powershell
powershell -ExecutionPolicy Bypass -File measure-launcher.ps1 -Exe .\ConsoleLauncher-Tauri\console-launcher.exe -Name tauri
powershell -ExecutionPolicy Bypass -File measure-launcher.ps1 -Exe .\ConsoleLauncher-Qt\console-launcher-qt.exe -Name qt
powershell -ExecutionPolicy Bypass -File measure-launcher.ps1 -Exe <dossier WPF>\ConsoleLauncher.exe -Name wpf
```

Pour une comparaison juste : même PC, même écran, même jeu et même émulateur, aucun autre programme lourd ouvert. Lancer chaque mesure deux fois et garder la seconde.

## Ce que les mesures ne disent pas

| Critère | WPF | Tauri | Qt Quick |
|---|---|---|---|
| Liberté de design | Très grande (XAML, animations, styles) | Très grande (tout le web : CSS, animations, WebGL) | Très grande (animations GPU, shaders, particules) |
| Facilité de créer un thème | Moyenne : XAML, rechargement à chaud peu courant | **La plus grande** : HTML/CSS, connus de beaucoup de monde | Bonne : QML est déclaratif et lisible, mais moins connu que le web |
| Fluidité des animations | Bonne | Bonne sous Windows ; WebKitGTK plus inégal sous Linux | **Excellente** sur les deux systèmes (rendu GPU natif) |
| Linux plus tard | Réécriture nécessaire (Avalonia, par exemple) | Oui, avec un moteur web différent (WebKitGTK) | Oui, même rendu |
| Difficulté de développement | Faible à moyenne | Moyenne (Rust + web) | Plus élevée (C++, outils Qt) |
| Licence | MIT (.NET) | MIT / Apache-2.0 | LGPL-3.0 (liaison dynamique, compatible open source) |
| Exemples existants | Playnite (lanceur de jeux en WPF) | — | Pegasus Frontend (lanceur d'émulateurs en QML, thèmes communautaires) |

## Lecture provisoire

- **Moteur** : le moteur Rust (bibliothèque, réglages, optimisation, mises à jour) fonctionne déjà avec deux interfaces différentes. Il peut servir de base quelle que soit l'interface retenue.
- **Consommation** : Qt Quick est nettement devant sous Linux, avec environ 4 fois moins de mémoire et rien de mesurable pendant un jeu. C'est le critère « consommer peu pendant les jeux ».
- **Thèmes** : Tauri reste le plus accessible pour les créateurs de thèmes, avec du HTML/CSS ordinaire.
- **WPF** : il bloque l'objectif Linux sans réécriture de l'interface.

Le choix final se fera sur les chiffres Windows des trois prototypes, mesurés avec le script ci-dessus.
