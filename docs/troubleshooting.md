# Dépannage

Ce document couvre les problèmes déjà identifiés dans la documentation du dépôt.

## 1) Un émulateur n'apparaît pas

Symptômes : console indisponible, jeu non lançable, ou page signalant un émulateur manquant.

À vérifier :

- L'émulateur est bien installé localement.
- Le chemin est bien ajouté dans les réglages (dossiers / exécutable).
- La console utilise le bon émulateur dans **Réglages > Consoles et émulateurs**.

Référence : [docs/prototype/EMULATORS.fr.md](prototype/EMULATORS.fr.md).

## 2) La mesure de fluidité est indisponible

### Windows (PresentMon)

- Ouvrir **Réglages > Mesure de fluidité**.
- Cliquer **Autoriser la mesure**.
- Accepter la demande administrateur.
- Fermer puis rouvrir la session Windows.

Sans cette autorisation, les jeux se lancent quand même avec les réglages estimés.

### Linux (MangoHud)

- Vérifier l'usage du paquet AppImage officiel du dépôt.
- Vérifier qu'une version MangoHud locale n'entre pas en conflit (la copie installée sur la machine est prioritaire).

Références :
- [qt-prototype/README.fr.md](../qt-prototype/README.fr.md)
- [docs/prototype/DEPENDENCIES.fr.md](prototype/DEPENDENCIES.fr.md)

## 3) Démarrage sans fenêtre / erreur QML

Un problème historique documenté était : `SettingsPanel is not a type`.

Actions recommandées :

- Réessayer avec un paquet récent.
- Lancer un test de démarrage :

```bash
console-launcher-qt --smoke-test
```

- Consulter les logs (`console-launcher.log`, `console-launcher-engine.log`).

Référence historique : [docs/development-log.md](development-log.md).

## 4) Problème de permissions Windows

Si la mesure PresentMon échoue, la permission Windows n'est probablement pas active.

- Refaire **Réglages > Mesure de fluidité > Autoriser la mesure**.
- Vérifier que la demande administrateur a bien été validée.
- Redémarrer la session Windows.

## 5) La manette ne répond pas

- Vérifier que la manette est reconnue par le système.
- Tester aussi la navigation clavier (flèches/Entrée/Échap) pour isoler le problème.
- Les manettes supportées passent par SDL2 GameController (Xbox, PlayStation, Switch Pro, génériques).

Référence : [qt-prototype/README.fr.md](../qt-prototype/README.fr.md).

## 6) Où trouver les logs

Les journaux utiles sont :

- `console-launcher.log` (interface)
- `console-launcher-engine.log` (moteur)

Ils sont écrits à côté de l'exécutable, ou dans un dossier temporaire si ce dossier est en lecture seule.
