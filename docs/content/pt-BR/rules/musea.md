---
title: "Regras Musea"
description: "Histórias `.art.vue`, variantes e uso de tokens de design do Musea."
---

# Regras Musea

Histórias `.art.vue`, variantes e uso de tokens de design do Musea.

Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.

## Opcionais (6)

Não fazem parte de nenhuma predefinição. Ative pelo nome em `linter.rules`, passado a `withVue`.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | Proibir blocos &lt;variant&gt; vazios. |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | Preferir variáveis CSS de tokens de design a valores primitivos fixos escritos diretamente. |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | Exigir o atributo component no bloco &lt;art&gt;. |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | Exigir o atributo title no bloco &lt;art&gt;. |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | Exigir nomes de variante únicos. |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | Exigir o atributo name nos blocos &lt;variant&gt;. |  |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
