---
title: "Regras HTML"
description: "Validade do HTML e marcação semântica nos templates."
---

# Regras HTML

Validade do HTML e marcação semântica nos templates.

Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.

## Essenciais (1)

Regras de correção da predefinição `essential`.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="html-id-duplication"></span>[`html/id-duplication`](./reference/html-id-duplication.md) | Proibir IDs de elementos duplicados. |  |

## Padrão (7)

Adicionadas por `happy-path`, a predefinição padrão.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="html-deprecated-attr"></span>[`html/deprecated-attr`](./reference/html-deprecated-attr.md) | Proibir atributos HTML obsoletos. |  |
| <span id="html-deprecated-element"></span>[`html/deprecated-element`](./reference/html-deprecated-element.md) | Proibir elementos HTML obsoletos. |  |
| <span id="html-no-consecutive-br"></span>[`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | Proibir elementos &lt;br&gt; consecutivos. |  |
| <span id="html-no-duplicate-class"></span>[`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | Proibir nomes de classe duplicados em um atributo class estático. |  |
| <span id="html-no-duplicate-dt"></span>[`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | Proibir nomes &lt;dt&gt; duplicados em &lt;dl&gt;. |  |
| <span id="html-no-empty-palpable-content"></span>[`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | Proibir elementos vazios que esperam conteúdo visível. |  |
| <span id="html-require-datetime"></span>[`html/require-datetime`](./reference/html-require-datetime.md) | Exigir o atributo datetime no elemento &lt;time&gt;. |  |

## Predefinições mais rígidas (1)

Só em `opinionated`, `nuxt` ou `ecosystem`. A coluna Predefinições mostra quais.

| Regra | O que verifica | Predefinições | Correção automática |
| --- | --- | --- | --- |
| <span id="html-no-dupe-style-properties"></span>[`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | Proibir propriedades duplicadas em atributos de estilo em linha. | `nuxt`, `opinionated` |  |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
