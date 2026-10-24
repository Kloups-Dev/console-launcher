# Console Launcher

Console Launcher est un lanceur de jeux orienté manette pour Windows et Linux.
Le projet vise un parcours simple : **menu → console → jeu → partie → retour au menu**, en s'appuyant sur des émulateurs existants.

> ⚠️ **Statut actuel : pré-version expérimentale.**
> Le projet est utilisable en prototype, mais il n'est pas encore validé sur toutes les configurations réelles (Windows physique, vrais émulateurs, vraies manettes, vraies parties).

## Fonctionnalités actuelles

- Interface Qt Quick (QML) pilotable à la manette, au clavier et à la souris.
- Bibliothèque de jeux par console avec lancement direct.
- Détection et configuration d'émulateurs reconnus, avec support d'émulateurs personnalisés.
- Réglages accessibles depuis l'interface (consoles, émulateurs, dossiers, mesure de fluidité).
- Outils de mesure intégrés dans les paquets de test : PresentMon (Windows) et MangoHud (Linux).
- Moteur Rust séparé de l'interface, avec commandes CLI (`clctl`) pour diagnostiquer et tester.

## Plateformes

- **Windows** : paquet de test x64.
- **Linux** : paquet AppImage x64.

## Prérequis

### Utilisation des paquets de test

- Vos propres jeux, BIOS/firmwares et installations d'émulateurs (non fournis par ce dépôt).
- Sous Windows, une autorisation peut être nécessaire pour la mesure de fluidité (PresentMon).

### Compilation depuis les sources

- Rust stable (workspace : `rust-version = 1.80`).
- CMake 3.21+.
- Compilateur C++17.
- Qt 6.8+ (Core, Gui, Qml, Quick, Quick Dialogs).
- SDL2.

Détails par système : [docs/prototype/DEPENDENCIES.fr.md](docs/prototype/DEPENDENCIES.fr.md).

## Télécharger

- Pré-versions publiées : [Releases GitHub](https://github.com/Kloups-Dev/console-launcher/releases)
- Artefacts de build (tests de branche) : onglet **Actions** du dépôt, exécution **CI**.

Guides d'installation : [docs/installation.md](docs/installation.md).

## Compiler et tester depuis les sources

### Build Qt Quick

```bash
cmake -S qt-prototype -B build-qt -DCMAKE_BUILD_TYPE=Release
cmake --build build-qt
./build-qt/console-launcher-qt
```

### Vérifications Rust (comme dans la CI)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p cl-cli -- --help
```

## Limites connues

- Validation incomplète sur des machines Windows physiques.
- Validation incomplète avec de vrais jeux/manettes/émulateurs sur tout le périmètre.
- Seuils de mesure/ajustement de fluidité encore à calibrer en conditions réelles.
- Certaines parties des réglages d'émulateurs restent à confirmer sur installations réelles.

Dépannage : [docs/troubleshooting.md](docs/troubleshooting.md).

## Documentation

- Installation : [docs/installation.md](docs/installation.md)
- Contribution : [CONTRIBUTING.md](CONTRIBUTING.md)
- Dépannage : [docs/troubleshooting.md](docs/troubleshooting.md)
- Roadmap : [docs/roadmap.md](docs/roadmap.md)
- Journal technique et historique : [docs/development-log.md](docs/development-log.md)
- Documentation prototype détaillée : [docs/prototype/README.fr.md](docs/prototype/README.fr.md)

## Licence

La licence du projet **n'est pas encore choisie**.
Aucun fichier `LICENSE` n'est publié pour le moment.
