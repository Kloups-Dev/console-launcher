# Architecture du prototype

## Frontières

`src/Launcher.Core` est indépendant de WPF. Il contient les modèles de bibliothèque, le stockage JSON, la navigation et le lancement de processus. `src/Launcher.Desktop` réalise l'affichage Windows, les dialogues locaux et la lecture XInput.

Le contrat initial est volontairement petit :

- `ConsoleSystem` : identifiant stable, nom, fabricant, repère visuel, description.
- `GameEntry` : identifiant stable, identifiant de console, titre, chemin local facultatif. Un chemin absent désigne un aperçu non lançable.
- `IEmulatorAdapter.Prepare` : valide les prérequis et produit une `LaunchRequest`.
- `LaunchRequest` : exécutable, liste d'arguments séparés et dossier de travail.
- `SessionRunner.RunAsync` : une seule session, attente de fermeture, code de sortie. L'état est libéré même après une erreur de démarrage.

Ne pas composer les arguments dans une chaîne exécutée par un shell. Les chemins de jeux restent des arguments séparés.

## Session Dolphin

1. Vérifier le chemin du jeu et celui de Dolphin.exe.
2. Construire les arguments batch, fichier du jeu et plein écran.
3. Arrêter le polling du menu et masquer le lanceur.
4. Lancer Dolphin directement sur le jeu et attendre sa fermeture.
5. Réafficher la même sélection, y compris si le démarrage échoue.

Le module ne force pas l'arrêt du processus et ne réécrit pas les fichiers de configuration Dolphin. Il n'expose pas encore une touche universelle de fermeture ou un menu de pause en jeu. Les options CLI de plein écran doivent être testées avec la version Dolphin ciblée.

## Stockage

La bibliothèque et le chemin Dolphin sont enregistrés ensemble. Une sauvegarde écrit d'abord un fichier temporaire adjacent, puis remplace le fichier principal. Une bibliothèque illisible est conservée et les nouvelles écritures sont bloquées dans cette session. Les jeux restent à leur emplacement d'origine.

## Répartition proposée avec Claude

Claude peut travailler sur la présentation dans `MainWindow.xaml` et les styles dans `App.xaml` sur sa branche. Préserver les noms `x:Name` et les événements existants. Signaler tout besoin de modifier le contrat ou `MainWindow.xaml.cs` avant de le faire.

Le cœur et la session Dolphin sont la première zone de travail côté Codex. Une prochaine tâche indépendante est le support des manettes en jeu, après discussion de la stratégie et des profils.

Chacun synchronise sa branche avec la proposition de socle avant de commencer. Ne pas développer une seconde application avec un contrat différent.
