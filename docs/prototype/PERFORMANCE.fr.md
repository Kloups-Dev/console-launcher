# Optimisation automatique

*[English version](PERFORMANCE.en.md)*

Objectif : que chaque jeu tourne à pleine vitesse, sans saccades, avec la meilleure qualité possible sur la machine, et sans que l'utilisateur ait à régler quoi que ce soit.

## Le principe en trois étapes

1. **Simulation, avant la première partie.** Le lanceur détecte le processeur, la mémoire et la carte graphique (via Vulkan), puis lance un test CPU d'environ une seconde. Ce test fait tourner un petit interpréteur de code, le même genre de travail que l'émulation d'un processeur. Avec ces données, il choisit un palier de départ sur « l'échelle de qualité » de l'émulateur.
2. **Conditions réelles, pendant les parties.** Tant que le bon réglage n'est pas trouvé, la partie est mesurée : MangoHud (Linux) ou PresentMon (Windows) enregistre la durée de chaque image pendant 60 secondes, après 20 secondes de chargement.
3. **Ajustement.** Si la partie a été fluide, le lanceur essaie le palier supérieur à la partie suivante. Si elle a saccadé, il revient au dernier palier fluide (ou descend d'un cran). Il s'arrête quand un palier fluide se trouve juste sous un palier qui saccade. Cela prend au plus autant de parties que l'échelle a de paliers (5 en général), et souvent 1 ou 2.

Le résultat est propre à chaque jeu et enregistré dans `profiles/<id>.json`. Si la machine change (autre processeur ou autre carte graphique), la recherche recommence.

## Échelles de qualité (v0.1)

| Émulateur | Paliers, du plus léger au plus lourd | Ce qui change |
|---|---|---|
| Dolphin (GameCube, Wii) | Natif · 720p · 1080p · 1440p · 4K | Résolution interne (×1 à ×6) ; ubershaders hybrides au-dessus du natif pour éviter les saccades de compilation |
| PCSX2 (PS2) | Natif · 720p · 1080p · 1440p · 4K | Multiplicateur de résolution (×1 à ×6) |
| RetroArch (consoles légères) | Eco · Standard · Faible latence · Latence minimale | Run-ahead (1 à 2 images) : moins de latence, en échange de plus de calcul CPU |
| RetroArch (N64, DS, PSP) | Eco · Standard | Run-ahead désactivé, trop coûteux |

L'API graphique est choisie automatiquement : Vulkan si disponible, sinon Direct3D sous Windows ou OpenGL.

## Critères de fluidité

Une session est jugée **fluide** si :
- la moyenne atteint au moins 97 % de la fréquence de la console (60 ou 50 images/s) ;
- le « 1 % bas » (l'image la plus lente parmi les 99 % meilleures) atteint au moins 85 % de cette fréquence ;
- moins de 1 % des images durent plus de 1,5 fois le temps prévu.

Pendant une mesure, l'émulateur affiche chaque rafraîchissement de la console (Dolphin reçoit une option pour ne pas sauter les images en double). Sans cela, un jeu prévu pour 30 images/s et un jeu qui tourne à mi-vitesse donneraient la même mesure.

Ces seuils sont **des valeurs de départ**. Ils seront confirmés ou corrigés après les premiers essais.

## Respect des réglages de l'utilisateur

- Émulateur **installé par l'utilisateur** : ses fichiers ne sont jamais modifiés. Les réglages passent par la ligne de commande et ne valent que pour la session (RetroArch `--appendconfig`, Dolphin `--config`). PCSX2 n'a pas cette possibilité : il garde alors ses propres réglages.
- Émulateur **géré par le lanceur** : le fichier INI est modifié clé par clé. Le lanceur retient ce qu'il a écrit. Si l'utilisateur change une valeur à la main, le lanceur la garde ensuite.
- `locked_keys` dans `config.toml` : liste de réglages que le lanceur ne touche jamais.
- Choisir un palier à la main (`clctl set-step` ou `Launcher.setQuality`) arrête la recherche pour ce jeu.

## Calibrer (à faire avec les premiers testeurs)

Les valeurs de l'estimation (`crates/cl-perf/src/estimate.rs`) et la référence du test CPU (`REFERENCE_NS` dans `crates/cl-hwprobe/src/bench.rs`) n'ont pas encore été mesurées sur de vraies machines. Protocole proposé :

1. Sur chaque machine de test : `clctl probe --json > machine.json`.
2. Pour 3 à 5 jeux par console : jouer avec la mesure activée jusqu'à ce que le réglage soit trouvé (`clctl plan` affiche « settled »).
3. Récupérer `hardware.json` et `profiles/*.json` (ils contiennent les statistiques de chaque session).
4. Comparer le palier estimé au palier final. Ajuster `required_cpu` et `gpu_budget` pour que l'estimation tombe juste dans la majorité des cas.
5. Consigner les machines, les jeux et les résultats dans un tableau partagé avant de modifier les seuils.

Mesurer une capture faite à la main :

```bash
mangohud --dlsym <émulateur> ...      # Linux, journal avec Maj+F2
clctl analyse capture.csv --game "nom du jeu"
```

## Points à vérifier sur les vraies versions des émulateurs

- Dolphin : valeurs de `ShaderCompilationMode` et nom de la clé `SkipPresentingDuplicateFrames`.
- PCSX2 : numéros de `Renderer` (14 = Vulkan, 3 = D3D11, 15 = D3D12, 12 = OpenGL) et lancement avec `-batch -nogui`.
- PresentMon 2.x : options de ligne de commande utilisées (`--process_name`, `--timed`, `--terminate_after_timed`, `--v1_metrics`).
- MangoHud : options `autostart_log` et `log_duration`.

## Limites connues

- Le premier lancement d'un jeu peut saccader pendant la compilation des shaders. Les 60 premières images sont ignorées et la mesure commence après 20 s, mais un jeu qui compile beaucoup peut être jugé trop sévèrement la première fois.
- La mesure ne voit que ce que l'écran affiche. Un ralentissement du son seul n'est pas détecté.
- Sous Windows, PresentMon demande des droits particuliers (voir [DEPENDENCIES.fr.md](DEPENDENCIES.fr.md)).
