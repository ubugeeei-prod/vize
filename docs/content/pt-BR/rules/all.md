---
title: "Todas as regras de lint"
description: "Todas as regras do linter do Vize, agrupadas por categoria. Abra uma regra para ver o que ela reporta, um exemplo incorreto e um correto, e como ativá-la. A coluna Predefinição indica a menor predefinição que ativa a regra; a página de cada regra lista todas. Veja em [predefinições](../rules/index.md#predefinições) o que cada uma contém."
---

# Todas as regras de lint

## Regras Vue (168)

| Regra | O que verifica | Predefinição | Correção automática |
| --- | --- | --- | --- |
| <span id="max-lines"></span>[`max-lines`](./reference/max-lines.md) | Limita o número de linhas de um arquivo. | — |  |
| <span id="vue-a11y-img-alt"></span>[`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | Exigir o atributo alt nas imagens para garantir acessibilidade. | — |  |
| <span id="vue-attribute-hyphenation"></span>[`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | Aplicar um padrão de nomes de atributos em componentes personalizados. | `happy-path` | ✓ |
| <span id="vue-attribute-order"></span>[`vue/attribute-order`](./reference/vue-attribute-order.md) | Aplicar uma ordem consistente aos atributos. | `happy-path` |  |
| <span id="vue-block-lang"></span>[`vue/block-lang`](./reference/vue-block-lang.md) | Restringe o lang do bloco SFC ao idioma configurado. | — |  |
| <span id="vue-camelcase"></span>[`vue/camelcase`](./reference/vue-camelcase.md) | Exige camelCase nos identificadores de expressões do template. | — |  |
| <span id="vue-comment-directive"></span>[`vue/comment-directive`](./reference/vue-comment-directive.md) | Aceita comentários de controle de diagnósticos em `<template>`. | `essential` |  |
| <span id="vue-component-definition-name-casing"></span>[`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | Exigir PascalCase ou kebab-case nos nomes de definição de componentes. | `happy-path` |  |
| <span id="vue-component-name-in-template-casing"></span>[`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | Aplicar um padrão específico de maiúsculas e minúsculas aos nomes de componentes nos templates. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-dot-notation"></span>[`vue/dot-notation`](./reference/vue-dot-notation.md) | Prefere notação de ponto para propriedades estáticas. | — |  |
| <span id="vue-enforce-style-attribute"></span>[`vue/enforce-style-attribute`](./reference/vue-enforce-style-attribute.md) | Restringe blocos style aos modos permitidos. | — |  |
| <span id="vue-eqeqeq"></span>[`vue/eqeqeq`](./reference/vue-eqeqeq.md) | Exige igualdade estrita em expressões do template. | — |  |
| <span id="vue-html-button-has-type"></span>[`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | Exigir um type explícito e válido nos elementos button. | `nuxt`, `opinionated` |  |
| <span id="vue-html-end-tags"></span>[`vue/html-end-tags`](./reference/vue-html-end-tags.md) | Exige tags de fechamento explícitas para elementos HTML cujo fechamento pode ser omitido. | `happy-path` | ✓ |
| <span id="vue-html-quotes"></span>[`vue/html-quotes`](./reference/vue-html-quotes.md) | Aplicar um padrão de aspas aos atributos HTML. | `happy-path` | ✓ |
| <span id="vue-html-self-closing"></span>[`vue/html-self-closing`](./reference/vue-html-self-closing.md) | Aplicar um padrão de tags com fechamento automático. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-jsx-uses-vars"></span>[`vue/jsx-uses-vars`](./reference/vue-jsx-uses-vars.md) | Considera usadas as variáveis referenciadas nos nomes dos elementos JSX. | `essential` |  |
| <span id="vue-match-component-file-name"></span>[`vue/match-component-file-name`](./reference/vue-match-component-file-name.md) | Exige que o nome declarado do componente corresponda ao nome do arquivo. | — |  |
| <span id="vue-max-lines-per-block"></span>[`vue/max-lines-per-block`](./reference/vue-max-lines-per-block.md) | Limita o número de linhas de cada bloco SFC. | — |  |
| <span id="vue-max-props"></span>[`vue/max-props`](./reference/vue-max-props.md) | Limita o número de props de um componente Vue. | — |  |
| <span id="vue-max-template-complexity"></span>[`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | Limitar a complexidade do próprio template de um componente, tanto ciclomática quanto cognitiva. | — |  |
| <span id="vue-max-template-depth"></span>[`vue/max-template-depth`](./reference/vue-max-template-depth.md) | Limita a profundidade do template. | — |  |
| <span id="vue-multi-word-component-names"></span>[`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | Exigir nomes de componentes com mais de uma palavra. | `essential` |  |
| <span id="vue-mustache-interpolation-spacing"></span>[`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | Aplicar espaçamento consistente dentro das interpolações com chaves duplas. | `happy-path` | ✓ |
| <span id="vue-next-tick-style"></span>[`vue/next-tick-style`](./reference/vue-next-tick-style.md) | Exige o estilo Promise no uso de nextTick. | — |  |
| <span id="vue-no-array-index-key"></span>[`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | Proibir o uso direto da variável de índice de v-for como :key. | `nuxt`, `opinionated` |  |
| <span id="vue-no-bare-strings-in-template"></span>[`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado. | — |  |
| <span id="vue-no-boolean-attr-value"></span>[`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | Proibir valores explícitos em atributos HTML booleanos. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-child-content"></span>[`vue/no-child-content`](./reference/vue-no-child-content.md) | Proibir conteúdo filho ao usar v-html ou v-text. | `essential` |  |
| <span id="vue-no-console"></span>[`vue/no-console`](./reference/vue-no-console.md) | Proíbe acesso ao console em expressões do template. | — |  |
| <span id="vue-no-constant-condition"></span>[`vue/no-constant-condition`](./reference/vue-no-constant-condition.md) | Proíbe condições constantes em expressões do template. | — |  |
| <span id="vue-no-content-outside-block"></span>[`vue/no-content-outside-block`](./reference/vue-no-content-outside-block.md) | Proíbe conteúdo fora dos blocos superiores do SFC. | — |  |
| <span id="vue-no-custom-modifiers-on-v-model"></span>[`vue/no-custom-modifiers-on-v-model`](./reference/vue-no-custom-modifiers-on-v-model.md) | Proíbe modificadores personalizados no `v-model` de componentes para manter compatibilidade com Vue 2. | — |  |
| <span id="vue-no-define-model-default"></span>[`vue/no-define-model-default`](./reference/vue-no-define-model-default.md) | Proíbe valores padrão em defineModel. | — |  |
| <span id="vue-no-deprecated-filter"></span>[`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | Proibir a sintaxe obsoleta de filtros do Vue 2 com o operador de barra vertical. | `essential` |  |
| <span id="vue-no-deprecated-functional-template"></span>[`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | Proibir o atributo `functional` no `<template>` de um SFC. | `essential` |  |
| <span id="vue-no-deprecated-html-element-is"></span>[`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | Proibir o atributo `is` em elementos HTML nativos. | `essential` |  |
| <span id="vue-no-deprecated-inline-template"></span>[`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | Proibir o atributo obsoleto `inline-template`. | `essential` |  |
| <span id="vue-no-deprecated-router-link-tag-prop"></span>[`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | Proibir a prop `tag` em &lt;router-link&gt;. | `essential` |  |
| <span id="vue-no-deprecated-scope-attribute"></span>[`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | Proibir o atributo obsoleto `scope` em &lt;template&gt;. | `essential` |  |
| <span id="vue-no-deprecated-slot-attribute"></span>[`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | Proibir o atributo obsoleto `slot`. | `essential` |  |
| <span id="vue-no-deprecated-slot-scope-attribute"></span>[`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | Proibir o atributo obsoleto `slot-scope`. | `essential` |  |
| <span id="vue-no-deprecated-v-bind-sync"></span>[`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | Proibir o modificador obsoleto `.sync` em `v-bind`. | `essential` |  |
| <span id="vue-no-deprecated-v-is"></span>[`vue/no-deprecated-v-is`](./reference/vue-no-deprecated-v-is.md) | Proibir a diretiva obsoleta `v-is`. | `essential` |  |
| <span id="vue-no-deprecated-v-on-native-modifier"></span>[`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | Proibir o modificador obsoleto `.native` em `v-on`. | `essential` |  |
| <span id="vue-no-deprecated-v-on-number-modifiers"></span>[`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | Proibir modificadores numéricos obsoletos de `keyCode` em `v-on`. | `essential` | ✓ |
| <span id="vue-no-dupe-v-else-if"></span>[`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | Proibir condições duplicadas em cadeias de `v-if` / `v-else-if`. | `essential` |  |
| <span id="vue-no-duplicate-attributes"></span>[`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | Proibir atributos duplicados no mesmo elemento. | `essential` |  |
| <span id="vue-no-empty-component-block"></span>[`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | Proibir blocos vazios em SFCs. | `nuxt`, `opinionated` |  |
| <span id="vue-no-empty-pattern"></span>[`vue/no-empty-pattern`](./reference/vue-no-empty-pattern.md) | Proíbe padrões de desestruturação vazios no template. | — |  |
| <span id="vue-no-implicit-coercion"></span>[`vue/no-implicit-coercion`](./reference/vue-no-implicit-coercion.md) | Proíbe coerção implícita em expressões do template. | — |  |
| <span id="vue-no-inline-style"></span>[`vue/no-inline-style`](./reference/vue-no-inline-style.md) | Desencorajar o uso de atributos de estilo inline. | `nuxt`, `opinionated` |  |
| <span id="vue-no-invalid-html-attribute"></span>[`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | Proibir valores estáticos inválidos para atributos HTML. | `happy-path` |  |
| <span id="vue-no-irregular-whitespace"></span>[`vue/no-irregular-whitespace`](./reference/vue-no-irregular-whitespace.md) | Detecta espaços irregulares em expressões do template. | — |  |
| <span id="vue-no-literals-in-template"></span>[`vue/no-literals-in-template`](./reference/vue-no-literals-in-template.md) | Proíbe literais de objeto, array e função em v-bind. | — |  |
| <span id="vue-no-lone-template"></span>[`vue/no-lone-template`](./reference/vue-no-lone-template.md) | Proibir elementos `<template>` desnecessários. | `happy-path` |  |
| <span id="vue-no-loss-of-precision"></span>[`vue/no-loss-of-precision`](./reference/vue-no-loss-of-precision.md) | Proíbe literais numéricos que perdem precisão. | — |  |
| <span id="vue-no-multi-spaces"></span>[`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | Proibir vários espaços consecutivos. | `happy-path` | ✓ |
| <span id="vue-no-multiple-objects-in-class"></span>[`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | Proibir vários objetos literais dentro de uma vinculação de array em :class. | `nuxt`, `opinionated` |  |
| <span id="vue-no-multiple-template-root"></span>[`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | Proibir vários nós raiz em um template. | — |  |
| <span id="vue-no-mutating-props"></span>[`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | Proibir a mutação de props de componentes. | `essential` |  |
| <span id="vue-no-negated-condition"></span>[`vue/no-negated-condition`](./reference/vue-no-negated-condition.md) | Proíbe ternários iniciados por uma condição negada. | — |  |
| <span id="vue-no-negated-v-if-condition"></span>[`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | Proibir uma condição negada em v-if quando a cadeia tiver v-else. | `nuxt`, `opinionated` |  |
| <span id="vue-no-non-component-keep-alive-child"></span>[`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | Proibir invólucros de elementos comuns diretamente abaixo de `<KeepAlive>`. | `essential` |  |
| <span id="vue-no-preprocessor-lang"></span>[`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | Desencorajar o uso de preprocessadores CSS em favor de CSS moderno. | `nuxt`, `opinionated` |  |
| <span id="vue-no-props-shadow"></span>[`vue/no-props-shadow`](./reference/vue-no-props-shadow.md) | Proíbe declarações de script setup que ocultam props. | — |  |
| <span id="vue-no-ref-object-reactivity-loss"></span>[`vue/no-ref-object-reactivity-loss`](./reference/vue-no-ref-object-reactivity-loss.md) | Proíbe copiar o valor de um ref no mesmo escopo e perder suas atualizações reativas. | — |  |
| <span id="vue-no-reserved-component-names"></span>[`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | Proibir o uso de nomes reservados como nomes de componentes. | `essential` |  |
| <span id="vue-no-restricted-block"></span>[`vue/no-restricted-block`](./reference/vue-no-restricted-block.md) | Proíbe o bloco configurado. | — |  |
| <span id="vue-no-restricted-call-after-await"></span>[`vue/no-restricted-call-after-await`](./reference/vue-no-restricted-call-after-await.md) | Proíbe chamar um método restrito depois de await. | — |  |
| <span id="vue-no-restricted-class"></span>[`vue/no-restricted-class`](./reference/vue-no-restricted-class.md) | Proíbe a class configurada. | — |  |
| <span id="vue-no-restricted-component-names"></span>[`vue/no-restricted-component-names`](./reference/vue-no-restricted-component-names.md) | Proíbe o nome de componente configurado. | — |  |
| <span id="vue-no-restricted-component-options"></span>[`vue/no-restricted-component-options`](./reference/vue-no-restricted-component-options.md) | Proíbe a opção de componente configurada. | — |  |
| <span id="vue-no-restricted-custom-event"></span>[`vue/no-restricted-custom-event`](./reference/vue-no-restricted-custom-event.md) | Proíbe o evento personalizado configurado. | — |  |
| <span id="vue-no-restricted-html-elements"></span>[`vue/no-restricted-html-elements`](./reference/vue-no-restricted-html-elements.md) | Proíbe o elemento configurado. | — |  |
| <span id="vue-no-restricted-props"></span>[`vue/no-restricted-props`](./reference/vue-no-restricted-props.md) | Proíbe a prop configurada. | — |  |
| <span id="vue-no-restricted-static-attribute"></span>[`vue/no-restricted-static-attribute`](./reference/vue-no-restricted-static-attribute.md) | Proíbe o atributo estático configurado. | — |  |
| <span id="vue-no-restricted-syntax"></span>[`vue/no-restricted-syntax`](./reference/vue-no-restricted-syntax.md) | Proíbe sintaxe configurada nas expressões do template. | — |  |
| <span id="vue-no-restricted-v-bind"></span>[`vue/no-restricted-v-bind`](./reference/vue-no-restricted-v-bind.md) | Proíbe argumentos restritos de v-bind. | — |  |
| <span id="vue-no-restricted-v-on"></span>[`vue/no-restricted-v-on`](./reference/vue-no-restricted-v-on.md) | Proíbe o argumento de v-on configurado. | — |  |
| <span id="vue-no-root-v-if"></span>[`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | Proibir v-if no único elemento raiz de um template. | `nuxt`, `opinionated` |  |
| <span id="vue-no-script-non-standard-lang"></span>[`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | Desencorajar valores não padronizados de lang em scripts. | `nuxt`, `opinionated` |  |
| <span id="vue-no-setup-props-reactivity-loss"></span>[`vue/no-setup-props-reactivity-loss`](./reference/vue-no-setup-props-reactivity-loss.md) | Proíbe cópias dos valores das props de setup que perdem as atualizações reativas. | — |  |
| <span id="vue-no-shadow-native-events"></span>[`vue/no-shadow-native-events`](./reference/vue-no-shadow-native-events.md) | Proíbe eventos de componente que colidem com eventos nativos. | — |  |
| <span id="vue-no-sparse-arrays"></span>[`vue/no-sparse-arrays`](./reference/vue-no-sparse-arrays.md) | Proíbe arrays esparsos em expressões do template. | — |  |
| <span id="vue-no-src-attribute"></span>[`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | Desencorajar o atributo src em blocos de SFCs. | `nuxt`, `opinionated` |  |
| <span id="vue-no-static-inline-styles"></span>[`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | Proibir atributos estáticos de estilo inline. | — |  |
| <span id="vue-no-template-key"></span>[`vue/no-template-key`](./reference/vue-no-template-key.md) | Proibir o atributo `key` em `<template>`. | `essential` |  |
| <span id="vue-no-template-lang"></span>[`vue/no-template-lang`](./reference/vue-no-template-lang.md) | Desencorajar o atributo lang no bloco template. | `nuxt`, `opinionated` |  |
| <span id="vue-no-template-shadow"></span>[`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | Proibir nomes de variáveis que ocultam variáveis de um escopo externo. | `happy-path` |  |
| <span id="vue-no-template-target-blank"></span>[`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | Proibir target="_blank" sem rel="noopener noreferrer". | `happy-path` |  |
| <span id="vue-no-textarea-mustache"></span>[`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | Proibir interpolação com chaves duplas em `<textarea>`. | `essential` |  |
| <span id="vue-no-this-in-before-route-enter"></span>[`vue/no-this-in-before-route-enter`](./reference/vue-no-this-in-before-route-enter.md) | Proíbe acessar com `this` uma instância ainda não criada em `beforeRouteEnter`. | `happy-path` |  |
| <span id="vue-no-undefined-refs"></span>[`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | Proibir referências a variáveis não definidas nos templates. | `happy-path` |  |
| <span id="vue-no-unsafe-url"></span>[`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | Alertar sobre vinculações de URL potencialmente inseguras. | `essential` |  |
| <span id="vue-no-unsandboxed-iframe"></span>[`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | Exigir um atributo sandbox nos elementos iframe. | `happy-path` |  |
| <span id="vue-no-unsupported-features"></span>[`vue/no-unsupported-features`](./reference/vue-no-unsupported-features.md) | Proíbe sintaxe ainda não suportada na versão configurada do Vue.js. | — |  |
| <span id="vue-no-unused-components"></span>[`vue/no-unused-components`](./reference/vue-no-unused-components.md) | Proibir o registro de componentes não usados nos templates. | `essential` |  |
| <span id="vue-no-unused-properties"></span>[`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | Proibir propriedades não usadas definidas em defineProps. | `happy-path` |  |
| <span id="vue-no-unused-refs"></span>[`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | Reportar refs de template (ref="x") nunca referenciadas em &lt;script&gt;. | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-script-vars"></span>[`vue/no-unused-script-vars`](./reference/vue-no-unused-script-vars.md) | Proibir variáveis, importações e parâmetros de script não usados. | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-setup-bindings"></span>[`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | Proibir variáveis de script setup que nunca são lidas. | `happy-path` |  |
| <span id="vue-no-unused-vars"></span>[`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | Proibir definições de variáveis não usadas nas diretivas v-for e v-slot. | `essential` |  |
| <span id="vue-no-use-v-else-with-v-for"></span>[`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | Proibir `v-else-if` ou `v-else` no mesmo elemento que `v-for`. | `happy-path` |  |
| <span id="vue-no-use-v-if-with-v-for"></span>[`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | Proibir `v-if` no mesmo elemento que `v-for`. | `essential` |  |
| <span id="vue-no-useless-concat"></span>[`vue/no-useless-concat`](./reference/vue-no-useless-concat.md) | Proíbe concatenação inútil de literais de string. | — |  |
| <span id="vue-no-useless-mustaches"></span>[`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | Proibir interpolação com chaves duplas cuja expressão seja uma string literal constante. | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-template-attributes"></span>[`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | Proibir atributos sem efeito em elementos `<template>`. | `essential` |  |
| <span id="vue-no-useless-v-bind"></span>[`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | Proibir um v-bind cujo valor seja uma string literal simples. | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-for-template-key"></span>[`vue/no-v-for-template-key`](./reference/vue-no-v-for-template-key.md) | Exige que a key de `template v-for` fique em um filho real para manter compatibilidade com Vue 2. | — |  |
| <span id="vue-no-v-for-template-key-on-child"></span>[`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | Proibir `key` no filho de um `<template v-for>`. | `essential` |  |
| <span id="vue-no-v-html"></span>[`vue/no-v-html`](./reference/vue-no-v-html.md) | Alertar sobre v-html para prevenir vulnerabilidades XSS. | `essential` |  |
| <span id="vue-no-v-text"></span>[`vue/no-v-text`](./reference/vue-no-v-text.md) | Proibir a diretiva v-text; preferir interpolação com chaves duplas. | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-text-v-html-on-component"></span>[`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | Proibir v-text / v-html em elementos de componente. | `essential` |  |
| <span id="vue-object-shorthand"></span>[`vue/object-shorthand`](./reference/vue-object-shorthand.md) | Exige propriedades abreviadas nos objetos do template. | — |  |
| <span id="vue-permitted-contents"></span>[`vue/permitted-contents`](./reference/vue-permitted-contents.md) | Aplicar as regras do modelo de conteúdo HTML. | `happy-path` |  |
| <span id="vue-prefer-const"></span>[`vue/prefer-const`](./reference/vue-prefer-const.md) | Exigir const para vínculos que o script e o template nunca reatribuem. | `nuxt`, `opinionated` |  |
| <span id="vue-prefer-prop-type-boolean-first"></span>[`vue/prefer-prop-type-boolean-first`](./reference/vue-prefer-prop-type-boolean-first.md) | Exige `Boolean` na primeira posição da lista de construtores de tipo das props. | — |  |
| <span id="vue-prefer-props-shorthand"></span>[`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | Recomendar sintaxe abreviada para props (Vue 3.4+). | `nuxt`, `opinionated` | ✓ |
| <span id="vue-prefer-template"></span>[`vue/prefer-template`](./reference/vue-prefer-template.md) | Prefere template literals à concatenação de strings. | — |  |
| <span id="vue-prefer-true-attribute-shorthand"></span>[`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | Preferir a forma abreviada para um atributo booleano vinculado a `true`. | `nuxt`, `opinionated` |  |
| <span id="vue-prefer-v-model"></span>[`vue/prefer-v-model`](./reference/vue-prefer-v-model.md) | Prefere v-model a um bind e listener update equivalentes. | — |  |
| <span id="vue-prop-name-casing"></span>[`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | Aplicar um padrão de maiúsculas e minúsculas aos nomes de props declaradas. | `happy-path` |  |
| <span id="vue-quote-props"></span>[`vue/quote-props`](./reference/vue-quote-props.md) | Proíbe aspas desnecessárias em propriedades de objetos. | — |  |
| <span id="vue-require-component-is"></span>[`vue/require-component-is`](./reference/vue-require-component-is.md) | Exigir `v-bind:is` em elementos `<component>`. | `essential` |  |
| <span id="vue-require-component-registration"></span>[`vue/require-component-registration`](./reference/vue-require-component-registration.md) | Exigir importação ou registro explícito de componentes. | `opinionated` |  |
| <span id="vue-require-default-export"></span>[`vue/require-default-export`](./reference/vue-require-default-export.md) | Exige que o script do componente seja o export padrão. | — |  |
| <span id="vue-require-expose"></span>[`vue/require-expose`](./reference/vue-require-expose.md) | Exige que componentes da Options API declarem explicitamente suas propriedades públicas com `expose`. | — |  |
| <span id="vue-require-macro-variable-name"></span>[`vue/require-macro-variable-name`](./reference/vue-require-macro-variable-name.md) | Exige nomes convencionais para variáveis de macros. | — |  |
| <span id="vue-require-prop-comment"></span>[`vue/require-prop-comment`](./reference/vue-require-prop-comment.md) | Exige um comentário JSDoc em cada prop declarada. | — |  |
| <span id="vue-require-scoped-style"></span>[`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | Exigir o atributo scoped nas tags style. | `happy-path` |  |
| <span id="vue-require-toggle-inside-transition"></span>[`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | Exigir uma alternância no elemento envolvido por `<transition>`. | `essential` |  |
| <span id="vue-require-v-for-key"></span>[`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | Exigir `v-bind:key` nas diretivas `v-for`. | `essential` |  |
| <span id="vue-scoped-event-names"></span>[`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | Recomendar nomes de eventos com escopo no formato context:event. | `nuxt`, `opinionated` |  |
| <span id="vue-sfc-element-order"></span>[`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | Aplicar uma ordem consistente aos elementos de nível superior dos SFCs. | `happy-path` |  |
| <span id="vue-single-style-block"></span>[`vue/single-style-block`](./reference/vue-single-style-block.md) | Recomendar um único bloco style. | `happy-path` |  |
| <span id="vue-slot-name-casing"></span>[`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | Exigir a capitalização configurada nos nomes estáticos de pontos de inserção de slot renderizados. | `nuxt`, `opinionated` |  |
| <span id="vue-sort-keys"></span>[`vue/sort-keys`](./reference/vue-sort-keys.md) | Ordena as chaves de opções do componente conforme configurado. | — |  |
| <span id="vue-this-in-template"></span>[`vue/this-in-template`](./reference/vue-this-in-template.md) | Proibir `this.` nas expressões do template. | `happy-path` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos. | `nuxt`, `opinionated` |  |
| <span id="vue-use-v-on-exact"></span>[`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | Exigir o modificador `.exact` em `v-on` quando houver manipuladores baseados em modificadores. | `essential` |  |
| <span id="vue-v-bind-style"></span>[`vue/v-bind-style`](./reference/vue-v-bind-style.md) | Aplicar um padrão de sintaxe à diretiva `v-bind`. | `happy-path` | ✓ |
| <span id="vue-v-if-else-key"></span>[`vue/v-if-else-key`](./reference/vue-v-if-else-key.md) | Exige key em componentes condicionais repetidos. | — |  |
| <span id="vue-v-on-event-hyphenation"></span>[`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | Exigir hífens nos nomes de eventos personalizados em v-on de componentes. | `happy-path` | ✓ |
| <span id="vue-v-on-handler-style"></span>[`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | Exigir manipuladores de v-on escritos como referência a método. | `nuxt`, `opinionated` |  |
| <span id="vue-v-on-style"></span>[`vue/v-on-style`](./reference/vue-v-on-style.md) | Aplicar um padrão de sintaxe à diretiva `v-on`. | `happy-path` | ✓ |
| <span id="vue-v-slot-style"></span>[`vue/v-slot-style`](./reference/vue-v-slot-style.md) | Aplicar um padrão de sintaxe à diretiva `v-slot`. | `happy-path` | ✓ |
| <span id="vue-valid-attribute-name"></span>[`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | Exigir nomes de atributos válidos. | `essential` |  |
| <span id="vue-valid-model-definition"></span>[`vue/valid-model-definition`](./reference/vue-valid-model-definition.md) | Exige chaves válidas na opção `model` do Vue 2. | — |  |
| <span id="vue-valid-template-root"></span>[`vue/valid-template-root`](./reference/vue-valid-template-root.md) | Exigir uma raiz `<template>` válida para a semântica de fragmentos do Vue 3. | `essential` |  |
| <span id="vue-valid-v-bind"></span>[`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | Exigir diretivas `v-bind` válidas. | `essential` |  |
| <span id="vue-valid-v-cloak"></span>[`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | Exigir diretivas `v-cloak` válidas. | `essential` |  |
| <span id="vue-valid-v-else"></span>[`vue/valid-v-else`](./reference/vue-valid-v-else.md) | Exigir diretivas `v-else` válidas. | `essential` | ✓ |
| <span id="vue-valid-v-else-if"></span>[`vue/valid-v-else-if`](./reference/vue-valid-v-else-if.md) | Exigir diretivas `v-else-if` válidas. | `essential` |  |
| <span id="vue-valid-v-for"></span>[`vue/valid-v-for`](./reference/vue-valid-v-for.md) | Exigir diretivas `v-for` válidas. | `essential` |  |
| <span id="vue-valid-v-html"></span>[`vue/valid-v-html`](./reference/vue-valid-v-html.md) | Exigir diretivas `v-html` válidas. | `essential` |  |
| <span id="vue-valid-v-if"></span>[`vue/valid-v-if`](./reference/vue-valid-v-if.md) | Exigir diretivas `v-if` válidas. | `essential` |  |
| <span id="vue-valid-v-is"></span>[`vue/valid-v-is`](./reference/vue-valid-v-is.md) | Exigir diretivas `v-is` válidas. | `essential` |  |
| <span id="vue-valid-v-memo"></span>[`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | Exigir diretivas `v-memo` válidas. | `essential` |  |
| <span id="vue-valid-v-model"></span>[`vue/valid-v-model`](./reference/vue-valid-v-model.md) | Exigir diretivas `v-model` válidas. | `essential` |  |
| <span id="vue-valid-v-on"></span>[`vue/valid-v-on`](./reference/vue-valid-v-on.md) | Exigir diretivas `v-on` válidas. | `essential` |  |
| <span id="vue-valid-v-once"></span>[`vue/valid-v-once`](./reference/vue-valid-v-once.md) | Exigir diretivas `v-once` válidas. | `essential` |  |
| <span id="vue-valid-v-pre"></span>[`vue/valid-v-pre`](./reference/vue-valid-v-pre.md) | Exigir diretivas `v-pre` válidas. | `essential` |  |
| <span id="vue-valid-v-show"></span>[`vue/valid-v-show`](./reference/vue-valid-v-show.md) | Exigir diretivas `v-show` válidas. | `essential` |  |
| <span id="vue-valid-v-slot"></span>[`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | Exigir diretivas `v-slot` válidas. | `essential` |  |
| <span id="vue-valid-v-text"></span>[`vue/valid-v-text`](./reference/vue-valid-v-text.md) | Exigir diretivas `v-text` válidas. | `essential` |  |
| <span id="vue-warn-custom-block"></span>[`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | Alertar sobre blocos personalizados em arquivos SFC. | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-directive"></span>[`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | Alertar sobre diretivas personalizadas que precisam de registro. | `nuxt`, `opinionated` |  |

## Regras de tipo e script (124)

| Regra | O que verifica | Predefinição | Correção automática |
| --- | --- | --- | --- |
| <span id="script-component-options-name-casing"></span>[`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | Exigir PascalCase na opção `name` do componente. | `opinionated`, `nuxt` |  |
| <span id="script-consistent-props-emits-key-quotes"></span>[`script/consistent-props-emits-key-quotes`](./reference/script-consistent-props-emits-key-quotes.md) | Uniformizar chaves entre aspas nas declarações de props e emits. | — |  |
| <span id="script-custom-event-name-casing"></span>[`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | Exigir a capitalização configurada nos nomes de eventos personalizados emitidos. | `opinionated`, `nuxt` |  |
| <span id="script-define-emits-declaration"></span>[`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | Exigir a forma de defineEmits&lt;{}&gt;() baseada em tipos em vez da forma em tempo de execução ou de array. | `opinionated`, `nuxt` |  |
| <span id="script-define-macros-order"></span>[`script/define-macros-order`](./reference/script-define-macros-order.md) | Exigir uma ordem consistente para as macros do compilador Vue em &lt;script setup&gt;. | `opinionated`, `nuxt` |  |
| <span id="script-define-props-declaration"></span>[`script/define-props-declaration`](./reference/script-define-props-declaration.md) | Exigir defineProps&lt;{ ... }&gt;() baseado em tipos em vez da forma com objeto em tempo de execução. | `opinionated`, `nuxt` |  |
| <span id="script-define-props-destructuring"></span>[`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | Exigir um estilo consistente de desestruturação de defineProps em &lt;script setup&gt;. | `happy-path` |  |
| <span id="script-no-arrow-functions-in-watch"></span>[`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | Proibir funções de seta como manipuladores de watch na Options API. | `essential` |  |
| <span id="script-no-async-in-computed"></span>[`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | Proibir funções assíncronas em propriedades computadas. | `essential` |  |
| <span id="script-no-boolean-default"></span>[`script/no-boolean-default`](./reference/script-no-boolean-default.md) | Proibir um valor padrão em uma prop Boolean. | `opinionated`, `nuxt` |  |
| <span id="script-no-computed-properties-in-data"></span>[`script/no-computed-properties-in-data`](./reference/script-no-computed-properties-in-data.md) | Proibir o uso de propriedades computadas dentro da função data do componente. | `essential` |  |
| <span id="script-no-deep-destructure-in-props"></span>[`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | Proibir desestruturação profundamente aninhada em defineProps. | `opinionated`, `nuxt` |  |
| <span id="script-no-deprecated-data-object-declaration"></span>[`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | Proibir um literal de objeto como opção data do componente (o Vue 3 exige uma função). | `essential` |  |
| <span id="script-no-deprecated-delete-set"></span>[`script/no-deprecated-delete-set`](./reference/script-no-deprecated-delete-set.md) | Proibir $set e $delete do Vue 2, removidos no Vue 3. | `essential` |  |
| <span id="script-no-deprecated-destroyed-lifecycle"></span>[`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | Proibir os hooks de ciclo de vida obsoletos destroyed e beforeDestroy. | `essential` | ✓ |
| <span id="script-no-deprecated-dollar-listeners-api"></span>[`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | Proibir a propriedade de instância $listeners removida no Vue 3 (incorporada a $attrs). | `essential` |  |
| <span id="script-no-deprecated-dollar-scopedslots-api"></span>[`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | Proibir a propriedade de instância $scopedSlots removida no Vue 3 (usar $slots). | `essential` |  |
| <span id="script-no-deprecated-events-api"></span>[`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | Proibir a API de eventos do Vue 2 removida ($on / $off / $once). | `essential` |  |
| <span id="script-no-deprecated-model-definition"></span>[`script/no-deprecated-model-definition`](./reference/script-no-deprecated-model-definition.md) | Proibir a opção model do Vue 2, removida no Vue 3. | `essential` |  |
| <span id="script-no-deprecated-props-default-this"></span>[`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | Proibir `this` dentro de uma função de valor padrão ou validação de prop (removido no Vue 3). | `essential` |  |
| <span id="script-no-deprecated-vue-config-keycodes"></span>[`script/no-deprecated-vue-config-keycodes`](./reference/script-no-deprecated-vue-config-keycodes.md) | Proibir Vue.config.keyCodes, removido no Vue 3. | `essential` |  |
| <span id="script-no-dupe-keys"></span>[`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | Proibir chaves duplicadas entre props/data/computed/methods/setup/inject da Options API. | `essential` |  |
| <span id="script-no-duplicate-attr-inheritance"></span>[`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | Sinalizar um componente que aplica duas vezes seus atributos repassados. | `essential` |  |
| <span id="script-no-export-in-script-setup"></span>[`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | Proibir instruções export dentro de &lt;script setup&gt;. | `essential` |  |
| <span id="script-no-expose-after-await"></span>[`script/no-expose-after-await`](./reference/script-no-expose-after-await.md) | Proibir expose() e defineExpose() após um await. | `essential` |  |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Proibir getCurrentInstance() no modo Vapor (retorna null). | `opinionated` |  |
| <span id="script-no-import-compiler-macros"></span>[`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | Proibir a importação de macros do compilador Vue que são importadas automaticamente. | `essential` |  |
| <span id="script-no-internal-imports"></span>[`script/no-internal-imports`](./reference/script-no-internal-imports.md) | Proibir importações de módulos internos do Vue. | `essential` |  |
| <span id="script-no-lifecycle-after-await"></span>[`script/no-lifecycle-after-await`](./reference/script-no-lifecycle-after-await.md) | Proibir hooks de ciclo de vida registrados após um await em setup(). | `essential` |  |
| <span id="script-no-multiple-slot-args"></span>[`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | Proibir passar mais de um argumento a uma chamada de função de slot com escopo. | `happy-path` |  |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Proibir o uso de nextTick() em componentes orientados a Vapor. | — |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Proibir padrões da Options API no modo Vapor. | `opinionated` |  |
| <span id="script-no-potential-component-option-typo"></span>[`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | Sinalizar prováveis erros de digitação nos nomes de opções de componentes da Options API. | `essential` |  |
| <span id="script-no-reactive-destructure"></span>[`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | Proibir a desestruturação de objetos reativos que causa perda de reatividade. | — |  |
| <span id="script-no-ref-as-operand"></span>[`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | Exigir que variáveis vinculadas a refs sejam acessadas por `.value` quando usadas como operandos. | `essential` |  |
| <span id="script-no-required-prop-with-default"></span>[`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | Proibir uma prop que tenha required: true e também um valor padrão. | `essential` |  |
| <span id="script-no-reserved-identifiers"></span>[`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | Proibir o uso de identificadores reservados pelo compilador Vue. | `essential` |  |
| <span id="script-no-reserved-keys"></span>[`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | Proibir nomes reservados pelo Vue como chaves de props/data/computed/methods/setup/inject na Options API. | `essential` |  |
| <span id="script-no-reserved-props"></span>[`script/no-reserved-props`](./reference/script-no-reserved-props.md) | Proibir nomes reservados na declaração de props de um componente. | `essential` |  |
| <span id="script-no-restricted-globals"></span>[`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | Proibir referências a variáveis globais do ambiente de execução que devem passar por um encapsulamento tipado. | — |  |
| <span id="script-no-restricted-members"></span>[`script/no-restricted-members`](./reference/script-no-restricted-members.md) | Proibir acessos a membros object.property configurados pelo projeto. | — |  |
| <span id="script-no-side-effects-in-computed-properties"></span>[`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | Proibir efeitos colaterais em getters computados da Options API. | `essential` |  |
| <span id="script-no-top-level-ref-in-script"></span>[`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | Proibir ref/reactive no nível superior para evitar contaminação de estado entre requisições. | — |  |
| <span id="script-no-unstable-nested-components"></span>[`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | Proibir definições de componentes dentro de funções de setup ou renderização. | `happy-path` |  |
| <span id="script-no-unused-emit-declarations"></span>[`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | Sinalizar eventos declarados que nunca são emitidos. | `opinionated`, `nuxt` |  |
| <span id="script-no-use-computed-property-like-method"></span>[`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | Proibir chamar uma propriedade computada da Options API como um método. | `essential` |  |
| <span id="script-no-watch-after-await"></span>[`script/no-watch-after-await`](./reference/script-no-watch-after-await.md) | Proibir watchers registrados após um await em setup() sem guardar a função de parada. | `essential` |  |
| <span id="script-no-with-defaults"></span>[`script/no-with-defaults`](./reference/script-no-with-defaults.md) | Desencorajar withDefaults em favor de valores padrão na desestruturação (Vue 3.5+). | `opinionated` |  |
| <span id="script-one-component-per-file"></span>[`script/one-component-per-file`](./reference/script-one-component-per-file.md) | Proibir mais de uma definição de componente em um arquivo. | `opinionated`, `nuxt` |  |
| <span id="script-order-in-components"></span>[`script/order-in-components`](./reference/script-order-in-components.md) | Exigir a ordem padrão das propriedades nas opções do componente. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-computed"></span>[`script/prefer-computed`](./reference/script-prefer-computed.md) | Preferir computed() para estado reativo derivado. | — |  |
| <span id="script-prefer-define-emits-binding"></span>[`script/prefer-define-emits-binding`](./reference/script-prefer-define-emits-binding.md) | Preferir a função capturada de defineEmits ao $emit do template. | — | ✓ |
| <span id="script-prefer-define-options"></span>[`script/prefer-define-options`](./reference/script-prefer-define-options.md) | Preferir defineOptions() a um &lt;script&gt; comum que só define name/inheritAttrs. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-import-from-vue"></span>[`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | Preferir importar de 'vue' em vez de pacotes internos. | `essential` | ✓ |
| <span id="script-prefer-ref-over-reactive"></span>[`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | Recomendar o uso de ref() em vez de reactive() para gerenciar estado. | — |  |
| <span id="script-prefer-use-attrs"></span>[`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | Recomendar o uso de useAttrs() em vez de context.attrs. | — |  |
| <span id="script-prefer-use-id"></span>[`script/prefer-use-id`](./reference/script-prefer-use-id.md) | Recomendar o uso de useId() para gerar IDs únicos (Vue 3.5+). | — |  |
| <span id="script-prefer-use-slots"></span>[`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | Recomendar o uso de useSlots() em vez de context.slots. | — |  |
| <span id="script-prefer-use-template-ref"></span>[`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | Recomendar useTemplateRef em vez de ref para referências de template (Vue 3.5+). | `opinionated`, `nuxt` |  |
| <span id="script-require-default-prop"></span>[`script/require-default-prop`](./reference/script-require-default-prop.md) | Exigir um valor padrão para toda prop opcional que não seja Boolean. | `opinionated`, `nuxt` |  |
| <span id="script-require-define-component"></span>[`script/require-define-component`](./reference/script-require-define-component.md) | Exigir defineComponent() em exports padrão de SFCs TypeScript. | `happy-path` |  |
| <span id="script-require-explicit-emits"></span>[`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | Exigir que os eventos emitidos sejam declarados em defineEmits ou na opção emits. | `happy-path` |  |
| <span id="script-require-explicit-slots"></span>[`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | Exigir que os slots consumidos por useSlots() sejam tipados explicitamente com defineSlots&lt;...&gt;(). | `opinionated`, `nuxt` |  |
| <span id="script-require-function-return-type"></span>[`script/require-function-return-type`](./reference/script-require-function-return-type.md) | Exigir anotações de tipo de retorno nas funções. | — |  |
| <span id="script-require-prop-type-constructor"></span>[`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | Exigir que os valores de `type` de props sejam construtores em vez de literais de string. | `essential` |  |
| <span id="script-require-prop-types"></span>[`script/require-prop-types`](./reference/script-require-prop-types.md) | Exigir que toda prop declare um tipo. | `opinionated` |  |
| <span id="script-require-render-return"></span>[`script/require-render-return`](./reference/script-require-render-return.md) | Exigir que a função render do componente devolva um valor. | `essential` |  |
| <span id="script-require-slots-as-functions"></span>[`script/require-slots-as-functions`](./reference/script-require-slots-as-functions.md) | Exigir que as propriedades de $slots sejam chamadas como funções. | `essential` |  |
| <span id="script-require-symbol-provide"></span>[`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | Recomendar o uso de Symbol como chave de injeção para provide/inject. | — |  |
| <span id="script-require-typed-object-prop"></span>[`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | Exigir um tipo explícito em uma prop cujo tipo de tempo de execução seja `Object` ou `Array`. | `opinionated`, `nuxt` |  |
| <span id="script-require-typed-ref"></span>[`script/require-typed-ref`](./reference/script-require-typed-ref.md) | Exigir um argumento de tipo explícito em um ref() inicializado sem valor, com null ou com undefined. | — |  |
| <span id="script-require-valid-default-prop"></span>[`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | Exigir que o valor padrão de uma prop seja válido para seu tipo declarado. | `essential` |  |
| <span id="script-return-in-computed-property"></span>[`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | Exigir um valor de retorno em todo getter computado. | `essential` |  |
| <span id="script-return-in-emits-validator"></span>[`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | Exigir um valor de retorno em todo validador de emits da Options API. | `essential` |  |
| <span id="script-valid-define-emits"></span>[`script/valid-define-emits`](./reference/script-valid-define-emits.md) | Exigir uso válido de defineEmits() (sem argumentos de tipo e de tempo de execução juntos, sem referências locais, uma única chamada). | `essential` |  |
| <span id="script-valid-define-options"></span>[`script/valid-define-options`](./reference/script-valid-define-options.md) | Exigir uso válido de defineOptions() (um único argumento de objeto, sem props/emits/expose/slots). | `essential` |  |
| <span id="script-valid-define-props"></span>[`script/valid-define-props`](./reference/script-valid-define-props.md) | Exigir uso válido de defineProps() (uma única chamada, sem argumentos de tipo e de tempo de execução juntos, sem referências locais). | `essential` |  |
| <span id="script-valid-next-tick"></span>[`script/valid-next-tick`](./reference/script-valid-next-tick.md) | Exigir que o resultado de uma chamada de nextTick() seja aguardado, encadeado ou receba uma função de retorno. | `essential` |  |
| <span id="type-await-thenable"></span>[`type/await-thenable`](./reference/type-await-thenable.md) | Proibir await de valores que não são thenable. | `nuxt`, `opinionated` |  |
| <span id="type-no-array-delete"></span>[`type/no-array-delete`](./reference/type-no-array-delete.md) | Proibir delete de elementos em valores semelhantes a array. | `nuxt`, `opinionated` |  |
| <span id="type-no-base-to-string"></span>[`type/no-base-to-string`](./reference/type-no-base-to-string.md) | Proibir a conversão em string de valores que recaem em Object.prototype.toString(). | `nuxt`, `opinionated` |  |
| <span id="type-no-confusing-void-expression"></span>[`type/no-confusing-void-expression`](./reference/type-no-confusing-void-expression.md) | Proibir expressões do tipo void em posições de valor confusas. | `nuxt`, `opinionated` |  |
| <span id="type-no-deprecated"></span>[`type/no-deprecated`](./reference/type-no-deprecated.md) | Proibir o uso de declarações obsoletas. | `nuxt`, `opinionated` |  |
| <span id="type-no-duplicate-type-constituents"></span>[`type/no-duplicate-type-constituents`](./reference/type-no-duplicate-type-constituents.md) | Proibir constituintes duplicados em uniões ou interseções. | `nuxt`, `opinionated` |  |
| <span id="type-no-floating-promises"></span>[`type/no-floating-promises`](./reference/type-no-floating-promises.md) | Proibir Promises soltas (não tratadas). | `nuxt`, `opinionated` |  |
| <span id="type-no-for-in-array"></span>[`type/no-for-in-array`](./reference/type-no-for-in-array.md) | Proibir for-in sobre valores semelhantes a array. | `nuxt`, `opinionated` |  |
| <span id="type-no-generated-empty-object-type"></span>[`type/no-generated-empty-object-type`](./reference/type-no-generated-empty-object-type.md) | Proibir tipos de objeto vazios gerados por operações de tipo. | `nuxt`, `opinionated` |  |
| <span id="type-no-implied-eval"></span>[`type/no-implied-eval`](./reference/type-no-implied-eval.md) | Proibir APIs de execução dinâmica baseadas em string. | `nuxt`, `opinionated` |  |
| <span id="type-no-meaningless-void-operator"></span>[`type/no-meaningless-void-operator`](./reference/type-no-meaningless-void-operator.md) | Proibir o operador void em expressões que já são void. | `nuxt`, `opinionated` |  |
| <span id="type-no-misused-promises"></span>[`type/no-misused-promises`](./reference/type-no-misused-promises.md) | Proibir Promises em lugares que não foram feitos para tratá-las. | `nuxt`, `opinionated` |  |
| <span id="type-no-misused-spread"></span>[`type/no-misused-spread`](./reference/type-no-misused-spread.md) | Proibir espalhar valores em posições incompatíveis. | `nuxt`, `opinionated` |  |
| <span id="type-no-mixed-enums"></span>[`type/no-mixed-enums`](./reference/type-no-mixed-enums.md) | Proibir a mistura de membros string e numéricos num enum. | `nuxt`, `opinionated` |  |
| <span id="type-no-reactivity-loss"></span>[`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | Proibir cópias estáticas simples de valores reativos em atribuições e chamadas. | `nuxt`, `opinionated` |  |
| <span id="type-no-redundant-type-constituents"></span>[`type/no-redundant-type-constituents`](./reference/type-no-redundant-type-constituents.md) | Proibir membros de uniões e interseções que não acrescentam ou substituem informação de tipo. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-boolean-literal-compare"></span>[`type/no-unnecessary-boolean-literal-compare`](./reference/type-no-unnecessary-boolean-literal-compare.md) | Proibir comparações desnecessárias com literais booleanos. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-condition"></span>[`type/no-unnecessary-condition`](./reference/type-no-unnecessary-condition.md) | Proibir condições que são sempre verdadeiras, sempre falsas, never ou sempre nullish. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-template-expression"></span>[`type/no-unnecessary-template-expression`](./reference/type-no-unnecessary-template-expression.md) | Proibir template literals que envolvem uma única expressão sem necessidade. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-arguments"></span>[`type/no-unnecessary-type-arguments`](./reference/type-no-unnecessary-type-arguments.md) | Proibir argumentos de tipo iguais ao valor padrão. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-assertion"></span>[`type/no-unnecessary-type-assertion`](./reference/type-no-unnecessary-type-assertion.md) | Proibir asserções de tipo que não mudam o tipo da expressão. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-conversion"></span>[`type/no-unnecessary-type-conversion`](./reference/type-no-unnecessary-type-conversion.md) | Proibir conversões que não mudam o tipo nem o valor da expressão. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-parameters"></span>[`type/no-unnecessary-type-parameters`](./reference/type-no-unnecessary-type-parameters.md) | Proibir parâmetros de tipo usados uma única vez na assinatura. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-argument"></span>[`type/no-unsafe-argument`](./reference/type-no-unsafe-argument.md) | Proibir chamar uma função com um valor do tipo `any`. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-assignment"></span>[`type/no-unsafe-assignment`](./reference/type-no-unsafe-assignment.md) | Proibir atribuir valores any a alvos mais específicos. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-call"></span>[`type/no-unsafe-call`](./reference/type-no-unsafe-call.md) | Proibir chamar valores tipados como any. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-enum-assignment"></span>[`type/no-unsafe-enum-assignment`](./reference/type-no-unsafe-enum-assignment.md) | Proibir atribuir um valor que não é do enum a um enum. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-enum-comparison"></span>[`type/no-unsafe-enum-comparison`](./reference/type-no-unsafe-enum-comparison.md) | Proibir comparar um valor de enum com um valor de outro domínio. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-member-access"></span>[`type/no-unsafe-member-access`](./reference/type-no-unsafe-member-access.md) | Proibir acesso a membros de valores tipados como any. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-return"></span>[`type/no-unsafe-return`](./reference/type-no-unsafe-return.md) | Proibir devolver valores any a partir de funções. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-template-binding"></span>[`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | Proibir vinculações de template que resultam em tipos inseguros. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-unary-minus"></span>[`type/no-unsafe-unary-minus`](./reference/type-no-unsafe-unary-minus.md) | Proibir a negação unária em valores que não são number nem bigint. | `nuxt`, `opinionated` |  |
| <span id="type-only-throw-error"></span>[`type/only-throw-error`](./reference/type-only-throw-error.md) | Exigir que os valores lançados sejam semelhantes a Error. | `nuxt`, `opinionated` |  |
| <span id="type-prefer-promise-reject-errors"></span>[`type/prefer-promise-reject-errors`](./reference/type-prefer-promise-reject-errors.md) | Exigir que o motivo de rejeição de uma Promise seja semelhante a Error. | `nuxt`, `opinionated` |  |
| <span id="type-prefer-reduce-type-parameter"></span>[`type/prefer-reduce-type-parameter`](./reference/type-prefer-reduce-type-parameter.md) | Preferir o parâmetro de tipo de Array#reduce a uma asserção do valor inicial. | `nuxt`, `opinionated` |  |
| <span id="type-prefer-return-this-type"></span>[`type/prefer-return-this-type`](./reference/type-prefer-return-this-type.md) | Exigir `this` quando o método devolve apenas o tipo `this`. | `nuxt`, `opinionated` |  |
| <span id="type-related-getter-setter-pairs"></span>[`type/related-getter-setter-pairs`](./reference/type-related-getter-setter-pairs.md) | Exigir que getters e setters emparelhados usem tipos compatíveis. | `nuxt`, `opinionated` |  |
| <span id="type-require-await"></span>[`type/require-await`](./reference/type-require-await.md) | Proibir funções async que não contêm uma expressão await. | `nuxt`, `opinionated` |  |
| <span id="type-require-typed-emits"></span>[`type/require-typed-emits`](./reference/type-require-typed-emits.md) | Exigir uma definição de tipo para defineEmits. | `happy-path` |  |
| <span id="type-require-typed-props"></span>[`type/require-typed-props`](./reference/type-require-typed-props.md) | Exigir uma definição de tipo para defineProps. | `happy-path` |  |
| <span id="type-restrict-plus-operands"></span>[`type/restrict-plus-operands`](./reference/type-restrict-plus-operands.md) | Exigir que os operandos de + sejam explicitamente compatíveis. | `nuxt`, `opinionated` |  |
| <span id="type-restrict-template-expressions"></span>[`type/restrict-template-expressions`](./reference/type-restrict-template-expressions.md) | Exigir que as expressões de um template literal sejam compatíveis com string. | `nuxt`, `opinionated` |  |
| <span id="type-return-await"></span>[`type/return-await`](./reference/type-return-await.md) | Exigir o uso consistente de `return await` conforme o contexto e a configuração. | `nuxt`, `opinionated` |  |
| <span id="type-strict-boolean-expressions"></span>[`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | Exigir expressões booleanas seguras nas condições de script e template. | — |  |
| <span id="type-unbound-method"></span>[`type/unbound-method`](./reference/type-unbound-method.md) | Proibir referências a métodos sem o seu recetor (`this`). | `nuxt`, `opinionated` |  |
| <span id="type-use-unknown-in-catch-callback-variable"></span>[`type/use-unknown-in-catch-callback-variable`](./reference/type-use-unknown-in-catch-callback-variable.md) | Exigir que a variável do callback de catch de uma Promise tenha uma anotação unknown explícita. | `nuxt`, `opinionated` |  |

## Regras HTML (9)

| Regra | O que verifica | Predefinição | Correção automática |
| --- | --- | --- | --- |
| <span id="html-deprecated-attr"></span>[`html/deprecated-attr`](./reference/html-deprecated-attr.md) | Proibir atributos HTML obsoletos. | `happy-path` |  |
| <span id="html-deprecated-element"></span>[`html/deprecated-element`](./reference/html-deprecated-element.md) | Proibir elementos HTML obsoletos. | `happy-path` |  |
| <span id="html-id-duplication"></span>[`html/id-duplication`](./reference/html-id-duplication.md) | Proibir IDs de elementos duplicados. | `essential` |  |
| <span id="html-no-consecutive-br"></span>[`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | Proibir elementos &lt;br&gt; consecutivos. | `happy-path` |  |
| <span id="html-no-dupe-style-properties"></span>[`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | Proibir propriedades duplicadas em atributos de estilo em linha. | `nuxt`, `opinionated` |  |
| <span id="html-no-duplicate-class"></span>[`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | Proibir nomes de classe duplicados em um atributo class estático. | `happy-path` |  |
| <span id="html-no-duplicate-dt"></span>[`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | Proibir nomes &lt;dt&gt; duplicados em &lt;dl&gt;. | `happy-path` |  |
| <span id="html-no-empty-palpable-content"></span>[`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | Proibir elementos vazios que esperam conteúdo visível. | `happy-path` |  |
| <span id="html-require-datetime"></span>[`html/require-datetime`](./reference/html-require-datetime.md) | Exigir o atributo datetime no elemento &lt;time&gt;. | `happy-path` |  |

## Regras de acessibilidade (31)

| Regra | O que verifica | Predefinição | Correção automática |
| --- | --- | --- | --- |
| <span id="a11y-alt-text"></span>[`a11y/alt-text`](./reference/a11y-alt-text.md) | Exigir texto alternativo para elementos de mídia. | `happy-path` |  |
| <span id="a11y-anchor-has-content"></span>[`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | Exigir conteúdo acessível nos elementos de âncora. | `happy-path` |  |
| <span id="a11y-anchor-is-valid"></span>[`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | Exigir um href válido nos elementos de âncora. | `happy-path` |  |
| <span id="a11y-aria-props"></span>[`a11y/aria-props`](./reference/a11y-aria-props.md) | Proibir atributos ARIA inválidos. | `happy-path` |  |
| <span id="a11y-aria-role"></span>[`a11y/aria-role`](./reference/a11y-aria-role.md) | Exigir que elementos com papéis ARIA usem um papel ARIA válido e não abstrato. | `happy-path` |  |
| <span id="a11y-aria-unsupported-elements"></span>[`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | Proibir atributos ARIA em elementos que não os aceitam. | `happy-path` |  |
| <span id="a11y-click-events-have-key-events"></span>[`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | Exigir manipuladores de eventos de teclado junto aos eventos de clique. | `happy-path` |  |
| <span id="a11y-form-control-has-label"></span>[`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | Exigir rótulos associados aos controles de formulário. | `happy-path` |  |
| <span id="a11y-heading-has-content"></span>[`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | Exigir conteúdo acessível nos elementos de título. | `happy-path` |  |
| <span id="a11y-heading-levels"></span>[`a11y/heading-levels`](./reference/a11y-heading-levels.md) | Proibir que níveis de título sejam pulados. | `nuxt`, `opinionated` |  |
| <span id="a11y-iframe-has-title"></span>[`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | Exigir um atributo title nos elementos iframe. | `happy-path` |  |
| <span id="a11y-img-alt"></span>[`a11y/img-alt`](./reference/a11y-img-alt.md) | Exigir o atributo alt nas imagens para garantir acessibilidade. | — |  |
| <span id="a11y-interactive-supports-focus"></span>[`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | Exigir que elementos com papéis interativos possam receber foco. | `happy-path` |  |
| <span id="a11y-label-has-for"></span>[`a11y/label-has-for`](./reference/a11y-label-has-for.md) | Exigir controles de formulário associados aos rótulos. | `happy-path` |  |
| <span id="a11y-landmark-roles"></span>[`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | Validar a posição e a unicidade dos papéis de regiões de referência. | `nuxt`, `opinionated` |  |
| <span id="a11y-media-has-caption"></span>[`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | Exigir legendas nos elementos de mídia. | `happy-path` |  |
| <span id="a11y-mouse-events-have-key-events"></span>[`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | Exigir eventos de foco e perda de foco junto aos eventos de mouse. | `happy-path` |  |
| <span id="a11y-no-access-key"></span>[`a11y/no-access-key`](./reference/a11y-no-access-key.md) | Proibir o uso do atributo accesskey. | `happy-path` |  |
| <span id="a11y-no-aria-hidden-on-focusable"></span>[`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | Proibir aria-hidden="true" em elementos que podem receber foco. | `happy-path` |  |
| <span id="a11y-no-autofocus"></span>[`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | Proibir o uso do atributo autofocus. | `happy-path` |  |
| <span id="a11y-no-distracting-elements"></span>[`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | Proibir elementos que causam distração, como &lt;marquee&gt; e &lt;blink&gt;. | `happy-path` |  |
| <span id="a11y-no-i-for-icon"></span>[`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | Proibir o uso do elemento &lt;i&gt; para ícones. | `happy-path` |  |
| <span id="a11y-no-redundant-roles"></span>[`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | Proibir papéis ARIA redundantes. | `happy-path` | ✓ |
| <span id="a11y-no-refer-to-non-existent-id"></span>[`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | Proibir referências a IDs inexistentes. | `happy-path` |  |
| <span id="a11y-no-role-presentation-on-focusable"></span>[`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | Proibir role="presentation" ou role="none" em elementos que podem receber foco. | `happy-path` |  |
| <span id="a11y-no-static-element-interactions"></span>[`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | Proibir manipuladores de eventos em elementos estáticos. | `happy-path` |  |
| <span id="a11y-placeholder-label-option"></span>[`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | Exigir uma opção de orientação em um select required. | `nuxt`, `opinionated` |  |
| <span id="a11y-role-has-required-aria-props"></span>[`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | Exigir as propriedades obrigatórias dos papéis ARIA. | `happy-path` |  |
| <span id="a11y-role-img-has-name"></span>[`a11y/role-img-has-name`](./reference/a11y-role-img-has-name.md) | Exigir um nome acessível nos elementos com role="img". | — |  |
| <span id="a11y-tabindex-no-positive"></span>[`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | Proibir valores positivos de tabindex. | `happy-path` |  |
| <span id="a11y-use-list"></span>[`a11y/use-list`](./reference/a11y-use-list.md) | Sugerir elementos de lista para textos que parecem itens de lista. | `nuxt`, `opinionated` |  |

## Regras de SSR (2)

| Regra | O que verifica | Predefinição | Correção automática |
| --- | --- | --- | --- |
| <span id="ssr-no-browser-globals-in-ssr"></span>[`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | Proibir variáveis globais exclusivas do navegador no contexto de SSR. | `happy-path` |  |
| <span id="ssr-no-hydration-mismatch"></span>[`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | Proibir valores não determinísticos que causam divergências na hidratação. | `happy-path` |  |

## Regras de Vapor (4)

| Regra | O que verifica | Predefinição | Correção automática |
| --- | --- | --- | --- |
| <span id="vapor-no-inline-template"></span>[`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | Proibir o atributo obsoleto inline-template. | `nuxt`, `opinionated` |  |
| <span id="vapor-no-vue-lifecycle-events"></span>[`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | Proibir eventos de ciclo de vida @vue:xxx por elemento (não suportados em Vapor). | `happy-path` |  |
| <span id="vapor-prefer-static-class"></span>[`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | Preferir class estática a uma vinculação dinâmica de class para literais de string. | `nuxt`, `opinionated` | ✓ |
| <span id="vapor-require-vapor-attribute"></span>[`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | Sugerir a adição do atributo vapor a script setup. | `nuxt`, `opinionated` |  |

## Regras do ecossistema e Nuxt (13)

| Regra | O que verifica | Predefinição | Correção automática |
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

## Regras CSS (10)

| Regra | O que verifica | Predefinição | Correção automática |
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

## Regras Musea (6)

| Regra | O que verifica | Predefinição | Correção automática |
| --- | --- | --- | --- |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | Proibir blocos &lt;variant&gt; vazios. | — |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | Preferir variáveis CSS de tokens de design a valores primitivos fixos escritos diretamente. | — |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | Exigir o atributo component no bloco &lt;art&gt;. | — |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | Exigir o atributo title no bloco &lt;art&gt;. | — |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | Exigir nomes de variante únicos. | — |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | Exigir o atributo name nos blocos &lt;variant&gt;. | — |  |

## Regras de petite-vue (3)

| Regra | O que verifica | Predefinição | Correção automática |
| --- | --- | --- | --- |
| <span id="petite-vue-no-unsupported-directive"></span>[`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | Proibir diretivas que o petite-vue não aceita. | — |  |
| <span id="petite-vue-valid-v-effect"></span>[`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | Exigir uma expressão não vazia em v-effect. | — |  |
| <span id="petite-vue-valid-v-scope"></span>[`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | Exigir que v-scope vincule um objeto literal. | — |  |

## Regras entre arquivos (66)

Ative estas verificações com `crossFile: true` em `linter`, passado a `withVue`, ou execute `vize lint --cross-file`. Os diagnósticos aparecem como `vize:croquis/cf/<name>`; configure-os em `linter.rules` como `croquis/cf/<name>`, sem `vize:`.

| Regra | O que verifica | Status |
| --- | --- | --- |
| <span id="ecosystem-vue-router-extra-param"></span>[`ecosystem/vue-router-extra-param`](./project/ecosystem-vue-router-extra-param.md) | Relata parâmetros de navegação que a rota de destino não declara. | Reportada por `vp run lint` |
| <span id="ecosystem-vue-router-missing-param"></span>[`ecosystem/vue-router-missing-param`](./project/ecosystem-vue-router-missing-param.md) | Relata uma navegação nomeada que omite um parâmetro obrigatório da rota. | Reportada por `vp run lint` |
| <span id="ecosystem-vue-router-param-type"></span>[`ecosystem/vue-router-param-type`](./project/ecosystem-vue-router-param-type.md) | Relata um valor de parâmetro de rota que o segmento declarado não aceita. | Reportada por `vp run lint` |
| <span id="ecosystem-vue-router-unknown-route"></span>[`ecosystem/vue-router-unknown-route`](./project/ecosystem-vue-router-unknown-route.md) | Relata um nome de navegação que o roteador instalado não declara. | Reportada por `vp run lint` |
| <span id="html-cross-component-nesting"></span>[`html/cross-component-nesting`](./project/html-cross-component-nesting.md) | Verificar o aninhamento HTML real após compor os componentes importados. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-array-mutation"></span>[`vize:croquis/cf/array-mutation`](./project/vize-croquis-cf-array-mutation.md) | Use operações observáveis para atualizar índices de arrays no Vue 2.7. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-async-boundary"></span>[`vize:croquis/cf/async-boundary`](./project/vize-croquis-cf-async-boundary.md) | O estado reativo atravessa um limite assíncrono e pode ser observado desatualizado. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-async-no-suspense"></span>[`vize:croquis/cf/async-no-suspense`](./project/vize-croquis-cf-async-no-suspense.md) | Um componente assíncrono é renderizado sem um limite Suspense. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-browser-api-ssr"></span>[`vize:croquis/cf/browser-api-ssr`](./project/vize-croquis-cf-browser-api-ssr.md) | Uma API exclusiva do navegador é usada onde o componente pode ser renderizado no servidor. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-circular-dep"></span>[`vize:croquis/cf/circular-dep`](./project/vize-croquis-cf-circular-dep.md) | Os componentes importam uns aos outros em um ciclo. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-circular-reactive-dependency"></span>[`vize:croquis/cf/circular-reactive-dependency`](./project/vize-croquis-cf-circular-reactive-dependency.md) | Os cálculos reativos dependem uns dos outros em um ciclo. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-closure-captures-reactive"></span>[`vize:croquis/cf/closure-captures-reactive`](./project/vize-croquis-cf-closure-captures-reactive.md) | Um fechamento captura um valor reativo e não verá atualizações posteriores. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-composable-outside-setup"></span>[`vize:croquis/cf/composable-outside-setup`](./project/vize-croquis-cf-composable-outside-setup.md) | Uma função de composição é chamada fora de `setup`. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-computed-side-effects"></span>[`vize:croquis/cf/computed-side-effects`](./project/vize-croquis-cf-computed-side-effects.md) | Uma função de leitura computada escreve no estado ou produz outro efeito colateral. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-deep-import"></span>[`vize:croquis/cf/deep-import`](./project/vize-croquis-cf-deep-import.md) | Uma cadeia de importações é mais profunda do que o projeto permite. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-destructuring-breaks-reactivity"></span>[`vize:croquis/cf/destructuring-breaks-reactivity`](./project/vize-croquis-cf-destructuring-breaks-reactivity.md) | Desestruturar um objeto reativo copia os campos e perde o rastreamento. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-di-outside-setup"></span>[`vize:croquis/cf/di-outside-setup`](./project/vize-croquis-cf-di-outside-setup.md) | `provide` ou `inject` é chamado fora de `setup`. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-dom-access-without-next-tick"></span>[`vize:croquis/cf/dom-access-without-next-tick`](./project/vize-croquis-cf-dom-access-without-next-tick.md) | O DOM é lido antes de o Vue aplicar a atualização. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-duplicate-id"></span>[`vize:croquis/cf/duplicate-id`](./project/vize-croquis-cf-duplicate-id.md) | O mesmo id de elemento é usado em mais de um componente. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-event-listener-leak"></span>[`vize:croquis/cf/event-listener-leak`](./project/vize-croquis-cf-event-listener-leak.md) | Um ouvinte de eventos é registrado e nunca removido. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-event-modifier"></span>[`vize:croquis/cf/event-modifier`](./project/vize-croquis-cf-event-modifier.md) | Um ouvinte de eventos usa um modificador que o evento emitido não suporta. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-hydration-risk"></span>[`vize:croquis/cf/hydration-risk`](./project/vize-croquis-cf-hydration-risk.md) | A renderização pode divergir entre o servidor e o cliente. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-inherit-attrs-unused"></span>[`vize:croquis/cf/inherit-attrs-unused`](./project/vize-croquis-cf-inherit-attrs-unused.md) | `inheritAttrs: false` está definido e o componente nunca lê os atributos. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-inject-without-symbol"></span>[`vize:croquis/cf/inject-without-symbol`](./project/vize-croquis-cf-inject-without-symbol.md) | `inject` usa uma chave comum em vez de um símbolo `InjectionKey`. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-injected-async-mutation-race"></span>[`vize:croquis/cf/injected-async-mutation-race`](./project/vize-croquis-cf-injected-async-mutation-race.md) | Um valor injetado é alterado por uma tarefa assíncrona sujeita a uma condição de corrida. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-lifecycle-outside-setup"></span>[`vize:croquis/cf/lifecycle-outside-setup`](./project/vize-croquis-cf-lifecycle-outside-setup.md) | Um gancho de ciclo de vida é registrado fora de `setup`. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-lifecycle-without-cleanup"></span>[`vize:croquis/cf/lifecycle-without-cleanup`](./project/vize-croquis-cf-lifecycle-without-cleanup.md) | Um gancho de ciclo de vida inicia um trabalho e nunca faz sua limpeza. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-missing-required-prop"></span>[`vize:croquis/cf/missing-required-prop`](./project/vize-croquis-cf-missing-required-prop.md) | Uma prop obrigatória não é passada. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-missing-suspense"></span>[`vize:croquis/cf/missing-suspense`](./project/vize-croquis-cf-missing-suspense.md) | Uma dependência assíncrona é usada fora de um limite Suspense. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-module-scope-reactive"></span>[`vize:croquis/cf/module-scope-reactive`](./project/vize-croquis-cf-module-scope-reactive.md) | O estado reativo é criado no escopo do módulo e compartilhado por todos os chamadores. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-multi-root-attrs"></span>[`vize:croquis/cf/multi-root-attrs`](./project/vize-croquis-cf-multi-root-attrs.md) | Um componente com múltiplas raízes recebe atributos e não tem onde colocá-los. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-mutated-after-escape"></span>[`vize:croquis/cf/mutated-after-escape`](./project/vize-croquis-cf-mutated-after-escape.md) | Um objeto reativo é alterado depois de escapar de seu proprietário. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-non-reactive-provide"></span>[`vize:croquis/cf/non-reactive-provide`](./project/vize-croquis-cf-non-reactive-provide.md) | Um valor fornecido não é reativo, portanto os descendentes não verão atualizações. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-non-unique-id"></span>[`vize:croquis/cf/non-unique-id`](./project/vize-croquis-cf-non-unique-id.md) | Um id de elemento dentro de um laço não é único por item. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-object-identity-comparison"></span>[`vize:croquis/cf/object-identity-comparison`](./project/vize-croquis-cf-object-identity-comparison.md) | Compare IDs estáveis de registros entre objetos brutos e proxies reativos. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-pinia-getter"></span>[`vize:croquis/cf/pinia-getter`](./project/vize-croquis-cf-pinia-getter.md) | Evite copiar valores de getters do Pinia como instantâneos durante setup. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-prop-type-mismatch"></span>[`vize:croquis/cf/prop-type-mismatch`](./project/vize-croquis-cf-prop-type-mismatch.md) | O valor de uma prop passada não corresponde ao tipo declarado. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-provide-inject-type"></span>[`vize:croquis/cf/provide-inject-type`](./project/vize-croquis-cf-provide-inject-type.md) | Um valor fornecido e sua injeção não têm o mesmo tipo. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-provide-without-symbol"></span>[`vize:croquis/cf/provide-without-symbol`](./project/vize-croquis-cf-provide-without-symbol.md) | `provide` usa uma chave comum em vez de um símbolo `InjectionKey`. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-reactive-export"></span>[`vize:croquis/cf/reactive-export`](./project/vize-croquis-cf-reactive-export.md) | O estado reativo é exportado pelo módulo. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-reactivity-outside-setup"></span>[`vize:croquis/cf/reactivity-outside-setup`](./project/vize-croquis-cf-reactivity-outside-setup.md) | Uma API reativa é chamada fora de `setup`. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-reassignment-breaks-reactivity"></span>[`vize:croquis/cf/reassignment-breaks-reactivity`](./project/vize-croquis-cf-reassignment-breaks-reactivity.md) | Reatribuir uma variável reativa a substitui por um valor simples. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-reference-escapes-scope"></span>[`vize:croquis/cf/reference-escapes-scope`](./project/vize-croquis-cf-reference-escapes-scope.md) | Uma referência reativa escapa do escopo responsável por seu ciclo de vida. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-setup-context-violation"></span>[`vize:croquis/cf/setup-context-violation`](./project/vize-croquis-cf-setup-context-violation.md) | Crie o estado reativo pertencente a cada componente no setup de sua instância. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-shallow-deep-access"></span>[`vize:croquis/cf/shallow-deep-access`](./project/vize-croquis-cf-shallow-deep-access.md) | Uma propriedade profunda de um valor `shallowReactive` ou `shallowRef` é lida como se fosse rastreada. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-spread-breaks-reactivity"></span>[`vize:croquis/cf/spread-breaks-reactivity`](./project/vize-croquis-cf-spread-breaks-reactivity.md) | Espalhar um objeto reativo copia seus valores e perde o rastreamento. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-suspense-no-fallback"></span>[`vize:croquis/cf/suspense-no-fallback`](./project/vize-croquis-cf-suspense-no-fallback.md) | `<Suspense>` não tem conteúdo alternativo. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-template-ref-timing"></span>[`vize:croquis/cf/template-ref-timing`](./project/vize-croquis-cf-template-ref-timing.md) | Uma ref de template é lida antes da montagem do componente. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-toraw-mutation"></span>[`vize:croquis/cf/toraw-mutation`](./project/vize-croquis-cf-toraw-mutation.md) | `toRaw` é usado e o objeto bruto é alterado em seguida. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-uncaught-error"></span>[`vize:croquis/cf/uncaught-error`](./project/vize-croquis-cf-uncaught-error.md) | Um componente pode lançar um erro e nenhum limite de erro o captura. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-undeclared-emit"></span>[`vize:croquis/cf/undeclared-emit`](./project/vize-croquis-cf-undeclared-emit.md) | O componente emite um evento que não foi declarado. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-undeclared-prop"></span>[`vize:croquis/cf/undeclared-prop`](./project/vize-croquis-cf-undeclared-prop.md) | Um pai passa uma prop que o filho não declara. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-undefined-slot"></span>[`vize:croquis/cf/undefined-slot`](./project/vize-croquis-cf-undefined-slot.md) | Um pai preenche um slot que o filho não expõe. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-unhandled-event"></span>[`vize:croquis/cf/unhandled-event`](./project/vize-croquis-cf-unhandled-event.md) | Um filho emite um evento que nenhum pai trata. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-unmatched-inject"></span>[`vize:croquis/cf/unmatched-inject`](./project/vize-croquis-cf-unmatched-inject.md) | `inject` nomeia uma chave que nenhum ancestral fornece. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-unmatched-listener"></span>[`vize:croquis/cf/unmatched-listener`](./project/vize-croquis-cf-unmatched-listener.md) | Um pai escuta um evento que o filho não emite. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-unregistered-component"></span>[`vize:croquis/cf/unregistered-component`](./project/vize-croquis-cf-unregistered-component.md) | Um template usa um componente que não foi registrado nem importado. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-unresolved-import"></span>[`vize:croquis/cf/unresolved-import`](./project/vize-croquis-cf-unresolved-import.md) | Uma importação não resolve para um módulo. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-unused-attrs"></span>[`vize:croquis/cf/unused-attrs`](./project/vize-croquis-cf-unused-attrs.md) | Atributos de herança automática são passados para um componente com múltiplas raízes que não os usa. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-unused-emit"></span>[`vize:croquis/cf/unused-emit`](./project/vize-croquis-cf-unused-emit.md) | Um evento declarado nunca é emitido. | Experimental; a CLI ainda não reporta |
| <span id="vize-croquis-cf-unused-provide"></span>[`vize:croquis/cf/unused-provide`](./project/vize-croquis-cf-unused-provide.md) | Uma chave fornecida nunca é injetada. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-value-extraction-breaks-reactivity"></span>[`vize:croquis/cf/value-extraction-breaks-reactivity`](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md) | Ler um valor reativo para uma variável local perde as atualizações posteriores. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-watch-can-be-computed"></span>[`vize:croquis/cf/watch-can-be-computed`](./project/vize-croquis-cf-watch-can-be-computed.md) | Um observador apenas copia um valor para o estado e pode ser um valor computado. | Planejada; ainda não é reportada |
| <span id="vize-croquis-cf-watcheffect-async"></span>[`vize:croquis/cf/watcheffect-async`](./project/vize-croquis-cf-watcheffect-async.md) | Proteja o trabalho assíncrono de watchEffect contra invalidação e respostas desatualizadas. | Reportada por `vp run lint` |
| <span id="vize-croquis-cf-watcher-outside-setup"></span>[`vize:croquis/cf/watcher-outside-setup`](./project/vize-croquis-cf-watcher-outside-setup.md) | `watch` ou `watchEffect` é chamado fora de `setup`. | Planejada; ainda não é reportada |
| <span id="vue-cross-file-attrs-fallthrough"></span>[`vue/cross-file-attrs-fallthrough`](./project/vue-cross-file-attrs-fallthrough.md) | Um pai passa atributos para um filho resolvido cuja raiz não pode herdá-los e que não usa $attrs explicitamente. | Reportada por `vp run lint` |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
