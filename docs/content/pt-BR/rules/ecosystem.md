---
title: "Regras do ecossistema e Nuxt"
description: "Verificações para projetos com Nuxt, Vue Router, Pinia, Vue I18n, Vue Test Utils e Void Vue."
---

# Regras do ecossistema e Nuxt

Verificações para projetos com Nuxt, Vue Router, Pinia, Vue I18n, Vue Test Utils e Void Vue.

Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.

## Predefinições mais rígidas (13)

Só em `opinionated`, `nuxt` ou `ecosystem`. A coluna Predefinições mostra quais.

| Regra | O que verifica | Predefinições | Correção automática |
| --- | --- | --- | --- |
| <span id="ecosystem-nuxt-prefer-nuxt-link"></span>[`ecosystem/nuxt-prefer-nuxt-link`](./reference/ecosystem-nuxt-prefer-nuxt-link.md) | Preferir NuxtLink para links internos da aplicação. | `nuxt` |  |
| <span id="ecosystem-pinia-prefer-store-to-refs"></span>[`ecosystem/pinia-prefer-store-to-refs`](./reference/ecosystem-pinia-prefer-store-to-refs.md) | Preferir storeToRefs() ao desestruturar stores do Pinia. | `ecosystem` |  |
| <span id="ecosystem-router-link-require-to"></span>[`ecosystem/router-link-require-to`](./reference/ecosystem-router-link-require-to.md) | Exigir um destino `to` nos componentes RouterLink e NuxtLink. | `ecosystem` |  |
| <span id="ecosystem-void-link-require-href"></span>[`ecosystem/void-link-require-href`](./reference/ecosystem-void-link-require-href.md) | Exigir `href` nos componentes Link do Void Vue. | `ecosystem` |  |
| <span id="ecosystem-void-link-valid-method"></span>[`ecosystem/void-link-valid-method`](./reference/ecosystem-void-link-valid-method.md) | Validar props method estáticas do Link do Void Vue. | `ecosystem` |  |
| <span id="ecosystem-vue-i18n-no-missing-key"></span>[`ecosystem/vue-i18n-no-missing-key`](./reference/ecosystem-vue-i18n-no-missing-key.md) | Relatar chaves estáticas do vue-i18n ausentes nas mensagens locais do SFC. | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-link"></span>[`ecosystem/vue-router-prefer-named-link`](./reference/ecosystem-vue-router-prefer-named-link.md) | Preferir objetos de rotas nomeadas a strings de caminho estáticas no RouterLink. | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-push"></span>[`ecosystem/vue-router-prefer-named-push`](./reference/ecosystem-vue-router-prefer-named-push.md) | Preferir objetos de rotas nomeadas para navegação programática com Vue Router. | `ecosystem` |  |
| <span id="ecosystem-vue-test-utils-no-html-snapshot"></span>[`ecosystem/vue-test-utils-no-html-snapshot`](./reference/ecosystem-vue-test-utils-no-html-snapshot.md) | Evitar snapshots de wrapper.html() nos testes com Vue Test Utils. | `ecosystem` |  |
| <span id="nuxt-no-nuxt-config-test-key"></span>[`nuxt/no-nuxt-config-test-key`](./reference/nuxt-no-nuxt-config-test-key.md) | Proibir a definição da chave `test` na configuração do Nuxt. | `nuxt` |  |
| <span id="nuxt-no-page-meta-runtime-values"></span>[`nuxt/no-page-meta-runtime-values`](./reference/nuxt-no-page-meta-runtime-values.md) | Proibir valores do contexto de execução no nível de avaliação imediata de `definePageMeta`, que é extraído para um fragmento separado durante a compilação e executado antes do setup do componente. | `nuxt` |  |
| <span id="nuxt-nuxt-config-keys-order"></span>[`nuxt/nuxt-config-keys-order`](./reference/nuxt-nuxt-config-keys-order.md) | Preferir a ordem recomendada das propriedades de configuração do Nuxt. | `nuxt` | ✓ |
| <span id="nuxt-prefer-import-meta"></span>[`nuxt/prefer-import-meta`](./reference/nuxt-prefer-import-meta.md) | Preferir `import.meta.*` a `process.*`. | `nuxt` | ✓ |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
