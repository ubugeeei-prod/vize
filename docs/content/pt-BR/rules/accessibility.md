---
title: "Regras de acessibilidade"
description: "ARIA, interação por teclado, rótulos, landmarks e alternativas de mídia, para que os templates funcionem com tecnologias assistivas."
---

# Regras de acessibilidade

ARIA, interação por teclado, rótulos, landmarks e alternativas de mídia, para que os templates funcionem com tecnologias assistivas.

Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.

## Padrão (25)

Adicionadas por `happy-path`, a predefinição padrão.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="a11y-alt-text"></span>[`a11y/alt-text`](./reference/a11y-alt-text.md) | Exigir texto alternativo para elementos de mídia. |  |
| <span id="a11y-anchor-has-content"></span>[`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | Exigir conteúdo acessível nos elementos de âncora. |  |
| <span id="a11y-anchor-is-valid"></span>[`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | Exigir um href válido nos elementos de âncora. |  |
| <span id="a11y-aria-props"></span>[`a11y/aria-props`](./reference/a11y-aria-props.md) | Proibir atributos ARIA inválidos. |  |
| <span id="a11y-aria-role"></span>[`a11y/aria-role`](./reference/a11y-aria-role.md) | Exigir que elementos com papéis ARIA usem um papel ARIA válido e não abstrato. |  |
| <span id="a11y-aria-unsupported-elements"></span>[`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | Proibir atributos ARIA em elementos que não os aceitam. |  |
| <span id="a11y-click-events-have-key-events"></span>[`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | Exigir manipuladores de eventos de teclado junto aos eventos de clique. |  |
| <span id="a11y-form-control-has-label"></span>[`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | Exigir rótulos associados aos controles de formulário. |  |
| <span id="a11y-heading-has-content"></span>[`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | Exigir conteúdo acessível nos elementos de título. |  |
| <span id="a11y-iframe-has-title"></span>[`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | Exigir um atributo title nos elementos iframe. |  |
| <span id="a11y-interactive-supports-focus"></span>[`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | Exigir que elementos com papéis interativos possam receber foco. |  |
| <span id="a11y-label-has-for"></span>[`a11y/label-has-for`](./reference/a11y-label-has-for.md) | Exigir controles de formulário associados aos rótulos. |  |
| <span id="a11y-media-has-caption"></span>[`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | Exigir legendas nos elementos de mídia. |  |
| <span id="a11y-mouse-events-have-key-events"></span>[`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | Exigir eventos de foco e perda de foco junto aos eventos de mouse. |  |
| <span id="a11y-no-access-key"></span>[`a11y/no-access-key`](./reference/a11y-no-access-key.md) | Proibir o uso do atributo accesskey. |  |
| <span id="a11y-no-aria-hidden-on-focusable"></span>[`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | Proibir aria-hidden="true" em elementos que podem receber foco. |  |
| <span id="a11y-no-autofocus"></span>[`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | Proibir o uso do atributo autofocus. |  |
| <span id="a11y-no-distracting-elements"></span>[`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | Proibir elementos que causam distração, como &lt;marquee&gt; e &lt;blink&gt;. |  |
| <span id="a11y-no-i-for-icon"></span>[`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | Proibir o uso do elemento &lt;i&gt; para ícones. |  |
| <span id="a11y-no-redundant-roles"></span>[`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | Proibir papéis ARIA redundantes. | ✓ |
| <span id="a11y-no-refer-to-non-existent-id"></span>[`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | Proibir referências a IDs inexistentes. |  |
| <span id="a11y-no-role-presentation-on-focusable"></span>[`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | Proibir role="presentation" ou role="none" em elementos que podem receber foco. |  |
| <span id="a11y-no-static-element-interactions"></span>[`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | Proibir manipuladores de eventos em elementos estáticos. |  |
| <span id="a11y-role-has-required-aria-props"></span>[`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | Exigir as propriedades obrigatórias dos papéis ARIA. |  |
| <span id="a11y-tabindex-no-positive"></span>[`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | Proibir valores positivos de tabindex. |  |

## Predefinições mais rígidas (5)

Só em `opinionated`, `nuxt` ou `ecosystem`. A coluna Predefinições mostra quais.

| Regra | O que verifica | Predefinições | Correção automática |
| --- | --- | --- | --- |
| <span id="a11y-heading-levels"></span>[`a11y/heading-levels`](./reference/a11y-heading-levels.md) | Proibir que níveis de título sejam pulados. | `nuxt`, `opinionated` |  |
| <span id="a11y-landmark-roles"></span>[`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | Validar a posição e a unicidade dos papéis de regiões de referência. | `nuxt`, `opinionated` |  |
| <span id="a11y-placeholder-label-option"></span>[`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | Exigir uma opção de orientação em um select required. | `nuxt`, `opinionated` |  |
| <span id="a11y-use-list"></span>[`a11y/use-list`](./reference/a11y-use-list.md) | Sugerir elementos de lista para textos que parecem itens de lista. | `nuxt`, `opinionated` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos. | `nuxt`, `opinionated` |  |

## Opcionais (1)

Não fazem parte de nenhuma predefinição. Ative pelo nome em `linter.rules`, passado a `withVue`.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="a11y-img-alt"></span>[`a11y/img-alt`](./reference/a11y-img-alt.md) | Exigir o atributo alt nas imagens para garantir acessibilidade. |  |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
