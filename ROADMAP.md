# Console Launcher — vision et feuille de route

Ce document décrit **ce que Console Launcher doit devenir**. Il ne décrit pas
les fonctions déjà garanties. L'état réellement disponible se trouve dans
[README.md](README.md), et l'historique publié dans
[CHANGELOG.md](CHANGELOG.md).

## Vision

Console Launcher doit rendre l'émulation compréhensible à une personne qui ne
connaît ni les émulateurs ni leurs réglages. Le résultat recherché n'est pas la
qualité maximale à tout prix, mais une expérience stable et cohérente, proche
de la console d'origine : vitesse correcte, image régulière, son propre,
contrôles prévisibles et retour simple au menu.

Parcours cible :

**installer → ajouter ses jeux → brancher une manette → choisir un jeu → jouer**.

Les options avancées doivent rester accessibles, mais seulement apparaître
lorsqu'elles sont utiles. Le mode automatique doit être explicable, réversible
et ne jamais écraser silencieusement un choix manuel.

## Principes produit

1. **Stabilité avant qualité.** Une résolution native fluide vaut mieux qu'une
   image 4K instable.
2. **Compatibilité avant optimisation.** Les règles propres au jeu et à
   l'émulateur passent avant un score matériel générique.
3. **Deux clics pour l'essentiel.** Les réglages fréquents restent visibles ;
   les réglages rares sont rangés par console et expliqués simplement.
4. **Mesurer au lieu de deviner.** Le matériel donne un point de départ, puis
   une vraie partie confirme ou corrige le choix.
5. **Toujours réversible.** Sauvegarde, retour arrière et priorité aux réglages
   manuels.
6. **Local par défaut.** Jeux, journaux et mesures restent sur la machine sauf
   export volontaire.
7. **Émulateurs indépendants.** L'interface reste commune ; chaque émulateur
   possède un adaptateur testable et remplaçable.
8. **Aucune promesse sans essai réel.** Compiler et réussir des tests ne prouve
   pas qu'une manette, le son ou une partie complète fonctionnent correctement.

## Priorités actuelles

### P1 — Contrôles compréhensibles et testables

Objectif : savoir immédiatement si le clavier ou la manette est reconnu et
comprendre ce que chaque entrée produira sur la console choisie.

Attendu :

- choix entre clavier et manettes détectées ;
- choix de la console cible ;
- représentation de la source à gauche et de la manette d'origine à droite ;
- réaction en direct des boutons, gâchettes, sticks et croix directionnelle ;
- correspondances Xbox, PlayStation, Switch et clavier vers chaque console ;
- zones mortes, calibration, inversion et sensibilité compréhensibles ;
- profils par joueur, appareil et console ;
- branchement et débranchement à chaud sans bloquer l'interface ;
- sauvegarde puis application par l'adaptateur de l'émulateur ;
- restauration du profil conseillé.

Terminé lorsque :

- le clavier et au moins une manette Xbox, PlayStation et Switch ont été testés
  physiquement sous Windows ;
- les entrées affichées correspondent à celles reçues dans Dolphin et dans au
  moins un émulateur RetroArch ;
- fermer le testeur rend toujours la navigation normale au lanceur ;
- aucun texte tapé ailleurs ni aucune entrée globale n'est enregistré.

État : première visualisation en cours sur
`feature/controller-visualizer`. La persistance et l'application aux
émulateurs restent à développer.

### P2 — Diagnostic en jeu et overlay discret

Objectif : voir et enregistrer ce qui se passe réellement pendant une partie,
sans ajouter une fenêtre technique envahissante.

Attendu :

- raccourci configurable parmi les touches de fonction disponibles ;
- overlay activable avec FPS, vitesse d'émulation, temps d'image et état de la
  mesure ;
- vue étendue facultative : 1 % bas, saccades, CPU, GPU, mémoire, température
  si disponible et version de l'émulateur ;
- indicateurs audio si l'émulateur expose les pertes ou l'état du tampon ;
- commandes sûres : afficher, masquer, arrêter la mesure, revenir au lanceur ;
- design neutre, lisible et désactivable qui ne masque pas le jeu.

Journal local attendu :

- identité du jeu, de l'émulateur, du pilote et du profil appliqué ;
- matériel utile, durée de session, code de sortie et erreurs ;
- mesures bornées avec rotation des fichiers ;
- aucune frappe, aucun texte saisi, aucun secret et aucun envoi automatique ;
- export manuel partageable avec réduction des chemins personnels.

Terminé lorsque l'overlay et le journal décrivent la même session, n'altèrent
pas sensiblement la fluidité et permettent d'expliquer : émulateur absent,
mauvais profil, vitesse insuffisante, frame pacing irrégulier, échec de capture
ou fermeture inattendue.

État local au 25 octobre 2026 : premier panneau Windows F8 implémenté pour
les mesures de présentation et les ressources du processus. Tests automatisés
et démarrage validés. Vitesse d'émulation et état audio non mesurés ; essais
réels, plein écran exclusif, mesures GPU/température et équivalent Linux à faire.
Cette première version ne remplit pas encore les critères de fin de P2.
### P3 — Réglage automatique orienté stabilité

Objectif : produire automatiquement la configuration la plus stable pour le PC,
le jeu et l'émulateur concernés.

Ordre de décision :

1. identité exacte du jeu et règles de compatibilité connues ;
2. version de l'émulateur et du pilote ;
3. matériel et système d'exploitation ;
4. profil prudent de départ ;
5. mesure réelle en jeu ;
6. amélioration progressive ou retour au dernier profil stable.

Attendu :

- critères distincts pour vitesse, régularité de l'image et santé audio ;
- profil séparé par jeu, révision, émulateur, pilote et matériel ;
- période de chauffe pour la compilation des shaders ;
- invalidation d'un ancien résultat après un changement important ;
- arrêt de la recherche dès qu'un profil fiable est trouvé ;
- réglage manuel verrouillable et retour arrière immédiat.

Terminé lorsque plusieurs machines modestes, moyennes et puissantes reproduisent
le même protocole sur plusieurs jeux par console, avec résultats consignés. Une
session fluide ne suffit pas à déclarer toute une console validée.

### P4 — Adaptateurs d'émulateurs fiables

Chaque adaptateur devra définir :

- versions prises en charge et méthode de détection ;
- téléchargement officiel et vérification d'intégrité ;
- commande de lancement et fermeture propre ;
- réglages modifiables, valeurs conseillées et sauvegarde ;
- capacités de contrôle, mesure, overlay et journal ;
- particularités par jeu et limites connues ;
- tests automatisés et protocole d'essai réel.

Ordre proposé : Dolphin, RetroArch, PCSX2, DuckStation, PPSSPP, mGBA, melonDS,
Cemu, puis de nouveaux émulateurs. La Switch demandera une étude légale et
technique séparée et ne doit pas être annoncée avant un socle maintenu et testé.

### P5 — Bibliothèque riche et fiable

Attendu :

- identification locale indépendante du nom du fichier ;
- titre, région, révision, date, éditeur, développeur, genres et jaquettes ;
- fusion de fournisseurs avec provenance de chaque champ ;
- correction manuelle prioritaire et persistante ;
- cache hors ligne, limite de taille et reprise après une panne réseau ;
- couverture progressive de toutes les consoles prises en charge ;
- import compatible RetroBat, Batocera et EmulationStation sans déplacer les
  jeux.

Un fournisseur en panne ne doit jamais empêcher de scanner ou lancer un jeu.

### P6 — Correctifs et améliorations facultatives

Attendu :

- catalogue lié à une identité, une région et une révision exactes ;
- source, auteur, licence, checksum, émulateurs testés et incompatibilités ;
- catégories : 60 FPS, écran large, traduction, textures et corrections ;
- coût matériel estimé ;
- sauvegarde et retour arrière ;
- activation explicite des modifications du comportement d'origine ;
- profil de stabilité séparé avec et sans patch.

Le lanceur ne doit jamais proposer un fichier dont la licence ou le droit de
redistribution n'est pas clair.

### P7 — Installation et première utilisation

Attendu :

- installateur Windows signé lorsque le projet sera prêt ;
- paquet Linux maintenable ;
- assistant initial : langue, jeux, périphérique principal et consoles ;
- installation guidée des émulateurs et composants nécessaires ;
- versions visibles, mises à jour explicites, épinglage et retour arrière ;
- diagnostic des dépendances manquantes avec une action directe ;
- migration entre versions sans perdre la bibliothèque.

### P8 — Thèmes et mode salon

La base fonctionnelle doit rester utilisable sans thème. Ensuite :

- mode bureau et mode plein écran/salon ;
- thèmes déclaratifs validés, sans code arbitraire ;
- variables communes de couleurs, typographie, espacements et composants ;
- navigation identique quel que soit le thème ;
- import, aperçu, activation et suppression d'un thème ;
- accessibilité : contraste, taille du texte, focus et animations réduites.

Un thème ne doit jamais modifier le moteur, lire les jeux ou exécuter du code
système.

## Ordre de réalisation recommandé

1. terminer et tester le visualiseur de contrôles ;
2. sauvegarder les mappings puis les appliquer à Dolphin ;
3. créer le journal de session commun ;
4. ajouter l'overlay minimal et mesurer son coût ;
5. tester GameCube/Dolphin sur plusieurs jeux et PC ;
6. corriger le modèle de stabilité avec ces mesures ;
7. généraliser le contrat aux autres adaptateurs ;
8. étendre métadonnées, patches, installation et thèmes.

Ce chemin évite d'ajouter beaucoup de consoles avant de savoir diagnostiquer
correctement une chaîne complète.

## Définition générale de « terminé »

Une fonctionnalité est terminée seulement si :

- son parcours et ses erreurs sont compréhensibles sans documentation technique ;
- clavier et souris fonctionnent, ainsi que la manette lorsqu'elle est concernée ;
- elle ne bloque pas l'interface pendant une opération lente ;
- elle possède des tests adaptés et un essai réel lorsque nécessaire ;
- elle conserve les données et les choix existants ;
- elle journalise une erreur utile sans exposer de secret ;
- elle fonctionne dans le paquet distribué, pas seulement après compilation ;
- le README, le changelog et cette roadmap reflètent son état réel.

## Hors périmètre actuel

- fournir des ROM, BIOS, firmwares ou clés ;
- contourner les protections d'une console ou d'un service ;
- promettre qu'un jeu sera parfait sur toute machine ;
- réécrire les émulateurs existants ;
- réseau social, boutique ou téléchargement de contenus protégés ;
- réglages automatiques opaques et impossibles à annuler.

## Décisions encore ouvertes

- nom définitif du produit ;
- licence du dépôt ;
- fournisseurs de métadonnées par console ;
- technologie exacte de l'overlay Windows/Linux ;
- format public des thèmes ;
- politique de signature et de mise à jour ;
- matrice minimale de PC, manettes et jeux avant une version 1.0.
