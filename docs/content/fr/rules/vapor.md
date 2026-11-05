---
title: "Règles Vapor"
description: "Contraintes de template et de script pour les composants compilés en mode Vapor."
---

# Règles Vapor

Contraintes de template et de script pour les composants compilés en mode Vapor.

Chaque page de règle montre un exemple qui échoue, un exemple qui passe et la configuration qui active la règle.

## Par défaut (1)

Ajoutées par `happy-path`, le préréglage par défaut.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="vapor-no-vue-lifecycle-events"></span>[`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor). |  |

## Préréglages plus stricts (5)

Seulement dans `opinionated`, `nuxt` ou `ecosystem`. La colonne Préréglages indique lesquels.

| Règle | Ce qu’elle vérifie | Préréglages | Correction automatique |
| --- | --- | --- | --- |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Interdire getCurrentInstance() en mode Vapor (renvoie null). | `opinionated` |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Interdire les formes de l’Options API en mode Vapor. | `opinionated` |  |
| <span id="vapor-no-inline-template"></span>[`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | Interdire l’attribut obsolète inline-template. | `nuxt`, `opinionated` |  |
| <span id="vapor-prefer-static-class"></span>[`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales. | `nuxt`, `opinionated` | ✓ |
| <span id="vapor-require-vapor-attribute"></span>[`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | Suggérer l’ajout de l’attribut vapor à script setup. | `nuxt`, `opinionated` |  |

## À activer (1)

Dans aucun préréglage. Activez-les par leur nom dans `vue.linter.rules`.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Interdire l’utilisation de nextTick() dans les composants destinés à Vapor. |  |

[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](/rules/options.md) · [Migrer depuis ESLint](/rules/migration.md)
