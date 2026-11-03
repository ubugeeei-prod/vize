---
title: "Règles de l’écosystème et de Nuxt"
description: "Vérifications pour les projets Nuxt, Vue Router, Pinia, Vue I18n, Vue Test Utils et Void Vue."
---

# Règles de l’écosystème et de Nuxt

Vérifications pour les projets Nuxt, Vue Router, Pinia, Vue I18n, Vue Test Utils et Void Vue.

Chaque page de règle montre un exemple qui échoue, un exemple qui passe et la configuration qui active la règle.

## Préréglages plus stricts (13)

Seulement dans `opinionated`, `nuxt` ou `ecosystem`. La colonne Préréglages indique lesquels.

| Règle | Ce qu’elle vérifie | Préréglages | Correction automatique |
| --- | --- | --- | --- |
| <span id="ecosystem-nuxt-prefer-nuxt-link"></span>[`ecosystem/nuxt-prefer-nuxt-link`](./reference/ecosystem-nuxt-prefer-nuxt-link.md) | Préférer NuxtLink pour les liens internes à l’application. | `nuxt` |  |
| <span id="ecosystem-pinia-prefer-store-to-refs"></span>[`ecosystem/pinia-prefer-store-to-refs`](./reference/ecosystem-pinia-prefer-store-to-refs.md) | Préférer storeToRefs() lors de la déstructuration des stores Pinia. | `ecosystem` |  |
| <span id="ecosystem-router-link-require-to"></span>[`ecosystem/router-link-require-to`](./reference/ecosystem-router-link-require-to.md) | Exiger une destination `to` sur les composants RouterLink et NuxtLink. | `ecosystem` |  |
| <span id="ecosystem-void-link-require-href"></span>[`ecosystem/void-link-require-href`](./reference/ecosystem-void-link-require-href.md) | Exiger `href` sur les composants Link de Void Vue. | `ecosystem` |  |
| <span id="ecosystem-void-link-valid-method"></span>[`ecosystem/void-link-valid-method`](./reference/ecosystem-void-link-valid-method.md) | Valider les props method statiques des composants Link de Void Vue. | `ecosystem` |  |
| <span id="ecosystem-vue-i18n-no-missing-key"></span>[`ecosystem/vue-i18n-no-missing-key`](./reference/ecosystem-vue-i18n-no-missing-key.md) | Signaler les clés vue-i18n statiques absentes des messages locaux du SFC. | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-link"></span>[`ecosystem/vue-router-prefer-named-link`](./reference/ecosystem-vue-router-prefer-named-link.md) | Préférer les objets de routes nommées aux chaînes de chemin statiques dans RouterLink. | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-push"></span>[`ecosystem/vue-router-prefer-named-push`](./reference/ecosystem-vue-router-prefer-named-push.md) | Préférer les objets de routes nommées pour la navigation programmatique de Vue Router. | `ecosystem` |  |
| <span id="ecosystem-vue-test-utils-no-html-snapshot"></span>[`ecosystem/vue-test-utils-no-html-snapshot`](./reference/ecosystem-vue-test-utils-no-html-snapshot.md) | Éviter les instantanés de wrapper.html() dans les tests Vue Test Utils. | `ecosystem` |  |
| <span id="nuxt-no-nuxt-config-test-key"></span>[`nuxt/no-nuxt-config-test-key`](./reference/nuxt-no-nuxt-config-test-key.md) | Interdire la clé `test` dans la configuration Nuxt. | `nuxt` |  |
| <span id="nuxt-no-page-meta-runtime-values"></span>[`nuxt/no-page-meta-runtime-values`](./reference/nuxt-no-page-meta-runtime-values.md) | Interdire les valeurs du contexte d’exécution évaluées immédiatement dans `definePageMeta`, extrait dans un chunk séparé à la compilation et exécuté avant le setup du composant. | `nuxt` |  |
| <span id="nuxt-nuxt-config-keys-order"></span>[`nuxt/nuxt-config-keys-order`](./reference/nuxt-nuxt-config-keys-order.md) | Préférer l’ordre recommandé des propriétés de configuration Nuxt. | `nuxt` | ✓ |
| <span id="nuxt-prefer-import-meta"></span>[`nuxt/prefer-import-meta`](./reference/nuxt-prefer-import-meta.md) | Préférer `import.meta.*` à `process.*`. | `nuxt` | ✓ |

[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](/rules/options.md) · [Migrer depuis ESLint](/rules/migration.md)
