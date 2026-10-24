# Métadonnées, jaquettes et correctifs

## Objectif

Une bibliothèque doit reconnaître une copie locale sans dépendre de son nom de
fichier, afficher une fiche riche hors ligne après la première synchronisation,
et fournir une identité assez précise pour l'optimisation et les correctifs par
jeu. Aucun fournisseur distant ne doit être nécessaire pour lancer un jeu.

## Identification

L'ordre de confiance est :

1. identifiant ou numéro de série lu dans le jeu (disques et certaines
   cartouches) ;
2. checksum du contenu pour les petites images de cartouche ;
3. nom normalisé, uniquement comme proposition à confirmer.

Les conteneurs compressés doivent être lus par un lecteur adapté au format ou
par l'émulateur. Il ne faut pas calculer le checksum du conteneur et le présenter
comme celui du jeu. L'identité contient toujours la console et, si possible, la
révision et la région : deux éditions d'un titre peuvent nécessiter des patches
différents.

Le moteur possède maintenant `GameIdentity` (`serial`, `crc32`, taille) et un
cache `metadata/catalogue.json`. Les ISO/GCM GameCube/Wii bruts, les fichiers NDS
et les petites cartouches constituent le premier lot pris en charge.

## Fournisseurs

Le lanceur utilise des adaptateurs interchangeables et fusionne leurs résultats
dans un modèle commun `GameMetadata`.

- **Title Screen** : premier fournisseur réellement branché. Son API publique,
  sans clé, permet une recherche par checksum, série ou titre et fournit les
  éditions, dates, studios, éditeurs, régions et genres. Les résultats sont mis
  en cache localement. Données Title Screen sous licence CC BY 4.0.
- **Libretro Database** : identification locale, titres normalisés et données
  rétro. Source ouverte à intégrer en premier sous forme de paquet de données
  versionné.
- **GameTDB** : excellent complément GameCube/Wii, Nintendo DS et Wii U par identifiant de jeu,
  avec fiches localisées et illustrations. Le projet doit être crédité et
  déclaré auprès de GameTDB avant distribution.
- **Libretro Thumbnails** : jaquettes, logos et captures. Les images sont mises
  en cache à la demande ; elles ne sont pas incluses aveuglément dans le paquet.
- **IGDB** : complément riche pour les jeux moins bien couverts. Son secret
  d'application ne doit jamais être embarqué dans l'exécutable ; il exige un
  petit service relais ou des identifiants fournis par l'utilisateur.
- **ScreenScraper** : fournisseur optionnel pour les utilisateurs possédant les
  identifiants requis. Les quotas et les conditions sont respectés.

Chaque champ conserve sa source, son identifiant distant, sa date de mise à jour
et son niveau de confiance. Les corrections manuelles de l'utilisateur ont
toujours priorité et ne sont jamais écrasées.

## Téléchargement et cache

- La synchronisation est explicite, annulable et relançable.
- Les réponses et images sont mises en cache avec une limite de taille.
- Une panne réseau conserve la dernière fiche valide.
- Les images sont téléchargées en arrière-plan après l'affichage de la
  bibliothèque.
- Les secrets ne sont jamais écrits dans les journaux ni dans `config.toml`.
- Une page Sources et licences crédite chaque fournisseur.

La sélection d'un jeu sans fiche lance automatiquement une seule recherche en
arrière-plan après un court délai ; **Actualiser/Réessayer la fiche** permet de
la relancer manuellement. Aucun téléchargement global n'est effectué au
démarrage. Pour GameCube/Wii, Nintendo DS et Wii U, cette action récupère la fiche
Title Screen puis la jaquette avant GameTDB correspondant à l'identifiant et à
l'édition régionale du jeu. Le PNG ou JPEG est contrôlé, limité à 10 Mo et conservé
sous `metadata/artwork/<console>/`. Un scan,
un démarrage ou un lancement de jeu reste entièrement local. Title Screen et
GameTDB restent deux sources distinctes dans le cache. L'interface n'affiche
jamais directement l'URL distante : une image doit d'abord avoir été contrôlée
et enregistrée localement. En cas d'échec, elle indique que l'actualisation peut
être retentée.

## Correctifs et patches

Un patch n'est jamais déduit seulement du titre. Il cible une identité exacte :
console, série/checksum, région et révision. Son manifeste indique la source,
l'auteur, la licence, le type (60 FPS, écran large, traduction, textures), les
versions d'émulateur testées, le coût matériel, les incompatibilités, le moyen de
retour arrière et un checksum du fichier appliqué.

Les réglages de compatibilité officiels de l'émulateur sont appliqués avant la
pyramide de qualité. Les patches qui changent le comportement du jeu, notamment
60 FPS, restent désactivés par défaut et nécessitent un choix explicite. Le
lanceur teste ensuite leur stabilité dans un profil séparé : un résultat obtenu
sans patch ne valide jamais automatiquement le même jeu patché.

## Lien avec l'optimisation

La clé d'identité du jeu devient la clé commune pour :

- la fiche et la jaquette ;
- les contraintes de compatibilité ;
- les patches disponibles ;
- le profil de performance mesuré ;
- la version de l'émulateur, du pilote et du patch ayant produit le résultat.

Le score matériel ne choisit qu'un niveau de départ. La meilleure configuration
est celle réellement mesurée comme stable pour cette identité précise.
