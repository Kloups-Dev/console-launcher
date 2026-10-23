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
- Premier prototype Windows : [PR n°1](https://github.com/Kloups-Dev/console-launcher/pull/1), branche `feat/desktop-foundation`, encore ouverte lors de cette mise à jour.
- Atelier fonctionnel : [PR n°4](https://github.com/Kloups-Dev/console-launcher/pull/4), branche `feat/functional-workbench`, proposée sur la branche du premier prototype, pas fusionnée. L'autre prototype Rust/Tauri reste une proposition distincte.
- Installation Dolphin : [PR n°5](https://github.com/Kloups-Dev/console-launcher/pull/5), branche `feat/dolphin-installation`, proposée sur `feat/functional-workbench`, pas fusionnée.
- Contrôles GameCube clavier/manette : [PR n°6](https://github.com/Kloups-Dev/console-launcher/pull/6), branche `feat/dolphin-keyboard`, proposée sur `feat/dolphin-installation`, pas fusionnée.
- Atelier : C#/.NET 8, WPF pour l'écran de travail ; cœur indépendant de WPF. Listes et boutons pour configurer chaque console, importer des fichiers locaux, préparer la commande, lancer directement, demander une fermeture normale et exporter un diagnostic. Aucun titre fictif ajouté au démarrage.
- Trois modules : Dolphin (GameCube/Wii), PCSX2 (PS2), RetroArch (systèmes indiqués disponibles, cœur adapté à choisir). Le catalogue de 36 systèmes distingue explicitement les intégrations absentes : présence au catalogue ne signifie pas compatibilité.
- Limites : installation guidée Dolphin x64 disponible sur sa branche ; autres émulateurs/cœurs/BIOS et contrôles à installer/configurer ; pas de profils automatiques par matériel/jeu ni de correctifs audio/vidéo. Atelier au clavier/souris, retour XInput en partie. Aucune garantie de compatibilité déduite de la présence des fichiers. Pas d'installateur autonome.
- Vérifications de l'atelier : compilation Release sans erreur ni avertissement, 63 contrôles locaux réussis, trois onglets WPF rendus et inspectés, dossier portable publié. Vérifications GitHub à consulter dans la PR. Installation Dolphin officielle testée et `Dolphin --version` réussi ; 86 contrôles avec cette intégration. Aucun essai avec vrai jeu/manette physique, ni benchmark.

## Essayer l'atelier

Sur `feat/functional-workbench`. Pour développer : Windows et SDK .NET 8, version 8.0.300 ou ultérieure dans la série 8.

```powershell
dotnet run --project src/Launcher.Desktop/Launcher.Desktop.csproj
dotnet build src/Launcher.Desktop/Launcher.Desktop.csproj --configuration Release
dotnet run --project tests/Launcher.Checks/Launcher.Checks.csproj --configuration Release
```

Choisir une console à gauche. Dans **Configuration de la console**, sélectionner l'exécutable et, pour RetroArch, le cœur Windows `.dll`, puis enregistrer. Dans **Bibliothèque et lancement**, ajouter ses fichiers locaux, vérifier la préparation et lancer. L'atelier se réduit pendant la partie et revient à sa fin.

Retour : fermeture normale du jeu, bouton de demande de fermeture, Ctrl + Alt + Retour arrière, ou Start + Back maintenus 1,5 seconde sur une manette XInput. Le raccourci clavier est enregistré seulement pendant la session ; si indisponible, un message le signale. Aucun arrêt forcé. Certains émulateurs peuvent demander une confirmation.

Bibliothèque : `%LOCALAPPDATA%/ConsoleLauncher/library.json`. Les jeux ne sont ni copiés ni déplacés ; retirer une entrée ne supprime aucun fichier. Les réglages et sauvegardes existants des émulateurs restent utilisés. Le dossier portable nécessite .NET Desktop Runtime 8. Les vérifications de préparation contrôlent les chemins et la commande, pas la compatibilité réelle du jeu, du BIOS ou du cœur.

Pour les essais locaux demandés par l'utilisateur, conserver un **dossier décompressé fixe `Console-Launcher` à côté du dépôt** (dans le dossier de sorties). Publier les nouvelles compilations dans ce même dossier, après fermeture du logiciel ; l'utilisateur ouvre toujours son `ConsoleLauncher.exe`. Ne pas imposer de ZIP à chaque modification. Les données restent dans `%LOCALAPPDATA%/ConsoleLauncher/`, hors des binaires de test. Le ZIP reste seulement un moyen de distribution si nécessaire.

## Installer Dolphin depuis l'atelier

Sur `feat/dolphin-installation`, choisir GameCube ou Wii → Configuration de la console → **Installer Dolphin 2609a**. Téléchargement officiel, vérification SHA-256, extraction 7z intégrée, progression/annulation et chemin enregistré automatiquement pour la console choisie. Si déjà présent, **Utiliser Dolphin installé** l'associe à la console sans nouveau téléchargement. Aucun besoin de 7-Zip installé.

Version officielle Windows x64 fixée : [2609a](https://dolphin-emu.org/download/release/2609a/). L'empreinte du paquet est fixée dans `DolphinPackage.Current` après vérification du téléchargement officiel. Une nouvelle version demande une nouvelle validation ; pas de mise à jour automatique ni de sélecteur de versions à ce stade. Les fichiers sont conservés sous `%LOCALAPPDATA%/ConsoleLauncher/emulators/dolphin/`. En cas de réparation, l'ancienne installation est conservée avant remplacement ; les dossiers temporaires de l'installation sont nettoyés.

Un dossier utilisateur Dolphin séparé est transmis avec `--user`. Un profil GameCube clavier + XInput de départ est créé s'il manque sur `feat/dolphin-keyboard`, sans écraser les réglages du joueur (profil XInput seul dans la proposition initiale). Les contrôles Wii restent à configurer ; les installations Dolphin choisies manuellement continuent à utiliser leurs réglages personnels. Aucun jeu ni BIOS fourni.

Dolphin demande le [prérequis Microsoft Visual C++ x64](https://learn.microsoft.com/cpp/windows/latest-supported-vc-redist), comme indiqué sur sa [page officielle](https://dolphin-emu.org/download/). L'atelier signale les bibliothèques manquantes et propose le lien officiel ; l'installation de ce prérequis reste manuelle. Leur présence seule ne valide pas leur version.

Vérifications : 86 contrôles automatisés locaux, dont extraction d'une archive de test, hash/source refusés, annulation, échec sans perte des fichiers précédents, détection/réutilisation et chemins d'extraction. Installation réelle de l'archive officielle dans un dossier isolé, puis `Dolphin --version` : version 2609a et sortie 0. Écrans absent/installé inspectés. Aucun vrai jeu ou essai physique de manette effectué.

## Contrôles GameCube au clavier ou à la manette

Sur `feat/dolphin-keyboard`, les nouvelles installations Dolphin activent le clavier et la manette XInput ensemble. Pour une installation gérée existante : **GameCube → Configuration → Contrôles GameCube — joueur 1 → choisir le profil → Appliquer les contrôles**. Pas de réinstallation nécessaire. Choix : clavier, manette XInput, ou les deux. Le profil est appliqué au prochain lancement ; modification bloquée pendant une session.

Touches : stick principal aux flèches, stick C avec I/K/J/L ; boutons GameCube A=X, B=Z, X=C, Y=S, Z=D ; Start=Entrée ; L=Q, R=W ; croix T/G/F/H (haut/bas/gauche/droite). Ces touches figurent dans l'atelier. Retour au lanceur : Ctrl+Alt+Retour arrière. Au clavier, les mouvements sont numériques.

Le changement sauvegarde le fichier précédent à côté de `Config/GCPadNew.ini`, remplace uniquement GCPad1, conserve les autres joueurs/sections et est sans effet si le même profil est déjà appliqué. Les installations Dolphin externes ne sont pas modifiées. Contrôles Wii inchangés, pas encore d'éditeur de touches. Vérifié : 97 contrôles locaux et rendu WPF ; aucun essai des entrées dans un vrai jeu.

## Passerelle entre fonctionnement et future interface

`ILauncherBackend` expose la bibliothèque, la configuration par console, l'import, le retrait d'entrée, la préparation, le lancement, la demande de fermeture et le diagnostic. Les états publiés sont : disponible, préparation, partie en cours, fermeture demandée, terminé et erreur. La future interface peut appeler ce contrat sans reprendre le code des boutons WPF. Le cœur est testable sans l'écran ; les processus et leurs arguments sont traités séparément.

La prochaine validation est un essai réel par module avec les jeux et le matériel des utilisateurs, puis des mesures du temps de lancement, du retour et de la consommation pendant une partie. Les profils de qualité et le design final restent à développer sur cette base.

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

### 2026-10-23 — Atelier fonctionnel publié

- Code : `3eabedb`, branche `feat/functional-workbench`, PR n°4 ouverte sur `feat/desktop-foundation` ; pas fusionné sur `main`.
- Fait : écran de travail simple, contrat de cœur indépendant, configuration par console, import réel, trois modules de lancement, diagnostic et retour normal de session.
- Vérifié : 63 contrôles locaux, compilation Release, rendu des trois onglets et publication portable Windows. Contrôles GitHub visibles dans la PR.
- Limites : pas encore de tests avec jeux/manettes physiques, de benchmark, de profils automatiques ou de design final. Autres modules encore à intégrer.

### 2026-10-23 — Installation Dolphin publiée

- Code : `10d8fed`, branche `feat/dolphin-installation`, PR n°5 ouverte sur `feat/functional-workbench` ; pas fusionné sur `main`.
- Fait : installation officielle Dolphin x64 vérifiée/annulable, détection, association automatique à la console et dossier de réglages séparé avec profil GameCube XInput de départ. Réglages existants préservés.
- Vérifié : 86 contrôles locaux, compilation/publication Windows, deux rendus WPF, téléchargement et extraction officiels réels, `Dolphin --version` réussi. Contrôles GitHub dans la PR.
- Limites : Visual C++ encore à installer manuellement s'il manque ; pas de mise à jour/retour de version dans l'interface, de contrôles Wii automatiques, de test avec vrai jeu/manette, ni de benchmark. Travail Rust/Tauri consulté jusqu'à `fe3c41b`, non modifié.

### 2026-10-23 — Contrôles clavier GameCube publiés

- Code : `354069c`, branche `feat/dolphin-keyboard`, PR n°6 ouverte sur `feat/dolphin-installation` ; pas fusionné sur `main`.
- Fait : profils clavier/manette/mixte, activation des installations existantes sans réinstallation, aide des touches, copie des anciens réglages et conservation des autres joueurs. Profil mixte pour les installations neuves.
- Vérifié : compilation Release sans avertissement, 97 contrôles locaux, rendu WPF inspecté, ZIP Windows publié. Contrôles GitHub visibles dans la PR.
- Limites : GameCube seulement, touches fixes et pas d'essai dans une partie réelle. Travail Rust/Tauri consulté jusqu'à `fe3c41b`, non modifié.
