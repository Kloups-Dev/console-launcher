# Créer un thème

*[English version](THEMES.en.md)*

Un thème est un dossier contenant 4 fichiers : HTML, CSS, JavaScript et un manifeste. Il décide de **tout** l'affichage : disposition, couleurs, animations, sons, ordre des menus. Le lanceur lui fournit les données (consoles, jeux, jaquettes, réglages) et les actions de la manette. Le thème n'a jamais besoin de savoir quel émulateur est utilisé.

Pour jouer, aucun thème n'est à créer : « Horizon » est actif par défaut.

## Où se trouvent les thèmes

`<dossier de données>/themes/<nom>/`. Les deux modèles y sont copiés au premier lancement et ne sont plus jamais écrasés ensuite : on peut les modifier directement.

```
themes/mon-theme/
  theme.json    manifeste (nom, options réglables)
  index.html    structure de la page
  style.css     apparence
  theme.js      comportement (reçoit l'objet Launcher)
  ...           images, polices, sons : libre
```

Le nom du dossier ne peut contenir que des lettres, des chiffres, `-` et `_`.

## Partir d'un modèle

1. Copier `themes/grid` (le plus court, environ 80 lignes de JS) ou `themes/horizon` (plus complet).
2. Renommer le dossier et le champ `name` de `theme.json`.
3. Ouvrir le menu (Tab / Start) → Thème, jusqu'à afficher le nouveau.
4. Modifier, puis appuyer sur **F5** pour recharger. **F11** passe en plein écran.

Pour travailler directement dans un dossier de développement :

```bash
CONSOLE_LAUNCHER_THEMES=/chemin/vers/mes-themes cargo run -p console-launcher
```

Les erreurs JavaScript s'affichent en bas de l'écran, et le menu reste accessible avec Tab pour changer de thème.

## Le manifeste `theme.json`

```json
{
  "name": "Mon thème",
  "author": "Pseudo",
  "version": "1.0.0",
  "description": "Une ligne",
  "options": [
    { "id": "accent", "type": "color", "label": "Accent", "default": "#ffc440" },
    { "id": "background", "type": "select", "label": "Fond", "choices": ["aurora", "plain"], "default": "aurora" },
    { "id": "clock", "type": "bool", "label": "Horloge", "default": true }
  ]
}
```

Chaque option devient :
- une variable CSS `--opt-<id>` (couleurs, nombres, textes) ;
- un attribut `data-opt-<id>` sur `<html>`, pratique pour les choix : `html[data-opt-clock="false"] .horloge { display: none }` ;
- une valeur lisible en JS avec `Launcher.option("accent")`.

Les valeurs choisies par l'utilisateur sont enregistrées dans `user-options.json`, à côté du thème.

## API `Launcher`

Toutes les fonctions asynchrones renvoient une promesse.

| Fonction | Rôle |
|---|---|
| `onNav(cb)` | Reçoit les actions : `up`, `down`, `left`, `right`, `confirm`, `back`, `menu`, `favorite`, `pageLeft`, `pageRight`. Elles viennent de la manette, du clavier ou de la répétition automatique quand une direction est maintenue |
| `onSession(cb)` | `{state: "started"}` au lancement d'un jeu, puis `{state: "ended", report}` ou `{state: "failed", error}` |
| `onPad(cb)` | Manette branchée ou débranchée : `{name, connected}` |
| `getLibrary()` | `{systems, games}` (détail ci-dessous) |
| `gameInfo(id)` | `{ready, emulator, emulatorId, tunesSettings, stepLabel, ladder, settled, measure, belowRequirements, notes}` ou `{ready: false, code, emulatorId, error}` (`code` : `emulator_missing`, `no_emulator`…) |
| `play(id)` | Lance le jeu. La promesse se termine quand l'émulateur se ferme, avec le rapport (fluidité mesurée, réglage suivant) |
| `toggleFavorite(id)` | Renvoie le nouvel état |
| `setQuality(id, palier)` | Fixe un palier à la main. L'optimisation automatique s'arrête pour ce jeu |
| `getSettings()` / `saveSettings(s)` | Lit ou écrit `config.toml` |
| `hardware(refresh)` | Machine détectée. `true` relance le test |
| `listThemes()` / `setTheme(id)` | Changer de thème (la page se recharge) |
| `option(id)` / `setOption(id, valeur)` | Options du manifeste |
| `asset("img/fond.png")` | URL d'un fichier du dossier du thème |
| `openSettings(cible?)` | Ouvre le panneau de réglages intégré, ce qui évite de devoir en coder un. `cible` : `{emulator: id}` ouvre la page d'un émulateur, `{page: "systems" \| "emulators" \| "folders" \| "capture"}` une page |
| `toggleFullscreen()`, `quit()` | — |
| `escapeHtml(texte)` | À utiliser pour insérer des titres de jeux dans du HTML |

Événement DOM : `launcher:library-changed`, émis quand l'utilisateur ajoute un dossier de jeux depuis le panneau intégré. Il faut alors recharger la bibliothèque.

### Données de `getLibrary()`

```js
systems: [{
  info: { id: "snes", name: "Super Nintendo", manufacturer: "Nintendo", year: 1990, cost: 1, ... },
  games: 42,              // nombre de jeux
  emulator: "retroarch",
  emulator_found: true
}]
games: [{
  id: "6edaa4f992fd1b98", // stable : favoris et profils y sont rattachés
  system: "snes",
  title: "Mon jeu",       // nettoyé : "(Europe)", "[!]"… retirés
  path: "...", cover: "...", coverUrl: "...", favorite: false
}]
```

## Bonnes pratiques

- Ne réagir qu'aux actions de `onNav`, sans lire la manette soi-même : le thème fonctionne ainsi avec toutes les manettes et au clavier.
- Afficher les longues listes par fenêtre (une dizaine d'éléments autour de la sélection), comme les modèles, pour rester fluide avec des milliers de jeux.
- Charger `gameInfo` avec un léger délai pendant le défilement (voir Horizon).
- Ne rien charger depuis Internet : le lanceur doit fonctionner hors ligne.
- Le JavaScript d'un thème a les mêmes droits que l'interface. N'installez que des thèmes de personnes de confiance.
