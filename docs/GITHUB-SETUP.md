# Mettre le projet sur GitHub

## Création par le propriétaire

1. Se connecter sur https://github.com.
2. Créer un dépôt via https://github.com/new.
3. Choisir son nom ; `console-launcher` est seulement un nom provisoire.
4. Choisir privé pour préparer le projet, ou public si son contenu doit être visible immédiatement.
5. Ne pas ajouter de README, .gitignore ou licence à cette étape : la base locale existe déjà, et la licence n'est pas encore décidée.
6. Copier l'adresse du dépôt et la transmettre à l'assistant pour connecter et publier la base locale.

Alternative : connecter GitHub CLI avec `gh auth login`, dans un terminal personnel. Ne jamais coller de jeton ou mot de passe dans le chat.

## Inviter le second contributeur

Dans les paramètres du dépôt, ouvrir Collaborators et inviter le compte GitHub du collègue. Il accepte l'invitation et clone le dépôt sur son PC.

Pour un projet appartenant durablement aux deux personnes, une organisation GitHub peut ensuite remplacer le dépôt personnel. Ce n'est pas nécessaire pour démarrer.

## Donner le contexte aux deux IA

Ouvrir le dossier cloné dans l'application de chaque assistant. Message de départ :

> Lis README.md, AGENTS.md, docs/DECISIONS.md et docs/TASKS.md. Consulte les tâches en cours avant de modifier les fichiers. Travaille sur une branche dédiée à la tâche qui t'est attribuée. Signale les décisions nécessaires et les vérifications réalisées.

Les assistants n'ont pas automatiquement accès à GitHub : connecter ou authentifier leurs outils selon l'application utilisée. Une copie locale du dépôt leur permet déjà de lire et modifier les fichiers.

## Après connexion du dépôt

- Créer les premières issues à partir de docs/TASKS.md.
- Faire les modifications par pull requests.
- Activer les protections de branche disponibles pour le compte et la visibilité choisis.
- Ajouter les vérifications automatiques lorsque la technologie et le premier code sont en place.
- Choisir une licence avant d'annoncer une publication open source.
