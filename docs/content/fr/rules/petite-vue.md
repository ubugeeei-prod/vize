---
title: "Règles petite-vue"
description: "Directives et portées dans les pages HTML qui utilisent petite-vue."
---

# Règles petite-vue

Directives et portées dans les pages HTML qui utilisent petite-vue.

Chaque page de règle montre un exemple qui échoue, un exemple qui passe et la configuration qui active la règle.

## À activer (3)

Dans aucun préréglage. Activez-les par leur nom dans `vue.linter.rules`.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="petite-vue-no-unsupported-directive"></span>[`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | Interdire les directives non prises en charge par petite-vue. |  |
| <span id="petite-vue-valid-v-effect"></span>[`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | Exiger une expression non vide pour v-effect. |  |
| <span id="petite-vue-valid-v-scope"></span>[`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | Exiger que v-scope reçoive un objet littéral. |  |

[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](/rules/options.md) · [Migrer depuis ESLint](/rules/migration.md)
