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

- Dépôt : https://github.com/Kloups-Dev/console-launcher. La branche principale est `main`.
- Version actuelle : **0.3.2**, interface Qt Quick intégrée, passerelle C++/Qt et moteur Rust. Les anciennes voies WPF et Tauri restent consultables dans l'historique et leurs branches, mais ne constituent plus l'application principale.
- Le lanceur affiche les 15 consoles prises en charge, organise les réglages généraux puis les réglages propres à chaque console, détecte ou installe les émulateurs pris en charge et lance directement les jeux.
- La bibliothèque possède un cache local de fiches et de jaquettes. GameCube/Wii, Nintendo DS et Wii U utilisent GameTDB avec sélection de l'édition régionale et validation des images téléchargées.
- L'optimisation privilégie la stabilité et respecte les réglages modifiés à la main. Dolphin démarre au rendu natif et ne monte plus automatiquement en qualité. PresentMon ou MangoHud mesure déjà la régularité vidéo ; la certification refuse de conclure « stable » tant que les preuves de vitesse interne et de santé audio ne sont pas encore disponibles.
- La CI vérifie le formatage, le lint et les tests du moteur sous Windows et Linux, construit les deux applications, les ouvre avec `--smoke-test`, puis fournit les paquets téléchargeables.
- Limites : aucun jeu ni BIOS fourni ; calibration sur davantage de machines réelles encore nécessaire ; RetroArch reste à installer manuellement ; pas encore d'installateur Windows classique. Détails : `docs/prototype/README.fr.md`.

## Essayer le prototype

Télécharger la pré-version la plus récente dans [Releases](https://github.com/Kloups-Dev/console-launcher/releases) :

- Windows : décompresser le ZIP, puis lancer `ConsoleLauncher\console-launcher-qt.exe`.
- Linux : rendre `ConsoleLauncher-x86_64.AppImage` exécutable, puis le lancer.

Les jeux ne sont ni copiés ni déplacés. Les réglages du lanceur sont enregistrés dans `%APPDATA%\ConsoleLauncher\console-launcher\data` sous Windows et `~/.local/share/console-launcher` sous Linux.

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

### 2026-10-24 — Certification stabilité-d'abord

- Code : `0359ef1`, branche `feature/stability-certification`, envoyé, pas encore fusionné.
- Fait : Dolphin repart du rendu natif, la qualité ne monte plus automatiquement et le moteur sépare maintenant les verdicts stable, instable et mesures insuffisantes à partir de trois preuves prévues (vidéo, vitesse interne et audio). L'état est visible dans la fiche du jeu et au retour d'une partie ; les anciens profils restent lisibles.
- Vérifié : 83 tests de l'espace Rust, formatage et lint strict réussis ; moteur et application Qt compilés en Release sous Windows ; test de démarrage Qt réussi.
- Limites : PresentMon/MangoHud fournit actuellement la preuve vidéo uniquement. La collecte de la vitesse interne et des sous-alimentations audio de Dolphin reste à connecter avant qu'une partie réelle puisse recevoir le verdict « stable ».
