---
title: "Regras de Musea e CSS"
description: "Histórias `.art.vue` do Musea e CSS puro em blocos `<style>`."
---

# Regras de Musea e CSS

Histórias `.art.vue` do Musea e CSS puro em blocos `<style>`.

Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.

## Predefinições mais rígidas (10)

Só em `opinionated`, `nuxt` ou `ecosystem`. A coluna Predefinições mostra quais.

| Regra | O que verifica | Predefinições | Correção automática |
| --- | --- | --- | --- |
| <span id="css-no-display-none"></span>[`css/no-display-none`](./reference/css-no-display-none.md) | Sugerir v-show em vez de display: none. | `opinionated`, `nuxt` |  |
| <span id="css-no-hardcoded-values"></span>[`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | Sugerir variáveis CSS em vez de valores fixos escritos diretamente. | `opinionated`, `nuxt` |  |
| <span id="css-no-id-selectors"></span>[`css/no-id-selectors`](./reference/css-no-id-selectors.md) | Desencorajar o uso de seletores de ID no CSS. | `opinionated`, `nuxt` |  |
| <span id="css-no-important"></span>[`css/no-important`](./reference/css-no-important.md) | Desencorajar o uso de !important no CSS. | `opinionated`, `nuxt` |  |
| <span id="css-no-utility-classes"></span>[`css/no-utility-classes`](./reference/css-no-utility-classes.md) | Alertar sobre a implementação de classes utilitárias nos estilos de componentes. | `opinionated`, `nuxt` |  |
| <span id="css-no-v-bind-performance"></span>[`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | Alertar sobre o custo de desempenho do v-bind() no CSS. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-logical-properties"></span>[`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | Recomendar propriedades lógicas de CSS para melhorar o suporte à internacionalização. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-nested-selectors"></span>[`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | Recomendar o aninhamento de CSS para seletores de descendentes. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-slotted"></span>[`css/prefer-slotted`](./reference/css-prefer-slotted.md) | Recomendar ::v-slotted() para estilizar o conteúdo de slots. | `opinionated`, `nuxt` |  |
| <span id="css-require-font-display"></span>[`css/require-font-display`](./reference/css-require-font-display.md) | Exigir font-display nas regras @font-face. | `opinionated`, `nuxt` |  |

## Opcionais (6)

Não fazem parte de nenhuma predefinição. Ative pelo nome em `vue.linter.rules`.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | Proibir blocos &lt;variant&gt; vazios. |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | Preferir variáveis CSS de tokens de design a valores primitivos fixos escritos diretamente. |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | Exigir o atributo component no bloco &lt;art&gt;. |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | Exigir o atributo title no bloco &lt;art&gt;. |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | Exigir nomes de variante únicos. |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | Exigir o atributo name nos blocos &lt;variant&gt;. |  |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
