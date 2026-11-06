---
title: "Regras de petite-vue"
description: "Diretivas e escopos em páginas HTML que usam petite-vue."
---

# Regras de petite-vue

Diretivas e escopos em páginas HTML que usam petite-vue.

Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.

## Opcionais (3)

Não fazem parte de nenhuma predefinição. Ative pelo nome em `linter.rules`, passado a `withVue`.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="petite-vue-no-unsupported-directive"></span>[`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | Proibir diretivas que o petite-vue não aceita. |  |
| <span id="petite-vue-valid-v-effect"></span>[`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | Exigir uma expressão não vazia em v-effect. |  |
| <span id="petite-vue-valid-v-scope"></span>[`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | Exigir que v-scope vincule um objeto literal. |  |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
