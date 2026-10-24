# Optimisation automatique

*[English version](PERFORMANCE.en.md)*

Objectif : que chaque jeu tourne comme sur sa console, à pleine vitesse, sans saccades ni coupures audio. La qualité supplémentaire est facultative et ne doit jamais dégrader cette stabilité.

## Le principe stabilité-d'abord

1. **Départ prudent.** Dolphin commence au rendu natif, avec un moteur graphique choisi selon le GPU. L'estimation matérielle reste utile aux autres adaptateurs, mais elle ne justifie jamais à elle seule le mot « stable ».
2. **Conditions réelles.** MangoHud (Linux) ou PresentMon (Windows) mesure la régularité des images pendant 60 secondes, après 20 secondes de chargement.
3. **Certification.** Trois preuves indépendantes sont prévues : régularité vidéo, vitesse réelle d'émulation et santé du tampon audio. Une preuve absente donne « mesures insuffisantes », pas un faux succès. Une anomalie connue donne « instable ».
4. **Ajustement prudent.** Un profil instable descend d'un palier. Un profil certifié reste au palier testé : le lanceur ne monte jamais automatiquement la résolution. L'utilisateur peut toujours choisir manuellement une qualité supérieure.

Le résultat est propre à chaque jeu et enregistré dans `profiles/<id>.json`. Si la machine change (autre processeur ou autre carte graphique), ou si une politique automatique incompatible est corrigée, la recherche recommence.

## Échelles de qualité (v0.1)

| Émulateur | Paliers, du plus léger au plus lourd | Ce qui change |
|---|---|---|
| Dolphin (GameCube, Wii) | Natif · 720p · 1080p · 1440p · 4K | Résolution interne (×1 à ×6) ; ubershaders hybrides au-dessus du natif pour éviter les saccades de compilation |
| PCSX2 (PS2) | Natif · 720p · 1080p · 1440p · 4K | Multiplicateur de résolution (×1 à ×6) |
| RetroArch (consoles légères) | Eco · Standard · Faible latence · Latence minimale | Run-ahead (1 à 2 images) : moins de latence, en échange de plus de calcul CPU |
| RetroArch (N64, DS, PSP) | Eco · Standard | Run-ahead désactivé, trop coûteux |

L'API graphique est choisie automatiquement selon l'émulateur, le système et le fabricant du GPU. Pour Dolphin sous Windows : Vulkan est privilégié sur AMD, D3D12 sur Intel et D3D11 sur NVIDIA afin d'éviter le couple Hybrid/Vulkan connu pour saccader sur ce pilote. Un fabricant inconnu reçoit le choix prudent D3D11. Sous Linux, OpenGL est préféré à Vulkan pour Intel et NVIDIA dans le profil stabilité-d'abord ; AMD conserve Vulkan.

Le palier natif de Dolphin utilise les shaders spécialisés, adaptés aux GPU modestes. Les paliers supérieurs utilisent les ubershaders hybrides. Le mode « Skip Drawing » est exclu des choix automatiques car il gagne du temps en ne dessinant pas les objets dont le shader n'est pas prêt. Le lanceur ne force pas non plus le double cœur : Dolphin conserve ainsi ses exceptions de compatibilité propres à chaque jeu.

## Critères vidéo et certification

Une session est jugée **fluide** si :
- la moyenne atteint au moins 97 % de la fréquence de la console (60 ou 50 images/s) ;
- le « 1 % bas » (l'image la plus lente parmi les 99 % meilleures) atteint au moins 85 % de cette fréquence ;
- moins de 1 % des images durent plus de 1,5 fois le temps prévu.

Pendant une mesure, l'émulateur affiche chaque rafraîchissement de la console (Dolphin reçoit une option pour ne pas sauter les images en double). Sans cela, un jeu prévu pour 30 images/s et un jeu qui tourne à mi-vitesse donneraient la même mesure.

Sous Windows, PresentMon cible uniquement l'identifiant du processus que le lanceur vient de démarrer. Il ne suit pas les entrées clavier/souris. S'il ne produit aucun CSV, son court journal local est repris dans le compte rendu de la partie afin que l'échec soit explicable au lieu d'être silencieux.

Ces seuils donnent uniquement le verdict vidéo « fluide » ou « saccades ». Depuis la politique stabilité-d'abord, ce verdict seul ne certifie pas le jeu : la vitesse interne et les sous-alimentations du tampon audio doivent aussi être connues. Leur collecte automatique est la prochaine étape ; en attendant, une bonne capture vidéo produit correctement « mesures insuffisantes ».

Ces seuils sont **des valeurs de départ**. Ils seront confirmés ou corrigés après les premiers essais.

## Respect des réglages de l'utilisateur

- Émulateur **installé par l'utilisateur** : ses fichiers ne sont jamais modifiés. Les réglages passent par la ligne de commande et ne valent que pour la session (RetroArch `--appendconfig`, Dolphin `--config`). PCSX2 n'a pas cette possibilité : il garde alors ses propres réglages.
- Émulateur **géré par le lanceur** : le fichier INI est modifié clé par clé. Le lanceur retient ce qu'il a écrit. Si l'utilisateur change une valeur à la main, le lanceur la garde ensuite.
- `locked_keys` dans `config.toml` : liste de réglages que le lanceur ne touche jamais.
- Choisir un palier à la main (`clctl set-step` ou `Launcher.setQuality`) arrête la recherche pour ce jeu.

## Calibrer (à faire avec les premiers testeurs)

Les valeurs de l'estimation (`crates/cl-perf/src/estimate.rs`) et la référence du test CPU (`REFERENCE_NS` dans `crates/cl-hwprobe/src/bench.rs`) n'ont pas encore été mesurées sur de vraies machines. Protocole proposé :

1. Sur chaque machine de test : `clctl probe --json > machine.json`.
2. Pour 3 à 5 jeux par console : jouer avec la mesure activée et conserver le palier natif tant que les trois preuves ne sont pas disponibles.
3. Récupérer `hardware.json` et `profiles/*.json` (ils contiennent les statistiques de chaque session).
4. Comparer le palier estimé au palier final. Ajuster `required_cpu` et `gpu_budget` pour que l'estimation tombe juste dans la majorité des cas.
5. Consigner les machines, les jeux et les résultats dans un tableau partagé avant de modifier les seuils.

Mesurer une capture faite à la main :

```bash
mangohud --dlsym <émulateur> ...      # Linux, journal avec Maj+F2
clctl analyse capture.csv --game "nom du jeu"
```

## Points à vérifier sur les vraies versions des émulateurs

- Dolphin : confirmer sur la version réellement fournie le nom de la clé de mesure `SkipPresentingDuplicateFrames`.
- PCSX2 : numéros de `Renderer` (14 = Vulkan, 3 = D3D11, 15 = D3D12, 12 = OpenGL) et lancement avec `-batch -nogui`.
- PresentMon 2.x : options de ligne de commande utilisées (`--process_name`, `--timed`, `--terminate_after_timed`, `--v1_metrics`).
- MangoHud : options `autostart_log` et `log_duration`.

## Limites connues

- Le premier lancement d'un jeu peut saccader pendant la compilation des shaders. Les 60 premières images sont ignorées et la mesure commence après 20 s, mais un jeu qui compile beaucoup peut être jugé trop sévèrement la première fois.
- PresentMon et MangoHud ne voient que ce que l'écran affiche. La structure de certification refuse donc de conclure « stable » tant que la vitesse interne et l'audio ne sont pas encore collectés.
- Sous Windows, PresentMon a besoin d'une autorisation : **Réglages › Mesure de fluidité › Autoriser la mesure**, puis fermer et rouvrir la session Windows (voir [DEPENDENCIES.fr.md](DEPENDENCIES.fr.md)).
