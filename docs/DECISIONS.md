# Décisions

## Besoin confirmé

Mode de collaboration précisé par l'utilisateur : chaque assistant travaille de façon autonome dans le périmètre fixé par les humains. Aucun assistant n'attribue de rôle ou de chemin technique à l'autre. Les propositions techniques restent révisables.

- Application installable sur PC.
- Navigation jolie et simple à la manette, consoles de gauche à droite.
- Sélection directe du jeu ; pas de passage dans la bibliothèque d'un émulateur.
- Réglages simplifiés et accessibles, pas interdiction des paramètres.
- Fluidité et intégration soignée.
- Projet partagé entre deux personnes et leurs assistants.
- Intention de distribuer le projet en open source ; licence exacte à choisir.

## Propositions à valider

- Windows en première cible.
- Séparation interface / bibliothèque / gestionnaire de session / adaptateurs.
- Comparer un prototype Pegasus et RetroBat avant de décider d'une interface entièrement dédiée.
- Trois intégrations représentatives à terme : RetroArch, Dolphin, PCSX2.
- Canal de mises à jour validé et retour arrière.

## Pas encore décidé

- Nom définitif et licence.
- Technologie de l'interface et du moteur.
- Première console et premier émulateur.
- PC minimal et objectifs de performance mesurables.

Consigner ici les nouvelles décisions avec date, raison et alternatives évaluées.

## 2026-10-22 — Proposition implémentée pour le premier prototype

L'utilisateur a autorisé le démarrage du travail. Cette branche propose un prototype Windows en C#/.NET 8 avec WPF, sans bibliothèque NuGet tierce. Le SDK 8 est disponible sur le poste de développement. Ce choix permet un exécutable local et une interface native ; ce n'est pas une affirmation de supériorité de performance. La cible .NET doit être réévaluée avant une distribution durable, selon les versions encore supportées.

La proposition doit être relue avant fusion. Elle remplace, pour ce prototype seulement, la proposition de commencer dans Pegasus. Le cœur ne dépend pas de WPF et peut être conservé si l'interface change. Aucun résultat ne démontre encore un avantage par rapport à RetroBat.

Premier adaptateur : Dolphin pour GameCube, en mode batch avec lancement direct et plein écran. La configuration et les sauvegardes personnelles restent celles de Dolphin. Le prototype ne configure pas encore les manettes en jeu et n'installe aucun émulateur.

La navigation du menu utilise le clavier et XInput sous Windows. Une DualSense native ou une Wiimote ne sont pas couvertes par cette première couche d'entrée.

Sources : [WPF](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/overview/), [API XInput](https://learn.microsoft.com/en-us/windows/win32/xinput/getting-started-with-xinput), [options Dolphin](https://github.com/dolphin-emu/dolphin/blob/master/Source/Core/UICommon/CommandLineParse.cpp).
