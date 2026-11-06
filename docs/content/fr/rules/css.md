---
title: "Règles CSS"
description: "Le CSS standard des blocs `<style>`, avec des vérifications de justesse et de maintenabilité."
---

# Règles CSS

Le CSS standard des blocs `<style>`, avec des vérifications de justesse et de maintenabilité.

Chaque page de règle montre un exemple qui échoue, un exemple qui passe et la configuration qui active la règle.

## Préréglages plus stricts (10)

Seulement dans `opinionated`, `nuxt` ou `ecosystem`. La colonne Préréglages indique lesquels.

| Règle | Ce qu’elle vérifie | Préréglages | Correction automatique |
| --- | --- | --- | --- |
| <span id="css-no-display-none"></span>[`css/no-display-none`](./reference/css-no-display-none.md) | Suggérer v-show plutôt que display: none. | `opinionated`, `nuxt` |  |
| <span id="css-no-hardcoded-values"></span>[`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | Suggérer des variables CSS plutôt que des valeurs codées en dur. | `opinionated`, `nuxt` |  |
| <span id="css-no-id-selectors"></span>[`css/no-id-selectors`](./reference/css-no-id-selectors.md) | Déconseiller les sélecteurs d’identifiant en CSS. | `opinionated`, `nuxt` |  |
| <span id="css-no-important"></span>[`css/no-important`](./reference/css-no-important.md) | Déconseiller !important en CSS. | `opinionated`, `nuxt` |  |
| <span id="css-no-utility-classes"></span>[`css/no-utility-classes`](./reference/css-no-utility-classes.md) | Déconseiller l’implémentation de classes utilitaires dans les styles des composants. | `opinionated`, `nuxt` |  |
| <span id="css-no-v-bind-performance"></span>[`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | Signaler le coût de performance de v-bind() en CSS. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-logical-properties"></span>[`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | Recommander les propriétés logiques CSS pour mieux prendre en charge l’internationalisation. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-nested-selectors"></span>[`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | Recommander l’imbrication CSS pour les sélecteurs de descendants. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-slotted"></span>[`css/prefer-slotted`](./reference/css-prefer-slotted.md) | Recommander ::v-slotted() pour styliser le contenu des slots. | `opinionated`, `nuxt` |  |
| <span id="css-require-font-display"></span>[`css/require-font-display`](./reference/css-require-font-display.md) | Exiger font-display dans les règles @font-face. | `opinionated`, `nuxt` |  |

[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](/rules/options.md) · [Migrer depuis ESLint](/rules/migration.md)
