# Roadmap (prudente)

Cette roadmap reprend les limites déjà documentées dans le dépôt. Elle décrit des priorités de stabilisation, pas des engagements fermes.

## Priorités avant une version stable

1. **Validation Windows réelle**
   - Tester les paquets sur des machines Windows physiques (pas uniquement CI/Wine).
   - Confirmer les parcours complets (lancement, retour menu, réglages, logs).

2. **Validation avec vrais émulateurs/jeux/manettes**
   - Vérifier les profils d'émulateurs en conditions réelles.
   - Confirmer la navigation manette dans tous les écrans clés.

3. **Mesure de fluidité et calibrage**
   - Vérifier PresentMon/MangoHud en session de jeu réelle.
   - Ajuster les seuils d'optimisation selon les résultats terrain.

4. **Stabilisation de la branche principale**
   - Continuer la consolidation des changements en cours.
   - Réduire les écarts entre prototypes et documentation publique.

5. **Licence du projet**
   - Choisir et publier une licence officielle.
   - Vérifier la cohérence documentaire une fois la licence fixée.

6. **Documentation utilisateur**
   - Compléter les guides d'installation et de dépannage avec retours vérifiés.
   - Clarifier les parcours supportés selon la plateforme.

7. **Préparer une première release stable**
   - Définir des critères de sortie explicites.
   - Publier une version non expérimentale uniquement après validation complète.

## Références

- Journal technique : [docs/development-log.md](development-log.md)
- Prototype Qt/Rust : [docs/prototype/README.fr.md](prototype/README.fr.md)
- Comparaison d'approches : [docs/comparison/COMPARAISON.fr.md](comparison/COMPARAISON.fr.md)
