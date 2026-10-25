# Console Launcher

Ce document décrit **l'application telle qu'elle existe réellement**. Les
objectifs futurs, les priorités et les critères de réussite sont séparés dans
[ROADMAP.md](ROADMAP.md). L'historique détaillé des versions reste dans
[CHANGELOG.md](CHANGELOG.md).

## Ce qu'est Console Launcher

Console Launcher est un lanceur d'émulation pour Windows et Linux. Il utilise
des émulateurs existants derrière une interface commune afin qu'une personne
qui ne les connaît pas puisse ajouter ses jeux, choisir une console et jouer.
L'objectif visuel est une interface originale, fluide et utilisable à la
manette. La PS3 est une référence de simplicité de navigation, pas une
interface à reproduire.

Parcours : **menu → console → jeu → partie → retour au menu**. Les menus et bibliothèques des émulateurs ne sont pas une étape visible. Les réglages restent accessibles et compréhensibles : il ne s'agit pas de supprimer tous les paramètres.

Les émulateurs sont réutilisés, pas réécrits. Aucun jeu, BIOS, firmware ou clé
n'est fourni. Il n'existe pas d'objectif de réseau social. Le nom définitif et
la licence restent à choisir. Aucun gain de performance ni aucune émulation
parfaite ne sont encore démontrés.

## Lecture rapide de l'état actuel

### Fonctionnel dans la version 0.3.2

- application Qt Quick/QML avec passerelle C++ et moteur Rust ;
- interface de bureau classique : menus, barre d'outils, catalogue des
  consoles, bibliothèque, détails et barre d'état ;
- catalogue permanent de 15 consoles ;
- ajout de dossiers ou de fichiers de jeu, recherche, favoris et lancement ;
- détection de RetroArch, Dolphin, PCSX2, mGBA, DuckStation, PPSSPP, melonDS et
  Cemu, plus les émulateurs personnalisés ;
- installation directe et vérifiée de Dolphin 2609a sous Windows, et gestion
  de PCSX2 ;
- fiches et jaquettes mises en cache, avec prise en charge régionale GameTDB
  pour GameCube/Wii, Nintendo DS et Wii U ;
- détection du matériel, choix d'un profil prudent, intégration de PresentMon
  sous Windows et de MangoHud sous Linux ;
- respect des réglages modifiés à la main et sauvegarde avant modification ;
- paquets Windows et Linux testés au démarrage par la CI ;
- journaux locaux de l'interface et du moteur, sans envoi automatique.

### Partiel ou non validé

- les profils automatiques et seuils de fluidité existent, mais leur calibration
  sur plusieurs vrais PC et plusieurs vrais jeux reste à faire ;
- la mesure ne permet pas encore de garantir l'absence de bug audio ;
- certains adaptateurs ont surtout été validés par des tests automatisés et
  doivent être confirmés avec les versions réelles des émulateurs ;
- l'identification et les jaquettes ne couvrent pas encore toutes les consoles
  et tous les formats compressés ;
- RetroArch et ses cœurs restent à installer manuellement ;
- il n'existe pas encore d'installateur Windows classique ni de thème final.

### Travail local en cours, non publié

La branche `feature/controller-visualizer` ajoute un premier testeur visuel :
source clavier ou manette, console cible, représentation des deux contrôleurs et
réaction en direct. Ce travail compile et le clavier a été observé dans
l'interface, mais les mappings ne sont pas encore enregistrés ni appliqués aux
émulateurs, et les manettes physiques restent à tester.

## Architecture actuelle

- `qt-prototype/src` : fenêtre native, SDL2 et passerelle vers le moteur ;
- `qt-prototype/ui` : interface et composants QML compilés dans l'application ;
- `crates/cl-library` : scan, identité, bibliothèque et métadonnées ;
- `crates/cl-adapters` : profils et adaptateurs d'émulateurs ;
- `crates/cl-hwprobe` et `crates/cl-perf` : matériel et fluidité ;
- `crates/cl-session` : lancement, mesure et suivi des processus ;
- `crates/cl-updater` : installations et mises à jour gérées ;
- `crates/cl-engine`, `cl-api` et `cl-ffi` : moteur et contrat avec Qt ;
- `crates/cl-cli` : outil de diagnostic `clctl`.

L'interface ne doit pas contenir la logique propre à Dolphin, PCSX2 ou
RetroArch. Elle présente les consoles, jeux et actions ; les adaptateurs
traduisent ensuite ces actions pour chaque émulateur.

## Sources de vérité et fonctionnement partagé

- `README.md` décrit ce qui existe ; `ROADMAP.md` décrit ce que l'on veut
  construire ; `CHANGELOG.md` décrit ce qui a été publié. Une ambition ne doit
  jamais être présentée comme une fonction disponible.
- Au début de chaque nouvelle phase de code, synchroniser le dépôt, lire le
  README et la roadmap, puis consulter les branches utiles. Signaler si l'accès
  réseau empêche d'actualiser le contexte.
- Chaque IA est indépendante dans le périmètre fixé par les humains. Aucune IA ne dirige l'autre, ne lui attribue de tâche ou de fichiers, ni ne lui impose une technologie ou une voie de développement.
- Les choix existants décrivent le travail réalisé ; ils peuvent être améliorés ou remis en question. Préserver les changements d'autrui et signaler les conflits.
- Après chaque push de code, mettre à jour l'état réel si nécessaire. Le détail
  historique appartient au changelog, à Git et aux pull requests.
- Indiquer date, branche, commit de code, changements, vérifications réellement faites et limites. Distinguer travail envoyé, fusionné et encore local. Ne pas écrire de secrets ici.
- Une compilation réussie ne prouve pas qu'une manette physique, un émulateur
  réel, le son ou une partie complète fonctionne.

## État actuel

- Dépôt : https://github.com/Kloups-Dev/console-launcher. La branche principale est `main`.
- Version actuelle : **0.3.2**, interface Qt Quick intégrée, passerelle C++/Qt et moteur Rust. Les anciennes voies WPF et Tauri restent consultables dans l'historique et leurs branches, mais ne constituent plus l'application principale.
- Le lanceur affiche les 15 consoles prises en charge, organise les réglages généraux puis les réglages propres à chaque console, détecte ou installe les émulateurs pris en charge et lance directement les jeux.
- La bibliothèque possède un cache local de fiches et de jaquettes. GameCube/Wii, Nintendo DS et Wii U utilisent GameTDB avec sélection de l'édition régionale et validation des images téléchargées.
- L'optimisation privilégie la stabilité, respecte les réglages modifiés à la main et peut mesurer la fluidité avec PresentMon sous Windows ou MangoHud sous Linux.
- La CI vérifie le formatage, le lint et les tests du moteur sous Windows et Linux, construit les deux applications, les ouvre avec `--smoke-test`, puis fournit les paquets téléchargeables.
- Limites : aucun jeu ni BIOS fourni ; calibration sur davantage de machines
  réelles encore nécessaire ; RetroArch reste à installer manuellement ; pas
  encore d'installateur Windows classique. Les attentes futures sont dans
  [ROADMAP.md](ROADMAP.md), pas dans cet état actuel.

## Essayer le prototype

Télécharger la pré-version la plus récente dans [Releases](https://github.com/Kloups-Dev/console-launcher/releases) :

- Windows : décompresser le ZIP, puis lancer `ConsoleLauncher\console-launcher-qt.exe`.
- Linux : rendre `ConsoleLauncher-x86_64.AppImage` exécutable, puis le lancer.

Les jeux ne sont ni copiés ni déplacés. Les réglages du lanceur sont enregistrés dans `%APPDATA%\ConsoleLauncher\console-launcher\data` sous Windows et `~/.local/share/console-launcher` sous Linux.

## Historique de construction

Cette section conserve les principaux jalons de développement. Pour les
changements par version, [CHANGELOG.md](CHANGELOG.md) reste la référence.

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

### 2026-10-23 — Interface Qt Quick avec le moteur intégré (v0.3.0)

- Code : `f784eae` à `7a497d9`, branche `feat/qt-quick-comparison`, envoyé. Branche reçue en bundle (`896df46`), fusionnée avec `feat/rust-tauri-prototype` (`745a9eb`).
- Fait : moteur intégré à l'application Qt (crates `cl-api` et `cl-ffi`, bibliothèque `console_launcher_engine`, appels sur des fils de travail) ; Qt 6.8 LTS minimum ; panneau de réglages QML partagé par les thèmes (`import ConsoleLauncher`) ; PresentMon 2.6.0 et MangoHud 0.8.4 fournis, bouton d'autorisation de la mesure sous Windows ; CI : paquet Windows (windeployqt) et AppImage Linux ; interface Tauri retirée de cette branche ; doc FR/EN (`ENGINE_API`, `THEMES` réécrit pour QML, `DEPENDENCIES`).
- Vérifié : tests Rust, `cargo clippy -D warnings` et `cargo fmt` sous Linux et Windows (CI) ; application compilée avec Qt 6.8.4 et essayée sur écran virtuel Linux avec une fausse installation RetroBat (bibliothèque, détail des jeux, réglages, choix d'émulateur, page d'un émulateur manquant, lancement avec un faux PPSSPP) ; paquets Windows et Linux produits par la CI.
- Limites : paquets non ouverts sur une vraie machine ; MangoHud fourni et autorisation PresentMon jamais essayés avec une vraie partie ; sélecteurs de fichiers non essayés à la manette ; PR n°1 (WPF) non modifiée.

### 2026-10-23 — Pré-version de test Qt Quick (v0.3.0-qt)

- Code : `88cdcea` puis suivi CI, branche `feat/qt-quick-comparison`, envoyé.
- Fait : un tag `v…` publie les paquets Windows et Linux en pré-version GitHub (jamais « dernière version ») ; une version créée depuis la page GitHub (qui crée le tag) est complétée par la CI avec les paquets.
- Limites : l'environnement de l'IA ne peut pas envoyer de tag ; le tag `v0.3.0-qt` est à créer par un humain. La page des versions est commune au dépôt ; la pré-version reste rattachée à cette branche par son tag.

### 2026-10-23 — Correction du démarrage sous Windows (Qt Quick, v0.3.1)

- Code : `8cf79d5` et `03cf2a3`, branche `feat/qt-quick-comparison`, envoyé.
- Fait : la v0.3.0 tournait sans fenêtre sous Windows (« SettingsPanel is not a type ») car le module QML `ConsoleLauncher` n'était pas dans les paquets ; il est maintenant compilé dans le programme. Ajouts : journaux `console-launcher.log` et `console-launcher-engine.log`, fenêtre d'erreur si un thème ne charge pas, manettes SDL sur leur propre fil après l'affichage, test `--smoke-test` lancé par la CI sur les deux paquets.
- Vérifié : panne reproduite avec le paquet v0.3.0 sous Wine et sur la machine Windows de la CI ; après correction, test de démarrage réussi sous Windows Server 2025 et dans l'AppImage Linux (CI), et depuis un dossier propre sous Linux.
- Limites : pas encore relancé sur la machine Windows d'un humain.

### 2026-10-23 — Pré-versions automatiques (Qt Quick)

- Code : `78be4bc`, branche `feat/qt-quick-comparison`, envoyé.
- Fait : la CI publie seule la pré-version `v<version>-qt` (paquets + section du changelog) quand la version de `Cargo.toml` n'a pas encore de release ; plus de tag à créer à la main. Un tag `v…` manuel reste possible.
- Limites : la publication suit la réussite des tests de démarrage ; une version déjà publiée n'est pas remplacée.

### 2026-10-24 — Base principale Qt Quick et version 0.3.2

- Code : `0071a91` pour le remplacement de la V1 sur `main`, puis `f1c134d` pour la version 0.3.2, envoyés.
- Fait : interface de bureau réorganisée, hiérarchie claire des réglages et consoles, installation directe de Dolphin, diagnostic local, optimisation stabilité-d'abord, fiches de jeux et jaquettes régionales GameTDB mises en cache.
- Vérifié : 79 tests locaux, formatage et lint strict réussis ; la CI du tag `v0.3.2-qt` a compilé, testé et ouvert les paquets Windows et Linux, puis publié la pré-version avec ses deux téléchargements.
- Limites : mesure de fluidité et réglages automatiques encore à calibrer sur plusieurs vraies machines et plusieurs émulateurs ; pas de jeu ni BIOS distribué.

### 2026-10-25 — Visualisation Xbox locale

- Face avant Xbox retracée en QML, affichée pour les noms de périphérique Xbox/XInput reconnus ; autres manettes inchangées.
- Boutons réactifs, gâchettes LT/RT en ronds au-dessus de LB/RB et points de direction des sticks.
- Détection visuelle fondée sur le nom SDL ; les modèles tiers sans nom Xbox/XInput conservent le dessin générique.
- Travail local : application des mappings aux émulateurs et validation avec une manette physique toujours à faire.

- Ajustement demandé : dessin Xbox par défaut même sans périphérique, clavier toujours sélectionnable ; panneau cible et sélection GameCube/PS2/Switch masqués en attendant leurs vrais dessins. Compilation Qt complète réussie.

- Testeur Xbox : transitions de couleur et pression visuelle des boutons, anneaux analogiques LT/RT avec pourcentages, mouvement des points de stick sur 20 ms, valeurs X/Y brutes affichées et animations désactivables. Compilation Qt réussie ; validation physique toujours à faire.

- GameCube : vue avant vectorielle d'après l'image fournie, affichée comme cible à droite de la source Xbox/clavier. A/B/X/Y aux formes distinctes, sticks principal et C, croix, Start, L/R analogiques et Z réactifs. Correspondance uniquement visuelle ; aucune configuration Dolphin modifiée. Vérification QML, rendu du composant et compilation complète réussis ; essai physique à faire.

- Vue GameCube remplacée par la géométrie exacte du SVG nettoyé validé : arcs supérieurs réguliers, croix et stick simplifiés, sans câble, logo ni texte Start/Pause. Animations et correspondances visuelles conservées, sans application aux émulateurs. Vérification QML et compilation complète réussies.

- Vue PS2 intégrée à partir du SVG approuvé : sans logos, bouton Analog ni voyant, indicateurs L1/L2/R1/R2 séparés au-dessus, boutons géométriques et deux joysticks réactifs (clics L3/R3 compris). Sélecteur GameCube/PS2 dans le testeur, PS2 affichée initialement. Correspondance visuelle uniquement, sans modification PCSX2. Vérification QML, rendu du composant et compilation réussis ; essai physique à faire.

- DualSense PS5 : dessin approuvé intégré comme périphérique source sélectionnable (Xbox/PS5), sans micro, haut-parleur ni voyants. Create, Options, accueil et clic du pavé tactile conservés ; lecture SDL du bouton Touchpad ajoutée. Sticks, clics L3/R3, boutons et gâchettes réactifs ; gâchettes adaptatives et suivi tactile non implémentés. Détection du dessin par nom explicite DualSense/PS5, sélection manuelle disponible. Cibles GameCube/PS2 inchangées ; aucun émulateur PS5 ajouté. Affectation personnalisée toujours à développer. Compilation complète réussie, essais physiques à faire.

- Sélecteurs du testeur simplifiés : source Clavier/Xbox/PS5 à gauche et cible GameCube/PlayStation 2/Xbox/PS5 à droite. Xbox cible reconvertit les codes de boutons normalisés pour afficher correctement ABXY. Les quatre dessins sont des correspondances visuelles, sans application aux émulateurs. Compilation réussie.

### 2026-10-25 — Première version locale du diagnostic en jeu (Windows)

- Affichage/masquage avec **F8** pendant une partie lancée depuis le logiciel ; touche F1 à F11 et activation configurables dans « Performance et machine ». F12 exclue.
- Panneau sans capture de souris/clavier : il ne prend pas le focus et se masque quand une autre application passe devant. Vise les jeux en mode fenêtré/sans bordure ; plein écran exclusif non validé.
- Collecteur PresentMon continu séparé de la capture d'optimisation : FPS présentés, temps d'image moyen, p99, 1 % bas dérivé du p99 et nombre de pics > 1,5 fois la médiane sur une fenêtre maximale de 2 secondes. Choix de la surface de rendu dominante, filtrage du PID exact et mémoire bornée. Aucun journal CSV supplémentaire continu.
- CPU du processus rapporté à la capacité CPU totale du PC et mémoire résidente du processus. Mesure soumise à l'autorisation Windows de PresentMon, disponible dans les réglages de fluidité.
- **Vitesse d'émulation, audio, charge GPU et températures non mesurés** : affichés comme tels, jamais déduits des FPS. Aucun verdict « sans lag » ou « son correct » produit.
- Liaison de session temporaire (PID et exécutable uniquement), supprimée en fin de partie ; collecteur arrêté et raccourci libéré. Aucun envoi réseau ni enregistrement des entrées.
- Vérifié : compilation Qt et Rust release, 3 tests Rust de session, tests CSV/multi-surfaces/expiration/bornes mémoire, test natif d'affichage/masquage et nettoyage, chargement complet avec `--smoke-test`.
- Limites : pas de validation avec une vraie partie/chaîne audio, pas de garantie d'absence de conflit avec les raccourcis internes d'un émulateur ; panneau Windows seulement, voie Linux à développer. Travail local, non envoyé sur GitHub.