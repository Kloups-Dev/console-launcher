# Installation (expérimentale)

Console Launcher est actuellement diffusé en **pré-versions de test**.
Les procédures ci-dessous reprennent uniquement ce qui est déjà documenté dans le dépôt.

## 1) Télécharger une build

Deux sources existent :

- **Releases** : [https://github.com/Kloups-Dev/console-launcher/releases](https://github.com/Kloups-Dev/console-launcher/releases)
- **Artefacts CI** : onglet **Actions** du dépôt, workflow **CI**, artefacts de la dernière exécution

Paquets attendus :

- `console-launcher-qt-windows-x64`
- `console-launcher-qt-linux-x64`

## 2) Windows (paquet x64)

1. Télécharger l'archive Windows.
2. Décompresser le dossier.
3. Lancer `ConsoleLauncher\console-launcher-qt.exe`.

Notes :

- Les DLL Qt/SDL2 sont incluses dans le paquet généré par la CI.
- Les outils de mesure sont fournis dans `tools/presentmon/`.
- Pour la première mesure de fluidité : dans **Réglages > Mesure de fluidité**, choisir **Autoriser la mesure**, puis fermer et rouvrir la session Windows.

## 3) Linux (AppImage x64)

1. Télécharger `ConsoleLauncher-x86_64.AppImage`.
2. Le rendre exécutable.
3. Le lancer.

Exemple :

```bash
chmod +x ConsoleLauncher-x86_64.AppImage
./ConsoleLauncher-x86_64.AppImage
```

Notes :

- Le paquet inclut les dépendances Qt nécessaires via AppImage.
- Les outils de mesure sont fournis dans `tools/mangohud/` dans l'AppImage.

## 4) Compiler depuis les sources

Prérequis détaillés : [docs/prototype/DEPENDENCIES.fr.md](prototype/DEPENDENCIES.fr.md).

```bash
cmake -S qt-prototype -B build-qt -DCMAKE_BUILD_TYPE=Release
cmake --build build-qt
./build-qt/console-launcher-qt
```

Vérifications recommandées :

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p cl-cli -- --help
```

## 5) Limites actuelles

- Pré-version non validée sur toutes les machines Windows physiques.
- Validation réelle encore incomplète avec tous les émulateurs/jeux/manettes.
- Mesure de fluidité intégrée mais encore en phase de validation terrain.

Voir aussi : [docs/troubleshooting.md](troubleshooting.md).
