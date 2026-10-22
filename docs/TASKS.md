# Premières tâches

Créer une issue GitHub par tâche. Réserver une tâche par commentaire avant de commencer et préciser les fichiers concernés.

## Travail démarré — branche feat/desktop-foundation

Socle proposé par Codex : interface Windows WPF, bibliothèque JSON, navigation clavier/XInput, contrat d'adaptateur et lancement direct Dolphin. Voir docs/ARCHITECTURE.md et docs/VALIDATION.md. Ne pas recréer un second socle en parallèle ; relire cette proposition avant de partir de sa branche.

Prochaine tâche proposée à Claude : améliorer la présentation de MainWindow.xaml et App.xaml, préserver les noms et événements existants, ne pas modifier le cœur ni le gestionnaire de session. Cette tâche doit commencer depuis le socle publié et être réalisée dans sa propre branche.

## 1 — Définir la première version

Résultat : OS initial, premier émulateur, matériel de test et critères d'acceptation décidés. Documenter les décisions sans élargir le périmètre.

## 2 — Prototype de navigation

Après choix de technologie : consoles horizontales, liste de jeux locale de démonstration, navigation clavier/manette et retour cohérent. Aucune intégration d'émulateur requise pour cette tâche. Médias originaux ou libres seulement.

## 3 — Contrat d'adaptateur

Décrire avec l'autre contributeur les données d'un jeu, les prérequis, la configuration et le cycle de session. Stabiliser le contrat avant les implémentations parallèles.

## 4 — Première session réelle

Après contrat : lancer directement un jeu fourni par le testeur, suivre sa fermeture et revenir au menu. Conserver les sauvegardes et configurations existantes.

## 5 — Comparaison de fluidité

Même PC, émulateur, jeu, réglages et état du cache. Mesurer menu, lancement, retour et consommation pendant le jeu. Les seuils sont définis avant de conclure.

Répartition suggérée après les tâches 1 et 3 : un binôme sur l'interface ; l'autre sur l'adaptateur. Les fichiers communs ont un responsable explicite par tâche.
