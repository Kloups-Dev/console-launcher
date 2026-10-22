# Console Launcher — nom provisoire

Lanceur d'émulation PC à installer, utilisable à la manette, avec une interface originale et fluide.

Parcours : **menu → console → jeu → partie → retour au menu**. Les bibliothèques des émulateurs ne sont pas des étapes visibles.

## État actuel

Base de collaboration et spécification initiale. Il n'y a pas encore d'application exécutable. Windows est la première cible proposée ; la technologie et la licence du projet restent à décider. Ne pas annoncer ce dépôt comme un logiciel open source sous licence tant que celle-ci n'est pas choisie.

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
2. Réserver une tâche dans une issue GitHub et indiquer les fichiers concernés.
3. Créer une branche `feat/nom-de-la-tache` ou `fix/nom-du-probleme`.
4. Développer et vérifier le changement.
5. Ouvrir une pull request et faire relire par l'autre personne.
6. Fusionner, puis mettre à jour l'autre copie avant la tâche suivante.

Éviter que les deux IA changent les mêmes fichiers simultanément. Toute modification des interfaces communes se décide dans l'issue avant implémentation.

Guide de mise en ligne : `docs/GITHUB-SETUP.md`.
