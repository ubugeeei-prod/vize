---
title: "Regras de Vapor"
description: "Restrições de template e script para componentes compilados no modo Vapor."
---

# Regras de Vapor

Restrições de template e script para componentes compilados no modo Vapor.

Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.

## Padrão (1)

Adicionadas por `happy-path`, a predefinição padrão.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="vapor-no-vue-lifecycle-events"></span>[`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | Proibir eventos de ciclo de vida @vue:xxx por elemento (não suportados em Vapor). |  |

## Predefinições mais rígidas (5)

Só em `opinionated`, `nuxt` ou `ecosystem`. A coluna Predefinições mostra quais.

| Regra | O que verifica | Predefinições | Correção automática |
| --- | --- | --- | --- |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Proibir getCurrentInstance() no modo Vapor (retorna null). | `opinionated` |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Proibir padrões da Options API no modo Vapor. | `opinionated` |  |
| <span id="vapor-no-inline-template"></span>[`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | Proibir o atributo obsoleto inline-template. | `nuxt`, `opinionated` |  |
| <span id="vapor-prefer-static-class"></span>[`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | Preferir class estática a uma vinculação dinâmica de class para literais de string. | `nuxt`, `opinionated` | ✓ |
| <span id="vapor-require-vapor-attribute"></span>[`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | Sugerir a adição do atributo vapor a script setup. | `nuxt`, `opinionated` |  |

## Opcionais (1)

Não fazem parte de nenhuma predefinição. Ative pelo nome em `linter.rules`, passado a `withVue`.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Proibir o uso de nextTick() em componentes orientados a Vapor. |  |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
