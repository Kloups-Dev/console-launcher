# Vérification du prototype

## Résultats locaux du 22 octobre 2026

- Compilation Release : 0 erreur, 0 avertissement.
- 23 vérifications réussies, dont exécution d'un processus enfant réel.
- Démarrage et rendu de la fenêtre WPF en image : sortie normale, disposition inspectée.
- Publication d'un dossier Windows dépendant du runtime .NET Desktop 8.

Ces résultats concernent le prototype local, pas encore l'exécution des vérifications GitHub.

## Vérifications automatisées

Le programme `tests/Launcher.Checks` est un exécutable de tests sans dépendance tierce. Il échoue avec une exception si une vérification ne passe pas.

Il vérifie la navigation et ses limites, l'import sans doublon, la conservation des chemins et des réglages, le refus des formats non pris en charge, le chargement d'un fichier JSON invalide sans destruction, la validation du module Dolphin et ses arguments. Il lance également un vrai processus enfant contrôlé pour vérifier les arguments contenant des espaces et caractères spéciaux, le refus des sessions concurrentes et la libération de session après fermeture ou échec.

Le mode `--preview chemin.png` du bureau rend son propre arbre WPF dans une image puis ferme la fenêtre. Il permet de vérifier la disposition sans capturer le bureau. `--data-dir chemin` permet d'isoler la bibliothèque de test.

## À vérifier avec du matériel et un vrai jeu

- Manette XInput en USB et Bluetooth : navigation, maintien de direction et reconnexion.
- Dolphin : vraie image GameCube, plein écran, son, contrôles et retour au menu.
- Dolphin portable et installation standard ; versions exactes consignées au test.
- Focus et écrans multiples, mise à l'échelle Windows, perte de périphérique.
- Menus de réglages encore dépendants du clavier/souris.
- Mesures de fluidité et comparaison avec RetroBat à configuration identique.

Les résultats de ces essais ne sont pas acquis par la compilation ou par les tests du processus enfant. Aucun gain de performance n'est annoncé.
