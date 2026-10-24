# Contribuer à Console Launcher

Merci pour votre aide.
Ce dépôt est encore en **pré-version expérimentale** : les contributions les plus utiles sont les corrections ciblées, la fiabilisation et les retours de test réels.

## Prérequis de développement

- Rust stable (workspace : `rust-version = 1.80`)
- CMake 3.21+
- Compilateur C++17
- Qt 6.8+ (Core, Gui, Qml, Quick, Quick Dialogs)
- SDL2

Détails par système : [docs/prototype/DEPENDENCIES.fr.md](docs/prototype/DEPENDENCIES.fr.md).

## Compiler localement

```bash
cmake -S qt-prototype -B build-qt -DCMAKE_BUILD_TYPE=Release
cmake --build build-qt
./build-qt/console-launcher-qt
```

## Formatage, lint et tests

Commandes utilisées dans l'intégration continue :

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p cl-cli -- --help
```

## Branches et pull requests

- Créez une branche dédiée par sujet (`feat/...`, `fix/...`, `docs/...`).
- Évitez les changements hors périmètre de votre PR.
- Ouvrez une PR vers `main` avec un titre clair en français ou en anglais.
- Utilisez le modèle de PR (`.github/pull_request_template.md`) et renseignez :
  - le problème traité,
  - les vérifications réellement effectuées,
  - les limites restantes.
- Si votre changement touche uniquement la documentation, indiquez-le explicitement.

## Signaler un bug

Ouvrez une issue avec :

1. système (Windows/Linux),
2. version utilisée (release/tag/commit),
3. étapes exactes pour reproduire,
4. résultat attendu et résultat observé,
5. extraits de logs si disponibles (`console-launcher.log`, `console-launcher-engine.log`).

Pour les problèmes d'émulateur ou de mesure, précisez aussi l'émulateur concerné et la méthode de lancement (paquet Windows, AppImage Linux ou build local).
