# Console Launcher — nom provisoire

Lanceur d'émulation PC à installer, utilisable à la manette, avec une interface originale et fluide.

Parcours : **menu → console → jeu → partie → retour au menu**. Les bibliothèques des émulateurs ne sont pas des étapes visibles.

## État actuel

Premier prototype Windows proposé : menu natif C#/.NET 8 et WPF, navigation clavier/XInput, bibliothèque locale et lancement direct GameCube avec Dolphin déjà installé. La technologie reste à relire et la licence du projet reste à choisir. Ne pas annoncer ce dépôt comme un logiciel open source sous licence tant que celle-ci n'est pas choisie.

## Essayer le prototype

Prérequis de développement : Windows et SDK .NET 8, version 8.0.300 ou ultérieure dans la série 8.

```powershell
dotnet run --project src/Launcher.Desktop/Launcher.Desktop.csproj
```

- Flèches : choisir une console puis un jeu.
- Entrée ou bouton A : choisir ; Échap ou bouton B : retour.
- F11 : plein écran. Souris et tabulation également disponibles.
- Réglages : sélectionner votre Dolphin.exe existant.
- Ajouter un jeu : choisir votre fichier GameCube local, puis Jouer.
- Fermer la partie dans Dolphin : retour au lanceur.

Les jeux affichés avant importation sont des titres fictifs et ne se lancent pas. PS2 et SNES sont des aperçus visuels, sans adaptateur fonctionnel pour le moment. Les manettes prises en charge dans le menu sont les modèles exposés via XInput ; les réglages de contrôles en jeu restent ceux de Dolphin. Les réglages et l'ajout de fichiers nécessitent encore le clavier ou la souris.

Bibliothèque du lanceur : `%LOCALAPPDATA%/ConsoleLauncher/library.json`. Les fichiers de jeux ne sont ni déplacés ni copiés. Le contenu du jeu n'est pas identifié ou validé lors de l'import : Dolphin le vérifie au démarrage. Ce prototype ne télécharge ni émulateur, ni jeu, ni BIOS.

## Vérifier et publier un dossier portable

```powershell
dotnet build src/Launcher.Desktop/Launcher.Desktop.csproj --configuration Release
dotnet run --project tests/Launcher.Checks/Launcher.Checks.csproj --configuration Release
dotnet publish src/Launcher.Desktop/Launcher.Desktop.csproj --configuration Release --self-contained false -o dist/windows
```

Le dossier publié nécessite le runtime .NET Desktop 8 sur le PC cible. Il ne constitue pas encore un installateur autonome.

Architecture et frontières de contribution : `docs/ARCHITECTURE.md`. État des vérifications : `docs/VALIDATION.md`.

## Objectifs

- Consoles accessibles de gauche à droite, présentation libre sans reproduction obligatoire de la PS3.
- Jeux classés, jaquettes et favoris.
- Configuration guidée, options compréhensibles et réglages avancés accessibles.
- Manette dans l'interface et dans les jeux pris en charge.
- Émulateurs existants, intégrés par adaptateurs.
- Mises à jour validées et retour à la version précédente.

## Travailler à deux avec des IA

Chaque personne clone le dépôt sur son PC. Chaque IA travaille dans cette copie et sur une branche dédiée. GitHub conserve le projet et les échanges ; il ne relie pas automatiquement les conversations des IA.

1. Lire `AGENTS.md`, `docs/DECISIONS.md` et `docs/TASKS.md`.
2. Indiquer le travail entrepris dans une issue GitHub pour rendre les changements visibles.
3. Créer une branche `feat/nom-de-la-tache` ou `fix/nom-du-probleme`.
4. Développer et vérifier le changement.
5. Ouvrir une pull request et faire relire par l'autre personne.
6. Fusionner, puis mettre à jour l'autre copie avant la tâche suivante.

Chaque assistant est libre de contribuer dans le périmètre défini par les humains. Aucun assistant ne distribue les rôles à l'autre et aucun fichier n'est réservé à une IA. Les branches, les échanges et les revues servent à gérer les éventuels conflits et à comparer les propositions.

Guide de mise en ligne : `docs/GITHUB-SETUP.md`.
