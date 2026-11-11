---
title: "Regras Vue"
description: "Verificações de correção, convenções e migração para o Vue 3 em templates, diretivas, componentes e blocos de SFC."
---

# Regras Vue

Verificações de correção, convenções e migração para o Vue 3 em templates, diretivas, componentes e blocos de SFC.

Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.

## Essenciais (53)

Regras de correção da predefinição `essential`.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="vue-comment-directive"></span>[`vue/comment-directive`](./reference/vue-comment-directive.md) | Aceita comentários de controle de diagnósticos em `<template>`. |  |
| <span id="vue-jsx-uses-vars"></span>[`vue/jsx-uses-vars`](./reference/vue-jsx-uses-vars.md) | Considera usadas as variáveis referenciadas nos nomes dos elementos JSX. |  |
| <span id="vue-multi-word-component-names"></span>[`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | Exigir nomes de componentes com mais de uma palavra. |  |
| <span id="vue-no-child-content"></span>[`vue/no-child-content`](./reference/vue-no-child-content.md) | Proibir conteúdo filho ao usar v-html ou v-text. |  |
| <span id="vue-no-deprecated-filter"></span>[`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | Proibir a sintaxe obsoleta de filtros do Vue 2 com o operador de barra vertical. |  |
| <span id="vue-no-deprecated-functional-template"></span>[`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | Proibir o atributo `functional` no `<template>` de um SFC. |  |
| <span id="vue-no-deprecated-html-element-is"></span>[`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | Proibir o atributo `is` em elementos HTML nativos. |  |
| <span id="vue-no-deprecated-inline-template"></span>[`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | Proibir o atributo obsoleto `inline-template`. |  |
| <span id="vue-no-deprecated-router-link-tag-prop"></span>[`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | Proibir a prop `tag` em &lt;router-link&gt;. |  |
| <span id="vue-no-deprecated-scope-attribute"></span>[`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | Proibir o atributo obsoleto `scope` em &lt;template&gt;. |  |
| <span id="vue-no-deprecated-slot-attribute"></span>[`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | Proibir o atributo obsoleto `slot`. |  |
| <span id="vue-no-deprecated-slot-scope-attribute"></span>[`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | Proibir o atributo obsoleto `slot-scope`. |  |
| <span id="vue-no-deprecated-v-bind-sync"></span>[`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | Proibir o modificador obsoleto `.sync` em `v-bind`. |  |
| <span id="vue-no-deprecated-v-is"></span>[`vue/no-deprecated-v-is`](./reference/vue-no-deprecated-v-is.md) | Proibir a diretiva obsoleta `v-is`. |  |
| <span id="vue-no-deprecated-v-on-native-modifier"></span>[`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | Proibir o modificador obsoleto `.native` em `v-on`. |  |
| <span id="vue-no-deprecated-v-on-number-modifiers"></span>[`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | Proibir modificadores numéricos obsoletos de `keyCode` em `v-on`. | ✓ |
| <span id="vue-no-dupe-v-else-if"></span>[`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | Proibir condições duplicadas em cadeias de `v-if` / `v-else-if`. |  |
| <span id="vue-no-duplicate-attributes"></span>[`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | Proibir atributos duplicados no mesmo elemento. |  |
| <span id="vue-no-mutating-props"></span>[`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | Proibir a mutação de props de componentes. |  |
| <span id="vue-no-non-component-keep-alive-child"></span>[`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | Proibir invólucros de elementos comuns diretamente abaixo de `<KeepAlive>`. |  |
| <span id="vue-no-reserved-component-names"></span>[`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | Proibir o uso de nomes reservados como nomes de componentes. |  |
| <span id="vue-no-template-key"></span>[`vue/no-template-key`](./reference/vue-no-template-key.md) | Proibir o atributo `key` em `<template>`. |  |
| <span id="vue-no-textarea-mustache"></span>[`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | Proibir interpolação com chaves duplas em `<textarea>`. |  |
| <span id="vue-no-unsafe-url"></span>[`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | Alertar sobre vinculações de URL potencialmente inseguras. |  |
| <span id="vue-no-unused-components"></span>[`vue/no-unused-components`](./reference/vue-no-unused-components.md) | Proibir o registro de componentes não usados nos templates. |  |
| <span id="vue-no-unused-vars"></span>[`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | Proibir definições de variáveis não usadas nas diretivas v-for e v-slot. |  |
| <span id="vue-no-use-v-if-with-v-for"></span>[`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | Proibir `v-if` no mesmo elemento que `v-for`. |  |
| <span id="vue-no-useless-template-attributes"></span>[`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | Proibir atributos sem efeito em elementos `<template>`. |  |
| <span id="vue-no-v-for-template-key-on-child"></span>[`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | Proibir `key` no filho de um `<template v-for>`. |  |
| <span id="vue-no-v-html"></span>[`vue/no-v-html`](./reference/vue-no-v-html.md) | Alertar sobre v-html para prevenir vulnerabilidades XSS. |  |
| <span id="vue-no-v-text-v-html-on-component"></span>[`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | Proibir v-text / v-html em elementos de componente. |  |
| <span id="vue-require-component-is"></span>[`vue/require-component-is`](./reference/vue-require-component-is.md) | Exigir `v-bind:is` em elementos `<component>`. |  |
| <span id="vue-require-toggle-inside-transition"></span>[`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | Exigir uma alternância no elemento envolvido por `<transition>`. |  |
| <span id="vue-require-v-for-key"></span>[`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | Exigir `v-bind:key` nas diretivas `v-for`. |  |
| <span id="vue-use-v-on-exact"></span>[`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | Exigir o modificador `.exact` em `v-on` quando houver manipuladores baseados em modificadores. |  |
| <span id="vue-valid-attribute-name"></span>[`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | Exigir nomes de atributos válidos. |  |
| <span id="vue-valid-template-root"></span>[`vue/valid-template-root`](./reference/vue-valid-template-root.md) | Exigir uma raiz `<template>` válida para a semântica de fragmentos do Vue 3. |  |
| <span id="vue-valid-v-bind"></span>[`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | Exigir diretivas `v-bind` válidas. |  |
| <span id="vue-valid-v-cloak"></span>[`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | Exigir diretivas `v-cloak` válidas. |  |
| <span id="vue-valid-v-else"></span>[`vue/valid-v-else`](./reference/vue-valid-v-else.md) | Exigir diretivas `v-else` válidas. | ✓ |
| <span id="vue-valid-v-else-if"></span>[`vue/valid-v-else-if`](./reference/vue-valid-v-else-if.md) | Exigir diretivas `v-else-if` válidas. |  |
| <span id="vue-valid-v-for"></span>[`vue/valid-v-for`](./reference/vue-valid-v-for.md) | Exigir diretivas `v-for` válidas. |  |
| <span id="vue-valid-v-html"></span>[`vue/valid-v-html`](./reference/vue-valid-v-html.md) | Exigir diretivas `v-html` válidas. |  |
| <span id="vue-valid-v-if"></span>[`vue/valid-v-if`](./reference/vue-valid-v-if.md) | Exigir diretivas `v-if` válidas. |  |
| <span id="vue-valid-v-is"></span>[`vue/valid-v-is`](./reference/vue-valid-v-is.md) | Exigir diretivas `v-is` válidas. |  |
| <span id="vue-valid-v-memo"></span>[`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | Exigir diretivas `v-memo` válidas. |  |
| <span id="vue-valid-v-model"></span>[`vue/valid-v-model`](./reference/vue-valid-v-model.md) | Exigir diretivas `v-model` válidas. |  |
| <span id="vue-valid-v-on"></span>[`vue/valid-v-on`](./reference/vue-valid-v-on.md) | Exigir diretivas `v-on` válidas. |  |
| <span id="vue-valid-v-once"></span>[`vue/valid-v-once`](./reference/vue-valid-v-once.md) | Exigir diretivas `v-once` válidas. |  |
| <span id="vue-valid-v-pre"></span>[`vue/valid-v-pre`](./reference/vue-valid-v-pre.md) | Exigir diretivas `v-pre` válidas. |  |
| <span id="vue-valid-v-show"></span>[`vue/valid-v-show`](./reference/vue-valid-v-show.md) | Exigir diretivas `v-show` válidas. |  |
| <span id="vue-valid-v-slot"></span>[`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | Exigir diretivas `v-slot` válidas. |  |
| <span id="vue-valid-v-text"></span>[`vue/valid-v-text`](./reference/vue-valid-v-text.md) | Exigir diretivas `v-text` válidas. |  |

## Padrão (27)

Adicionadas por `happy-path`, a predefinição padrão.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="vue-attribute-hyphenation"></span>[`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | Aplicar um padrão de nomes de atributos em componentes personalizados. | ✓ |
| <span id="vue-attribute-order"></span>[`vue/attribute-order`](./reference/vue-attribute-order.md) | Aplicar uma ordem consistente aos atributos. |  |
| <span id="vue-component-definition-name-casing"></span>[`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | Exigir PascalCase ou kebab-case nos nomes de definição de componentes. |  |
| <span id="vue-html-end-tags"></span>[`vue/html-end-tags`](./reference/vue-html-end-tags.md) | Exige tags de fechamento explícitas para elementos HTML cujo fechamento pode ser omitido. | ✓ |
| <span id="vue-html-quotes"></span>[`vue/html-quotes`](./reference/vue-html-quotes.md) | Aplicar um padrão de aspas aos atributos HTML. | ✓ |
| <span id="vue-mustache-interpolation-spacing"></span>[`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | Aplicar espaçamento consistente dentro das interpolações com chaves duplas. | ✓ |
| <span id="vue-no-invalid-html-attribute"></span>[`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | Proibir valores estáticos inválidos para atributos HTML. |  |
| <span id="vue-no-lone-template"></span>[`vue/no-lone-template`](./reference/vue-no-lone-template.md) | Proibir elementos `<template>` desnecessários. |  |
| <span id="vue-no-multi-spaces"></span>[`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | Proibir vários espaços consecutivos. | ✓ |
| <span id="vue-no-template-shadow"></span>[`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | Proibir nomes de variáveis que ocultam variáveis de um escopo externo. |  |
| <span id="vue-no-template-target-blank"></span>[`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | Proibir target="_blank" sem rel="noopener noreferrer". |  |
| <span id="vue-no-this-in-before-route-enter"></span>[`vue/no-this-in-before-route-enter`](./reference/vue-no-this-in-before-route-enter.md) | Proíbe acessar com `this` uma instância ainda não criada em `beforeRouteEnter`. |  |
| <span id="vue-no-undefined-refs"></span>[`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | Proibir referências a variáveis não definidas nos templates. |  |
| <span id="vue-no-unsandboxed-iframe"></span>[`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | Exigir um atributo sandbox nos elementos iframe. |  |
| <span id="vue-no-unused-properties"></span>[`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | Proibir propriedades não usadas definidas em defineProps. |  |
| <span id="vue-no-unused-setup-bindings"></span>[`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | Proibir variáveis de script setup que nunca são lidas. |  |
| <span id="vue-no-use-v-else-with-v-for"></span>[`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | Proibir `v-else-if` ou `v-else` no mesmo elemento que `v-for`. |  |
| <span id="vue-permitted-contents"></span>[`vue/permitted-contents`](./reference/vue-permitted-contents.md) | Aplicar as regras do modelo de conteúdo HTML. |  |
| <span id="vue-prop-name-casing"></span>[`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | Aplicar um padrão de maiúsculas e minúsculas aos nomes de props declaradas. |  |
| <span id="vue-require-scoped-style"></span>[`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | Exigir o atributo scoped nas tags style. |  |
| <span id="vue-sfc-element-order"></span>[`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | Aplicar uma ordem consistente aos elementos de nível superior dos SFCs. |  |
| <span id="vue-single-style-block"></span>[`vue/single-style-block`](./reference/vue-single-style-block.md) | Recomendar um único bloco style. |  |
| <span id="vue-this-in-template"></span>[`vue/this-in-template`](./reference/vue-this-in-template.md) | Proibir `this.` nas expressões do template. |  |
| <span id="vue-v-bind-style"></span>[`vue/v-bind-style`](./reference/vue-v-bind-style.md) | Aplicar um padrão de sintaxe à diretiva `v-bind`. | ✓ |
| <span id="vue-v-on-event-hyphenation"></span>[`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | Exigir hífens nos nomes de eventos personalizados em v-on de componentes. | ✓ |
| <span id="vue-v-on-style"></span>[`vue/v-on-style`](./reference/vue-v-on-style.md) | Aplicar um padrão de sintaxe à diretiva `v-on`. | ✓ |
| <span id="vue-v-slot-style"></span>[`vue/v-slot-style`](./reference/vue-v-slot-style.md) | Aplicar um padrão de sintaxe à diretiva `v-slot`. | ✓ |

## Predefinições mais rígidas (29)

Só em `opinionated`, `nuxt` ou `ecosystem`. A coluna Predefinições mostra quais.

| Regra | O que verifica | Predefinições | Correção automática |
| --- | --- | --- | --- |
| <span id="vue-component-name-in-template-casing"></span>[`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | Aplicar um padrão específico de maiúsculas e minúsculas aos nomes de componentes nos templates. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-html-button-has-type"></span>[`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | Exigir um type explícito e válido nos elementos button. | `nuxt`, `opinionated` |  |
| <span id="vue-html-self-closing"></span>[`vue/html-self-closing`](./reference/vue-html-self-closing.md) | Aplicar um padrão de tags com fechamento automático. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-array-index-key"></span>[`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | Proibir o uso direto da variável de índice de v-for como :key. | `nuxt`, `opinionated` |  |
| <span id="vue-no-boolean-attr-value"></span>[`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | Proibir valores explícitos em atributos HTML booleanos. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-empty-component-block"></span>[`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | Proibir blocos vazios em SFCs. | `nuxt`, `opinionated` |  |
| <span id="vue-no-inline-style"></span>[`vue/no-inline-style`](./reference/vue-no-inline-style.md) | Desencorajar o uso de atributos de estilo inline. | `nuxt`, `opinionated` |  |
| <span id="vue-no-multiple-objects-in-class"></span>[`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | Proibir vários objetos literais dentro de uma vinculação de array em :class. | `nuxt`, `opinionated` |  |
| <span id="vue-no-negated-v-if-condition"></span>[`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | Proibir uma condição negada em v-if quando a cadeia tiver v-else. | `nuxt`, `opinionated` |  |
| <span id="vue-no-preprocessor-lang"></span>[`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | Desencorajar o uso de preprocessadores CSS em favor de CSS moderno. | `nuxt`, `opinionated` |  |
| <span id="vue-no-root-v-if"></span>[`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | Proibir v-if no único elemento raiz de um template. | `nuxt`, `opinionated` |  |
| <span id="vue-no-script-non-standard-lang"></span>[`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | Desencorajar valores não padronizados de lang em scripts. | `nuxt`, `opinionated` |  |
| <span id="vue-no-src-attribute"></span>[`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | Desencorajar o atributo src em blocos de SFCs. | `nuxt`, `opinionated` |  |
| <span id="vue-no-template-lang"></span>[`vue/no-template-lang`](./reference/vue-no-template-lang.md) | Desencorajar o atributo lang no bloco template. | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-refs"></span>[`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | Reportar refs de template (ref="x") nunca referenciadas em &lt;script&gt;. | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-script-vars"></span>[`vue/no-unused-script-vars`](./reference/vue-no-unused-script-vars.md) | Proibir variáveis, importações e parâmetros de script não usados. | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-mustaches"></span>[`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | Proibir interpolação com chaves duplas cuja expressão seja uma string literal constante. | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-v-bind"></span>[`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | Proibir um v-bind cujo valor seja uma string literal simples. | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-text"></span>[`vue/no-v-text`](./reference/vue-no-v-text.md) | Proibir a diretiva v-text; preferir interpolação com chaves duplas. | `nuxt`, `opinionated` |  |
| <span id="vue-prefer-const"></span>[`vue/prefer-const`](./reference/vue-prefer-const.md) | Exigir const para vínculos que o script e o template nunca reatribuem. | `nuxt`, `opinionated` |  |
| <span id="vue-prefer-props-shorthand"></span>[`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | Recomendar sintaxe abreviada para props (Vue 3.4+). | `nuxt`, `opinionated` | ✓ |
| <span id="vue-prefer-true-attribute-shorthand"></span>[`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | Preferir a forma abreviada para um atributo booleano vinculado a `true`. | `nuxt`, `opinionated` |  |
| <span id="vue-require-component-registration"></span>[`vue/require-component-registration`](./reference/vue-require-component-registration.md) | Exigir importação ou registro explícito de componentes. | `opinionated` |  |
| <span id="vue-scoped-event-names"></span>[`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | Recomendar nomes de eventos com escopo no formato context:event. | `nuxt`, `opinionated` |  |
| <span id="vue-slot-name-casing"></span>[`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | Exigir a capitalização configurada nos nomes estáticos de pontos de inserção de slot renderizados. | `nuxt`, `opinionated` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos. | `nuxt`, `opinionated` |  |
| <span id="vue-v-on-handler-style"></span>[`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | Exigir manipuladores de v-on escritos como referência a método. | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-block"></span>[`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | Alertar sobre blocos personalizados em arquivos SFC. | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-directive"></span>[`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | Alertar sobre diretivas personalizadas que precisam de registro. | `nuxt`, `opinionated` |  |

## Opcionais (59)

Não fazem parte de nenhuma predefinição. Ative pelo nome em `linter.rules`, passado a `withVue`.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="max-lines"></span>[`max-lines`](./reference/max-lines.md) | Limita o número de linhas de um arquivo. |  |
| <span id="vue-a11y-img-alt"></span>[`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | Exigir o atributo alt nas imagens para garantir acessibilidade. |  |
| <span id="vue-block-lang"></span>[`vue/block-lang`](./reference/vue-block-lang.md) | Restringe o lang do bloco SFC ao idioma configurado. |  |
| <span id="vue-camelcase"></span>[`vue/camelcase`](./reference/vue-camelcase.md) | Exige camelCase nos identificadores de expressões do template. |  |
| <span id="vue-dot-notation"></span>[`vue/dot-notation`](./reference/vue-dot-notation.md) | Prefere notação de ponto para propriedades estáticas. |  |
| <span id="vue-enforce-style-attribute"></span>[`vue/enforce-style-attribute`](./reference/vue-enforce-style-attribute.md) | Restringe blocos style aos modos permitidos. |  |
| <span id="vue-eqeqeq"></span>[`vue/eqeqeq`](./reference/vue-eqeqeq.md) | Exige igualdade estrita em expressões do template. |  |
| <span id="vue-match-component-file-name"></span>[`vue/match-component-file-name`](./reference/vue-match-component-file-name.md) | Exige que o nome declarado do componente corresponda ao nome do arquivo. |  |
| <span id="vue-max-lines-per-block"></span>[`vue/max-lines-per-block`](./reference/vue-max-lines-per-block.md) | Limita o número de linhas de cada bloco SFC. |  |
| <span id="vue-max-props"></span>[`vue/max-props`](./reference/vue-max-props.md) | Limita o número de props de um componente Vue. |  |
| <span id="vue-max-template-complexity"></span>[`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | Limitar a complexidade do próprio template de um componente, tanto ciclomática quanto cognitiva. |  |
| <span id="vue-max-template-depth"></span>[`vue/max-template-depth`](./reference/vue-max-template-depth.md) | Limita a profundidade do template. |  |
| <span id="vue-next-tick-style"></span>[`vue/next-tick-style`](./reference/vue-next-tick-style.md) | Exige o estilo Promise no uso de nextTick. |  |
| <span id="vue-no-bare-strings-in-template"></span>[`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado. |  |
| <span id="vue-no-console"></span>[`vue/no-console`](./reference/vue-no-console.md) | Proíbe acesso ao console em expressões do template. |  |
| <span id="vue-no-constant-condition"></span>[`vue/no-constant-condition`](./reference/vue-no-constant-condition.md) | Proíbe condições constantes em expressões do template. |  |
| <span id="vue-no-content-outside-block"></span>[`vue/no-content-outside-block`](./reference/vue-no-content-outside-block.md) | Proíbe conteúdo fora dos blocos superiores do SFC. |  |
| <span id="vue-no-custom-modifiers-on-v-model"></span>[`vue/no-custom-modifiers-on-v-model`](./reference/vue-no-custom-modifiers-on-v-model.md) | Proíbe modificadores personalizados no `v-model` de componentes para manter compatibilidade com Vue 2. |  |
| <span id="vue-no-define-model-default"></span>[`vue/no-define-model-default`](./reference/vue-no-define-model-default.md) | Proíbe valores padrão em defineModel. |  |
| <span id="vue-no-empty-pattern"></span>[`vue/no-empty-pattern`](./reference/vue-no-empty-pattern.md) | Proíbe padrões de desestruturação vazios no template. |  |
| <span id="vue-no-implicit-coercion"></span>[`vue/no-implicit-coercion`](./reference/vue-no-implicit-coercion.md) | Proíbe coerção implícita em expressões do template. |  |
| <span id="vue-no-irregular-whitespace"></span>[`vue/no-irregular-whitespace`](./reference/vue-no-irregular-whitespace.md) | Detecta espaços irregulares em expressões do template. |  |
| <span id="vue-no-literals-in-template"></span>[`vue/no-literals-in-template`](./reference/vue-no-literals-in-template.md) | Proíbe literais de objeto, array e função em v-bind. |  |
| <span id="vue-no-loss-of-precision"></span>[`vue/no-loss-of-precision`](./reference/vue-no-loss-of-precision.md) | Proíbe literais numéricos que perdem precisão. |  |
| <span id="vue-no-multiple-template-root"></span>[`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | Proibir vários nós raiz em um template. |  |
| <span id="vue-no-negated-condition"></span>[`vue/no-negated-condition`](./reference/vue-no-negated-condition.md) | Proíbe ternários iniciados por uma condição negada. |  |
| <span id="vue-no-props-shadow"></span>[`vue/no-props-shadow`](./reference/vue-no-props-shadow.md) | Proíbe declarações de script setup que ocultam props. |  |
| <span id="vue-no-ref-object-reactivity-loss"></span>[`vue/no-ref-object-reactivity-loss`](./reference/vue-no-ref-object-reactivity-loss.md) | Proíbe copiar o valor de um ref no mesmo escopo e perder suas atualizações reativas. |  |
| <span id="vue-no-restricted-block"></span>[`vue/no-restricted-block`](./reference/vue-no-restricted-block.md) | Proíbe o bloco configurado. |  |
| <span id="vue-no-restricted-call-after-await"></span>[`vue/no-restricted-call-after-await`](./reference/vue-no-restricted-call-after-await.md) | Proíbe chamar um método restrito depois de await. |  |
| <span id="vue-no-restricted-class"></span>[`vue/no-restricted-class`](./reference/vue-no-restricted-class.md) | Proíbe a class configurada. |  |
| <span id="vue-no-restricted-component-names"></span>[`vue/no-restricted-component-names`](./reference/vue-no-restricted-component-names.md) | Proíbe o nome de componente configurado. |  |
| <span id="vue-no-restricted-component-options"></span>[`vue/no-restricted-component-options`](./reference/vue-no-restricted-component-options.md) | Proíbe a opção de componente configurada. |  |
| <span id="vue-no-restricted-custom-event"></span>[`vue/no-restricted-custom-event`](./reference/vue-no-restricted-custom-event.md) | Proíbe o evento personalizado configurado. |  |
| <span id="vue-no-restricted-html-elements"></span>[`vue/no-restricted-html-elements`](./reference/vue-no-restricted-html-elements.md) | Proíbe o elemento configurado. |  |
| <span id="vue-no-restricted-props"></span>[`vue/no-restricted-props`](./reference/vue-no-restricted-props.md) | Proíbe a prop configurada. |  |
| <span id="vue-no-restricted-static-attribute"></span>[`vue/no-restricted-static-attribute`](./reference/vue-no-restricted-static-attribute.md) | Proíbe o atributo estático configurado. |  |
| <span id="vue-no-restricted-syntax"></span>[`vue/no-restricted-syntax`](./reference/vue-no-restricted-syntax.md) | Proíbe sintaxe configurada nas expressões do template. |  |
| <span id="vue-no-restricted-v-bind"></span>[`vue/no-restricted-v-bind`](./reference/vue-no-restricted-v-bind.md) | Proíbe argumentos restritos de v-bind. |  |
| <span id="vue-no-restricted-v-on"></span>[`vue/no-restricted-v-on`](./reference/vue-no-restricted-v-on.md) | Proíbe o argumento de v-on configurado. |  |
| <span id="vue-no-setup-props-reactivity-loss"></span>[`vue/no-setup-props-reactivity-loss`](./reference/vue-no-setup-props-reactivity-loss.md) | Proíbe cópias dos valores das props de setup que perdem as atualizações reativas. |  |
| <span id="vue-no-shadow-native-events"></span>[`vue/no-shadow-native-events`](./reference/vue-no-shadow-native-events.md) | Proíbe eventos de componente que colidem com eventos nativos. |  |
| <span id="vue-no-sparse-arrays"></span>[`vue/no-sparse-arrays`](./reference/vue-no-sparse-arrays.md) | Proíbe arrays esparsos em expressões do template. |  |
| <span id="vue-no-static-inline-styles"></span>[`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | Proibir atributos estáticos de estilo inline. |  |
| <span id="vue-no-unsupported-features"></span>[`vue/no-unsupported-features`](./reference/vue-no-unsupported-features.md) | Proíbe sintaxe ainda não suportada na versão configurada do Vue.js. |  |
| <span id="vue-no-useless-concat"></span>[`vue/no-useless-concat`](./reference/vue-no-useless-concat.md) | Proíbe concatenação inútil de literais de string. |  |
| <span id="vue-no-v-for-template-key"></span>[`vue/no-v-for-template-key`](./reference/vue-no-v-for-template-key.md) | Exige que a key de `template v-for` fique em um filho real para manter compatibilidade com Vue 2. |  |
| <span id="vue-object-shorthand"></span>[`vue/object-shorthand`](./reference/vue-object-shorthand.md) | Exige propriedades abreviadas nos objetos do template. |  |
| <span id="vue-prefer-prop-type-boolean-first"></span>[`vue/prefer-prop-type-boolean-first`](./reference/vue-prefer-prop-type-boolean-first.md) | Exige `Boolean` na primeira posição da lista de construtores de tipo das props. |  |
| <span id="vue-prefer-template"></span>[`vue/prefer-template`](./reference/vue-prefer-template.md) | Prefere template literals à concatenação de strings. |  |
| <span id="vue-prefer-v-model"></span>[`vue/prefer-v-model`](./reference/vue-prefer-v-model.md) | Prefere v-model a um bind e listener update equivalentes. |  |
| <span id="vue-quote-props"></span>[`vue/quote-props`](./reference/vue-quote-props.md) | Proíbe aspas desnecessárias em propriedades de objetos. |  |
| <span id="vue-require-default-export"></span>[`vue/require-default-export`](./reference/vue-require-default-export.md) | Exige que o script do componente seja o export padrão. |  |
| <span id="vue-require-expose"></span>[`vue/require-expose`](./reference/vue-require-expose.md) | Exige que componentes da Options API declarem explicitamente suas propriedades públicas com `expose`. |  |
| <span id="vue-require-macro-variable-name"></span>[`vue/require-macro-variable-name`](./reference/vue-require-macro-variable-name.md) | Exige nomes convencionais para variáveis de macros. |  |
| <span id="vue-require-prop-comment"></span>[`vue/require-prop-comment`](./reference/vue-require-prop-comment.md) | Exige um comentário JSDoc em cada prop declarada. |  |
| <span id="vue-sort-keys"></span>[`vue/sort-keys`](./reference/vue-sort-keys.md) | Ordena as chaves de opções do componente conforme configurado. |  |
| <span id="vue-v-if-else-key"></span>[`vue/v-if-else-key`](./reference/vue-v-if-else-key.md) | Exige key em componentes condicionais repetidos. |  |
| <span id="vue-valid-model-definition"></span>[`vue/valid-model-definition`](./reference/vue-valid-model-definition.md) | Exige chaves válidas na opção `model` do Vue 2. |  |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
