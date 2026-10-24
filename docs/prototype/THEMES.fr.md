# Créer un thème

> **État actuel : thèmes externes désactivés.** L'application charge uniquement
> l'interface QML compilée dans l'exécutable. Les anciens points d'entrée
> `Main.qml` ont été retirés et ne sont plus exécutés. Le
> futur format Theme v1 sera un paquet de données validé (manifeste, couleurs,
> polices, images et sons), avec l'interface fonctionnelle intégrée comme repli.
> Cette décision évite qu'un thème puisse appeler le moteur ou casser le
> démarrage. La suite de ce document décrit l'ancien prototype et ne constitue
> plus une API prise en charge.

*[English version](THEMES.en.md)*

Un thème est un dossier qui contient un fichier `Main.qml`, écrit en QML, le langage déclaratif de Qt Quick. Il décide de **tout** ce qui est à l'écran : disposition, couleurs, animations, sons, ordre des menus. Le lanceur fournit les données (consoles, jeux, jaquettes, réglages) et les actions de la manette. Le thème n'a jamais besoin de savoir quel émulateur est utilisé.

Personne n'est obligé de créer un thème pour jouer : « Horizon » est actif par défaut.

## Où sont les thèmes

Dans le dossier `themes` à côté de l'exécutable (`ConsoleLauncher\themes` dans le paquet Windows). Chaque sous-dossier qui contient un `Main.qml` est un thème.

```
themes/mon-theme/
  Main.qml       le thème ; sa racine est une Window
  ...            images, polices, sons, autres fichiers QML : libre
```

Pour travailler depuis un dossier de développement, indiquer le dossier des thèmes dans la variable `CONSOLE_LAUNCHER_QT_THEMES`.

## Partir d'un modèle

1. Copier `themes/grid` (le plus court, environ 120 lignes) ou `themes/horizon` (plus complet).
2. Renommer le dossier.
3. Ouvrir les réglages (Tab / Start) → Thème, jusqu'à voir le nouveau.
4. Modifier `Main.qml`, puis changer de thème et revenir pour le recharger. **F11** bascule le plein écran.

Les erreurs QML s'affichent dans la console ; les réglages restent accessibles avec Tab.

## Thème minimal

```qml
import QtQuick
import QtQuick.Window
import ConsoleLauncher            // composants intégrés (panneau de réglages)

Window {
    visible: true
    width: 1280; height: 720
    color: "#101010"

    property int sel: 0

    Connections {
        target: Launcher
        function onNav(a) {
            if (a === "down") sel = Math.min(Launcher.games.length - 1, sel + 1);
            else if (a === "up") sel = Math.max(0, sel - 1);
            else if (a === "confirm") Launcher.play(Launcher.games[sel].id);
            else if (a === "menu") Launcher.openSettings();
        }
    }

    ListView {
        anchors.fill: parent; anchors.margins: 40
        model: Launcher.games
        delegate: Text { text: modelData.title; color: index === sel ? "gold" : "white"; font.pixelSize: 22 }
    }

    SettingsPanel { anchors.fill: parent }   // ouvert par Launcher.openSettings()
}
```

## L'objet `Launcher`

### Propriétés

| Propriété | Contenu |
|---|---|
| `systems` | Consoles qui ont des jeux (détail plus bas) |
| `games` | Tous les jeux trouvés |
| `running` | `true` pendant une partie (la fenêtre est alors masquée) |
| `padName` | Nom de la manette branchée, ou vide |
| `themes`, `theme` | Thèmes disponibles, thème actif |
| `language` | `fr` ou `en`, choisi dans les réglages |
| `settingsOpen` | `true` quand le panneau de réglages est ouvert |
| `lastError` | Dernier message d'erreur, ou vide |
| `version` | Version du lanceur |

### Signaux

| Signal | Quand |
|---|---|
| `nav(action)` | Manette ou clavier : `up`, `down`, `left`, `right`, `confirm`, `back`, `menu`, `favorite`, `pageLeft`, `pageRight`. Les directions se répètent tant qu'elles sont tenues. Rien n'est envoyé quand le panneau de réglages est ouvert |
| `libraryChanged()` | `systems` et `games` ont été rechargés |
| `gameInfoReady(id, info)` | Réponse à `requestInfo(id)` |
| `favoriteChanged(id, on)` | Réponse à `toggleFavorite(id)` |
| `sessionEnded(report)` | L'émulateur s'est fermé ; `report` contient la fluidité mesurée et le réglage suivant, ou `error` |

### Fonctions

| Fonction | Rôle |
|---|---|
| `requestInfo(id)` | Demande le détail d'un jeu → `gameInfoReady`. `info` vaut `{ready, emulator, emulatorId, tunesSettings, stepLabel, ladder, settled, measure, belowRequirements, notes}` ou `{ready: false, code, emulatorId, error}` (`code` : `emulator_missing`, `no_emulator`…) |
| `play(id)` | Lance le jeu |
| `toggleFavorite(id)` | Favori activé / désactivé |
| `openSettings(cible)` | Ouvre le panneau de réglages intégré. `cible` (facultative) : `{emulator: id}` ouvre la page de cet émulateur, `{page: "systems" \| "emulators" \| "folders" \| "capture"}` une page. À utiliser quand `info.ready` est faux, pour envoyer le joueur directement au bon endroit |
| `request(méthode, paramètres, rappel)` | Appelle n'importe quelle méthode du moteur ([ENGINE_API.fr.md](ENGINE_API.fr.md)) ; `rappel(ok, résultatOuErreur)` |
| `reload()` | Recharge la bibliothèque |
| `setTheme(nom)` | Change de thème |
| `fileUrl(chemin)` | URL d'un fichier local (jaquettes) |
| `asset("img/fond.png")` | URL d'un fichier du dossier du thème |
| `toggleFullscreen()`, `quit()` | — |

### Données

```js
systems: [{
  info: { id: "snes", name: "Super Nintendo", manufacturer: "Nintendo", year: 1990, cost: 1, ... },
  games: 42,                 // nombre de jeux
  emulator: "retroarch",     // identifiant de l'émulateur
  emulator_name: "RetroArch",
  emulator_found: true
}]
games: [{
  id: "6edaa4f992fd1b98",    // stable : favoris et profils y sont liés
  system: "snes",
  title: "Mon jeu",          // nettoyé : « (Europe) », « [!] »… retirés
  path: "...", cover: "...", favorite: false
}]
```

## Composants intégrés

`import ConsoleLauncher` donne accès à :

| Composant | Rôle |
|---|---|
| `SettingsPanel` | Tout l'écran de réglages (consoles, émulateurs, dossiers, mesure de fluidité, langue…), utilisable à la manette. Le placer en dernier dans la fenêtre avec `anchors.fill: parent`. Ses couleurs se changent avec les propriétés `text`, `accent` et `panelColor` |

## Bonnes pratiques

- Réagir seulement à `nav`, ne jamais lire la manette directement : le thème fonctionne alors avec toutes les manettes et au clavier.
- Utiliser `ListView` / `GridView` pour les listes de jeux : ils ne créent que les éléments visibles et restent fluides avec des milliers de jeux.
- Demander `requestInfo` après un court délai pendant le défilement (voir Horizon).
- Ne rien charger depuis Internet : le lanceur doit fonctionner hors ligne.
- Le QML d'un thème a les mêmes droits que l'interface. N'installer que des thèmes de personnes de confiance.
