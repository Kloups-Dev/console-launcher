# API du moteur

*[English version](ENGINE_API.en.md)*

Le moteur (Rust) est une bibliothèque partagée, `console_launcher_engine.dll` sous Windows et `libconsole_launcher_engine.so` sous Linux, compilée depuis la crate `cl-ffi`. L'interface Qt la charge directement ; n'importe quel autre programme peut l'utiliser grâce à l'en-tête C [`crates/cl-ffi/include/console_launcher_engine.h`](../../crates/cl-ffi/include/console_launcher_engine.h).

```c
ClEngine *e = cl_engine_open(NULL);                  /* NULL : dossier de données standard */
char *reponse = cl_engine_call(e, "library", NULL); /* réponse JSON */
cl_string_free(reponse);
cl_engine_close(e);
```

- Les demandes et les réponses sont en JSON UTF-8. Chaque réponse vaut `{"ok": true, "result": …}` ou `{"ok": false, "error": "…"}`.
- `cl_engine_call` peut être appelée depuis plusieurs fils. Les appels sont traités un par un, sauf `play`, qui attend la fermeture de l'émulateur et tourne en parallèle des autres.
- Les messages d'erreur suivent la langue choisie dans les réglages (`fr` ou `en`).
- Une erreur interne ne fait jamais planter l'appelant : elle revient sous la forme `{"ok": false}`.

L'interface QML intégrée utilise une passerelle privée `Launcher.internalRequest(...)`. Elle n'appartient pas au contrat des futurs thèmes et aucun QML externe n'est exécuté.

## Méthodes

| Méthode | Paramètres | Résultat |
|---|---|---|
| `version` | — | Version du moteur |
| `config` | — | Contenu de `config.toml` (rapide : aucun scan du disque) |
| `library` | — | `{systems, games}` |
| `add_game` | `system`, `path` | Ajoute directement un fichier de jeu après vérification de son extension |
| `remove_game` | `path` | Retire un jeu ajouté directement, sans supprimer le fichier |
| `game_info` | `id` | Ce que ferait un lancement : émulateur, palier de qualité, remarques. N'écrit rien. Quand le jeu ne peut pas démarrer : `{ready: false, code, emulatorId, error}` |
| `refresh_game_metadata` | `id` | Recherche explicitement la fiche dans Title Screen et la jaquette GameTDB pour GameCube/Wii, Nintendo DS et Wii U. La fiche et l'image PNG/JPEG validée sont mises en cache puis les métadonnées sont renvoyées, ou `null`. Le scan normal ne fait jamais de réseau. |
| `play` | `id` | Lance le jeu, attend la fermeture de l'émulateur, mesure et ajuste. Rapport : fluidité, palier suivant, remarques |
| `toggle_favorite` | `id` | Nouvel état (`true` / `false`) |
| `set_quality` | `id`, `step` | Fixe un palier à la main ; la recherche automatique s'arrête pour ce jeu |
| `settings_overview` | — | Tout ce qu'affiche l'écran de réglages : `config`, `emulators` (état, chemin, réglages gardés par l'utilisateur), `systems` (choix de l'émulateur par console), `capture` (outil de mesure, autorisation), `profileErrors`, `dataDir` |
| `set_system_emulator` | `system`, `id` (null : automatique) | — |
| `set_emulator_path` | `id`, `path` (null : détection automatique) | — |
| `add_custom_emulator` | `path`, `name`, `systems`, `args` | Identifiant de l'émulateur. Un exécutable connu donne l'émulateur connu |
| `update_custom_emulator` | `id`, `emulator` (`{name, path, args, systems}`) | — |
| `remove_custom_emulator` | `id` | — |
| `reset_emulator_tuning` | `id` | Rend à l'optimisation automatique les réglages modifiés à la main |
| `open_emulator` | `id` | Ouvre l'émulateur sans jeu |
| `install_emulator` | `id` | Télécharge, vérifie et active une version gérée disponible (Dolphin 2609a sous Windows) |
| `add_path` | `path` | `{kind: "games" \| "emulators" \| "emulator" \| "already_added"}` |
| `remove_path` | `path` | — |
| `update_preferences` | au choix `autoTune`, `systemTweaks`, `presentmonPath` (`""` l'efface), `language`, `theme` | — |
| `allow_capture` | — | Windows : ajoute l'utilisateur au groupe « Utilisateurs du journal de performances » après une fenêtre administrateur (actif à la prochaine ouverture de session) |
| `hardware` | `refresh` | Machine détectée ; `true` refait le test |
| `reload` | — | Relit `config.toml` et les fichiers de profil |
