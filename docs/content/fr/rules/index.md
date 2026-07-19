---
title: Règles
---

<!-- Generated translation; source: rules/index.md -->

# Règles

Les diagnostics Vize sont documentés sous forme de règles, et non comme une seule grande matrice. Chaque page de règle garde le comportement de détection de
proche des exemples Mauvais/Bons afin que la référence puisse être lue comme un manuel de règles ESLint
.

## Pages

- [All Patina rules](./all.md): table de métadonnées d’une page pour chaque implémentation de la règle Patina,
  incluant des liens source GitHub.
- [Vue rules](./vue.md): structure de modèles SFC, directives Vue, conventions de composants, et
  des vérifications de correction Vue en file indienne.
- [Type and script rules](./type-and-script.md): Diagnostics et Vapor par vérificateur TypeScript
  restrictions de script.
- [HTML rules](./html.md): vérifications de validité HTML et de balisage sémantique.
- [Accessibility rules](./accessibility.md): ARIA, interaction clavier, étiquettes, repères, et
  des contrôles médias accessibles.
- [SSR rules](./ssr.md): Risques de rendu des serveurs et d’hydratation.
- [Vapor rules](./vapor.md): Contraintes de gabarit uniquement pour la vapeur.
- [Ecosystem rules](./ecosystem.md): vérifications prédéfinies pour Nuxt, Vue Router, Pinia, vue-i18n,
  Vue Test Utils et Void Vue.
- [Musea and CSS rules](./musea-and-css.md): Contrôles de blocs d’art Musea et diagnostics de style.
- [Cross-file rules](./cross-file.md): diagnostic du graphe de projet émis par
  `vize lint --cross-file`.

## Presets

`essential` contient des règles de correction qui devraient presque toujours être activées. `happy-path` ajoute
contrôles pratiques d’hygiène pour le développement quotidien de Vue. `ecosystem` part du large ensemble par défaut
et ajoute Vue Router, Vue I18n, Pinia, Vue Test Utils, Nuxt et Void Vue Checks. `nuxt`
inclut les attentes SSR orientées Nuxt et les attentes Vapor. `opinionated` est le préréglage intégré
le plus large.

`incremental` commence vide. Utilisez-le lorsqu’un hôte souhaite choisir des règles spécifiques sans hériter d’un préréglage
plus grand.

## Configuration sensible aux types

Les règles nécessitant des informations sémantiques lisent le projet TypeScript à travers `tsconfig.json`. Je préfère
mettre des noms d’environnement partagés dans `compilerOptions.types` ou références de projet plutôt que de garder
une liste de `globals` séparée dans la configuration Vize.
