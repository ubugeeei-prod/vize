---
title: Regras Vue
---

# Regras Vue

Cada regra Vue reúne nesta página sua finalidade, configuração e exemplos incorreto e correto. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo.

<span id="regras-do-vue"></span>
<span id="regras-de-sintaxe-e-estilo"></span>

| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [Incorreto](#vue-a11y-img-alt-bad) · [Correto](#vue-a11y-img-alt-good) | Exigir o atributo alt nas imagens para garantir acessibilidade |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [Incorreto](#vue-attribute-hyphenation-bad) · [Correto](#vue-attribute-hyphenation-good) | Aplicar um padrão de nomes de atributos em componentes personalizados |
| [`vue/attribute-order`](#vue-attribute-order) | [Incorreto](#vue-attribute-order-bad) · [Correto](#vue-attribute-order-good) | Aplicar uma ordem consistente aos atributos |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [Incorreto](#vue-component-definition-name-casing-bad) · [Correto](#vue-component-definition-name-casing-good) | Exigir PascalCase ou kebab-case nos nomes de definição de componentes |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [Incorreto](#vue-component-name-in-template-casing-bad) · [Correto](#vue-component-name-in-template-casing-good) | Aplicar um padrão específico de maiúsculas e minúsculas aos nomes de componentes nos templates |
| [`vue/html-button-has-type`](#vue-html-button-has-type) | [Incorreto](#vue-html-button-has-type-bad) · [Correto](#vue-html-button-has-type-good) | Exigir um type explícito e válido nos elementos button |
| [`vue/html-quotes`](#vue-html-quotes) | [Incorreto](#vue-html-quotes-bad) · [Correto](#vue-html-quotes-good) | Aplicar um padrão de aspas aos atributos HTML |
| [`vue/html-self-closing`](#vue-html-self-closing) | [Incorreto](#vue-html-self-closing-bad) · [Correto](#vue-html-self-closing-good) | Aplicar um padrão de tags com fechamento automático |
| [`vue/max-template-complexity`](#vue-max-template-complexity) | [Incorreto](#vue-max-template-complexity-bad) · [Correto](#vue-max-template-complexity-good) | Limitar a complexidade do próprio template de um componente, tanto ciclomática quanto cognitiva |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [Incorreto](#vue-multi-word-component-names-bad) · [Correto](#vue-multi-word-component-names-good) | Exigir nomes de componentes com mais de uma palavra |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [Incorreto](#vue-mustache-interpolation-spacing-bad) · [Correto](#vue-mustache-interpolation-spacing-good) | Aplicar espaçamento consistente dentro das interpolações com chaves duplas |
| [`vue/no-array-index-key`](#vue-no-array-index-key) | [Incorreto](#vue-no-array-index-key-bad) · [Correto](#vue-no-array-index-key-good) | Proibir o uso direto da variável de índice de v-for como :key |
| [`vue/no-bare-strings-in-template`](#vue-no-bare-strings-in-template) | [Incorreto](#vue-no-bare-strings-in-template-bad) · [Correto](#vue-no-bare-strings-in-template-good) | Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [Incorreto](#vue-no-boolean-attr-value-bad) · [Correto](#vue-no-boolean-attr-value-good) | Proibir valores explícitos em atributos HTML booleanos |
| [`vue/no-child-content`](#vue-no-child-content) | [Incorreto](#vue-no-child-content-bad) · [Correto](#vue-no-child-content-good) | Proibir conteúdo filho ao usar v-html ou v-text |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [Incorreto](#vue-no-deprecated-filter-bad) · [Correto](#vue-no-deprecated-filter-good) | Proibir a sintaxe obsoleta de filtros do Vue 2 com o operador de barra vertical |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [Incorreto](#vue-no-deprecated-functional-template-bad) · [Correto](#vue-no-deprecated-functional-template-good) | Proibir o atributo `functional` no `<template>` de um SFC |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [Incorreto](#vue-no-deprecated-html-element-is-bad) · [Correto](#vue-no-deprecated-html-element-is-good) | Proibir o atributo `is` em elementos HTML nativos |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [Incorreto](#vue-no-deprecated-inline-template-bad) · [Correto](#vue-no-deprecated-inline-template-good) | Proibir o atributo obsoleto `inline-template` |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [Incorreto](#vue-no-deprecated-router-link-tag-prop-bad) · [Correto](#vue-no-deprecated-router-link-tag-prop-good) | Proibir a prop `tag` em &lt;router-link&gt; |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [Incorreto](#vue-no-deprecated-scope-attribute-bad) · [Correto](#vue-no-deprecated-scope-attribute-good) | Proibir o atributo obsoleto `scope` em &lt;template&gt; |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [Incorreto](#vue-no-deprecated-slot-attribute-bad) · [Correto](#vue-no-deprecated-slot-attribute-good) | Proibir o atributo obsoleto `slot` |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [Incorreto](#vue-no-deprecated-slot-scope-attribute-bad) · [Correto](#vue-no-deprecated-slot-scope-attribute-good) | Proibir o atributo obsoleto `slot-scope` |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [Incorreto](#vue-no-deprecated-v-bind-sync-bad) · [Correto](#vue-no-deprecated-v-bind-sync-good) | Proibir o modificador obsoleto `.sync` em `v-bind` |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [Incorreto](#vue-no-deprecated-v-on-native-modifier-bad) · [Correto](#vue-no-deprecated-v-on-native-modifier-good) | Proibir o modificador obsoleto `.native` em `v-on` |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [Incorreto](#vue-no-deprecated-v-on-number-modifiers-bad) · [Correto](#vue-no-deprecated-v-on-number-modifiers-good) | Proibir modificadores numéricos obsoletos de `keyCode` em `v-on` |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [Incorreto](#vue-no-dupe-v-else-if-bad) · [Correto](#vue-no-dupe-v-else-if-good) | Proibir condições duplicadas em cadeias de `v-if` / `v-else-if` |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [Incorreto](#vue-no-duplicate-attributes-bad) · [Correto](#vue-no-duplicate-attributes-good) | Proibir atributos duplicados no mesmo elemento |
| [`vue/no-empty-component-block`](#vue-no-empty-component-block) | [Incorreto](#vue-no-empty-component-block-bad) · [Correto](#vue-no-empty-component-block-good) | Proibir blocos vazios em SFCs |
| [`vue/no-inline-style`](#vue-no-inline-style) | [Incorreto](#vue-no-inline-style-bad) · [Correto](#vue-no-inline-style-good) | Desencorajar o uso de atributos de estilo inline |
| [`vue/no-invalid-html-attribute`](#vue-no-invalid-html-attribute) | [Incorreto](#vue-no-invalid-html-attribute-bad) · [Correto](#vue-no-invalid-html-attribute-good) | Proibir valores estáticos inválidos para atributos HTML |
| [`vue/no-lone-template`](#vue-no-lone-template) | [Incorreto](#vue-no-lone-template-bad) · [Correto](#vue-no-lone-template-good) | Proibir elementos `<template>` desnecessários |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [Incorreto](#vue-no-multi-spaces-bad) · [Correto](#vue-no-multi-spaces-good) | Proibir vários espaços consecutivos |
| [`vue/no-multiple-objects-in-class`](#vue-no-multiple-objects-in-class) | [Incorreto](#vue-no-multiple-objects-in-class-bad) · [Correto](#vue-no-multiple-objects-in-class-good) | Proibir vários objetos literais dentro de uma vinculação de array em :class |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [Incorreto](#vue-no-multiple-template-root-bad) · [Correto](#vue-no-multiple-template-root-good) | Proibir vários nós raiz em um template |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [Incorreto](#vue-no-mutating-props-bad) · [Correto](#vue-no-mutating-props-good) | Proibir a mutação de props de componentes |
| [`vue/no-negated-v-if-condition`](#vue-no-negated-v-if-condition) | [Incorreto](#vue-no-negated-v-if-condition-bad) · [Correto](#vue-no-negated-v-if-condition-good) | Proibir uma condição negada em v-if quando a cadeia tiver v-else |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [Incorreto](#vue-no-non-component-keep-alive-child-bad) · [Correto](#vue-no-non-component-keep-alive-child-good) | Proibir invólucros de elementos comuns diretamente abaixo de `<KeepAlive>` |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [Incorreto](#vue-no-preprocessor-lang-bad) · [Correto](#vue-no-preprocessor-lang-good) | Desencorajar o uso de preprocessadores CSS em favor de CSS moderno |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [Incorreto](#vue-no-reserved-component-names-bad) · [Correto](#vue-no-reserved-component-names-good) | Proibir o uso de nomes reservados como nomes de componentes |
| [`vue/no-root-v-if`](#vue-no-root-v-if) | [Incorreto](#vue-no-root-v-if-bad) · [Correto](#vue-no-root-v-if-good) | Proibir v-if no único elemento raiz de um template |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [Incorreto](#vue-no-script-non-standard-lang-bad) · [Correto](#vue-no-script-non-standard-lang-good) | Desencorajar valores não padronizados de lang em scripts |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [Incorreto](#vue-no-src-attribute-bad) · [Correto](#vue-no-src-attribute-good) | Desencorajar o atributo src em blocos de SFCs |
| [`vue/no-static-inline-styles`](#vue-no-static-inline-styles) | [Incorreto](#vue-no-static-inline-styles-bad) · [Correto](#vue-no-static-inline-styles-good) | Proibir atributos estáticos de estilo inline |
| [`vue/no-template-key`](#vue-no-template-key) | [Incorreto](#vue-no-template-key-bad) · [Correto](#vue-no-template-key-good) | Proibir o atributo `key` em `<template>` |
| [`vue/no-template-lang`](#vue-no-template-lang) | [Incorreto](#vue-no-template-lang-bad) · [Correto](#vue-no-template-lang-good) | Desencorajar o atributo lang no bloco template |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [Incorreto](#vue-no-template-shadow-bad) · [Correto](#vue-no-template-shadow-good) | Proibir nomes de variáveis que ocultam variáveis de um escopo externo |
| [`vue/no-template-target-blank`](#vue-no-template-target-blank) | [Incorreto](#vue-no-template-target-blank-bad) · [Correto](#vue-no-template-target-blank-good) | Proibir target="_blank" sem rel="noopener noreferrer" |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [Incorreto](#vue-no-textarea-mustache-bad) · [Correto](#vue-no-textarea-mustache-good) | Proibir interpolação com chaves duplas em `<textarea>` |
| [`vue/no-undefined-refs`](#vue-no-undefined-refs) | [Incorreto](#vue-no-undefined-refs-bad) · [Correto](#vue-no-undefined-refs-good) | Proibir referências a variáveis não definidas nos templates |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [Incorreto](#vue-no-unsafe-url-bad) · [Correto](#vue-no-unsafe-url-good) | Alertar sobre vinculações de URL potencialmente inseguras |
| [`vue/no-unsandboxed-iframe`](#vue-no-unsandboxed-iframe) | [Incorreto](#vue-no-unsandboxed-iframe-bad) · [Correto](#vue-no-unsandboxed-iframe-good) | Exigir um atributo sandbox nos elementos iframe |
| [`vue/no-unused-components`](#vue-no-unused-components) | [Incorreto](#vue-no-unused-components-bad) · [Correto](#vue-no-unused-components-good) | Proibir o registro de componentes não usados nos templates |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [Incorreto](#vue-no-unused-properties-bad) · [Correto](#vue-no-unused-properties-good) | Proibir propriedades não usadas definidas em defineProps |
| [`vue/no-unused-refs`](#vue-no-unused-refs) | [Incorreto](#vue-no-unused-refs-bad) · [Correto](#vue-no-unused-refs-good) | Reportar refs de template (ref="x") nunca referenciadas em &lt;script&gt; |
| [`vue/no-unused-setup-bindings`](#vue-no-unused-setup-bindings) | [Incorreto](#vue-no-unused-setup-bindings-bad) · [Correto](#vue-no-unused-setup-bindings-good) | Proibir variáveis de script setup que nunca são lidas |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [Incorreto](#vue-no-unused-vars-bad) · [Correto](#vue-no-unused-vars-good) | Proibir definições de variáveis não usadas nas diretivas v-for e v-slot |
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [Incorreto](#vue-no-use-v-else-with-v-for-bad) · [Correto](#vue-no-use-v-else-with-v-for-good) | Proibir `v-else-if` ou `v-else` no mesmo elemento que `v-for` |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [Incorreto](#vue-no-use-v-if-with-v-for-bad) · [Correto](#vue-no-use-v-if-with-v-for-good) | Proibir `v-if` no mesmo elemento que `v-for` |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [Incorreto](#vue-no-useless-mustaches-bad) · [Correto](#vue-no-useless-mustaches-good) | Proibir interpolação com chaves duplas cuja expressão seja uma string literal constante |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [Incorreto](#vue-no-useless-template-attributes-bad) · [Correto](#vue-no-useless-template-attributes-good) | Proibir atributos sem efeito em elementos `<template>` |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [Incorreto](#vue-no-useless-v-bind-bad) · [Correto](#vue-no-useless-v-bind-good) | Proibir um v-bind cujo valor seja uma string literal simples |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [Incorreto](#vue-no-v-for-template-key-on-child-bad) · [Correto](#vue-no-v-for-template-key-on-child-good) | Proibir `key` no filho de um `<template v-for>` |
| [`vue/no-v-html`](#vue-no-v-html) | [Incorreto](#vue-no-v-html-bad) · [Correto](#vue-no-v-html-good) | Alertar sobre v-html para prevenir vulnerabilidades XSS |
| [`vue/no-v-text`](#vue-no-v-text) | [Incorreto](#vue-no-v-text-bad) · [Correto](#vue-no-v-text-good) | Proibir a diretiva v-text; preferir interpolação com chaves duplas |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [Incorreto](#vue-no-v-text-v-html-on-component-bad) · [Correto](#vue-no-v-text-v-html-on-component-good) | Proibir v-text / v-html em elementos de componente |
| [`vue/permitted-contents`](#vue-permitted-contents) | [Incorreto](#vue-permitted-contents-bad) · [Correto](#vue-permitted-contents-good) | Aplicar as regras do modelo de conteúdo HTML |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [Incorreto](#vue-prefer-props-shorthand-bad) · [Correto](#vue-prefer-props-shorthand-good) | Recomendar sintaxe abreviada para props (Vue 3.4+) |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [Incorreto](#vue-prefer-true-attribute-shorthand-bad) · [Correto](#vue-prefer-true-attribute-shorthand-good) | Preferir a forma abreviada para um atributo booleano vinculado a `true` |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [Incorreto](#vue-prop-name-casing-bad) · [Correto](#vue-prop-name-casing-good) | Aplicar um padrão de maiúsculas e minúsculas aos nomes de props declaradas |
| [`vue/require-component-is`](#vue-require-component-is) | [Incorreto](#vue-require-component-is-bad) · [Correto](#vue-require-component-is-good) | Exigir `v-bind:is` em elementos `<component>` |
| [`vue/require-component-registration`](#vue-require-component-registration) | [Incorreto](#vue-require-component-registration-bad) · [Correto](#vue-require-component-registration-good) | Exigir importação ou registro explícito de componentes |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [Incorreto](#vue-require-scoped-style-bad) · [Correto](#vue-require-scoped-style-good) | Exigir o atributo scoped nas tags style |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [Incorreto](#vue-require-toggle-inside-transition-bad) · [Correto](#vue-require-toggle-inside-transition-good) | Exigir uma alternância no elemento envolvido por `<transition>` |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [Incorreto](#vue-require-v-for-key-bad) · [Correto](#vue-require-v-for-key-good) | Exigir `v-bind:key` nas diretivas `v-for` |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [Incorreto](#vue-scoped-event-names-bad) · [Correto](#vue-scoped-event-names-good) | Recomendar nomes de eventos com escopo no formato context:event |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [Incorreto](#vue-sfc-element-order-bad) · [Correto](#vue-sfc-element-order-good) | Aplicar uma ordem consistente aos elementos de nível superior dos SFCs |
| [`vue/single-style-block`](#vue-single-style-block) | [Incorreto](#vue-single-style-block-bad) · [Correto](#vue-single-style-block-good) | Recomendar um único bloco style |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [Incorreto](#vue-slot-name-casing-bad) · [Correto](#vue-slot-name-casing-good) | Exigir kebab-case nos slots nomeados usados por v-slot |
| [`vue/this-in-template`](#vue-this-in-template) | [Incorreto](#vue-this-in-template-bad) · [Correto](#vue-this-in-template-good) | Proibir `this.` nas expressões do template |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Incorreto](#vue-use-unique-element-ids-bad) · [Correto](#vue-use-unique-element-ids-good) | Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [Incorreto](#vue-use-v-on-exact-bad) · [Correto](#vue-use-v-on-exact-good) | Exigir o modificador `.exact` em `v-on` quando houver manipuladores baseados em modificadores |
| [`vue/v-bind-style`](#vue-v-bind-style) | [Incorreto](#vue-v-bind-style-bad) · [Correto](#vue-v-bind-style-good) | Aplicar um padrão de sintaxe à diretiva `v-bind` |
| [`vue/v-on-event-hyphenation`](#vue-v-on-event-hyphenation) | [Incorreto](#vue-v-on-event-hyphenation-bad) · [Correto](#vue-v-on-event-hyphenation-good) | Exigir hífens nos nomes de eventos personalizados em v-on de componentes |
| [`vue/v-on-handler-style`](#vue-v-on-handler-style) | [Incorreto](#vue-v-on-handler-style-bad) · [Correto](#vue-v-on-handler-style-good) | Exigir manipuladores de v-on escritos como referência a método ou função inline |
| [`vue/v-on-style`](#vue-v-on-style) | [Incorreto](#vue-v-on-style-bad) · [Correto](#vue-v-on-style-good) | Aplicar um padrão de sintaxe à diretiva `v-on` |
| [`vue/v-slot-style`](#vue-v-slot-style) | [Incorreto](#vue-v-slot-style-bad) · [Correto](#vue-v-slot-style-good) | Aplicar um padrão de sintaxe à diretiva `v-slot` |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [Incorreto](#vue-valid-attribute-name-bad) · [Correto](#vue-valid-attribute-name-good) | Exigir nomes de atributos válidos |
| [`vue/valid-template-root`](#vue-valid-template-root) | [Incorreto](#vue-valid-template-root-bad) · [Correto](#vue-valid-template-root-good) | Exigir uma raiz `<template>` válida para a semântica de fragmentos do Vue 3 |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [Incorreto](#vue-valid-v-bind-bad) · [Correto](#vue-valid-v-bind-good) | Exigir diretivas `v-bind` válidas |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [Incorreto](#vue-valid-v-cloak-bad) · [Correto](#vue-valid-v-cloak-good) | Exigir diretivas `v-cloak` válidas |
| [`vue/valid-v-else`](#vue-valid-v-else) | [Incorreto](#vue-valid-v-else-bad) · [Correto](#vue-valid-v-else-good) | Exigir diretivas `v-else` válidas |
| [`vue/valid-v-for`](#vue-valid-v-for) | [Incorreto](#vue-valid-v-for-bad) · [Correto](#vue-valid-v-for-good) | Exigir diretivas `v-for` válidas |
| [`vue/valid-v-html`](#vue-valid-v-html) | [Incorreto](#vue-valid-v-html-bad) · [Correto](#vue-valid-v-html-good) | Exigir diretivas `v-html` válidas |
| [`vue/valid-v-if`](#vue-valid-v-if) | [Incorreto](#vue-valid-v-if-bad) · [Correto](#vue-valid-v-if-good) | Exigir diretivas `v-if` válidas |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [Incorreto](#vue-valid-v-memo-bad) · [Correto](#vue-valid-v-memo-good) | Exigir diretivas `v-memo` válidas |
| [`vue/valid-v-model`](#vue-valid-v-model) | [Incorreto](#vue-valid-v-model-bad) · [Correto](#vue-valid-v-model-good) | Exigir diretivas `v-model` válidas |
| [`vue/valid-v-on`](#vue-valid-v-on) | [Incorreto](#vue-valid-v-on-bad) · [Correto](#vue-valid-v-on-good) | Exigir diretivas `v-on` válidas |
| [`vue/valid-v-once`](#vue-valid-v-once) | [Incorreto](#vue-valid-v-once-bad) · [Correto](#vue-valid-v-once-good) | Exigir diretivas `v-once` válidas |
| [`vue/valid-v-show`](#vue-valid-v-show) | [Incorreto](#vue-valid-v-show-bad) · [Correto](#vue-valid-v-show-good) | Exigir diretivas `v-show` válidas |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [Incorreto](#vue-valid-v-slot-bad) · [Correto](#vue-valid-v-slot-good) | Exigir diretivas `v-slot` válidas |
| [`vue/valid-v-text`](#vue-valid-v-text) | [Incorreto](#vue-valid-v-text-bad) · [Correto](#vue-valid-v-text-good) | Exigir diretivas `v-text` válidas |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [Incorreto](#vue-warn-custom-block-bad) · [Correto](#vue-warn-custom-block-good) | Alertar sobre blocos personalizados em arquivos SFC |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [Incorreto](#vue-warn-custom-directive-bad) · [Correto](#vue-warn-custom-directive-good) | Alertar sobre diretivas personalizadas que precisam de registro |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `vue/a11y-img-alt`

Exigir o atributo alt nas imagens para garantir acessibilidade

[Incorreto](#vue-a11y-img-alt-bad) · [Correto](#vue-a11y-img-alt-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/a11y-img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-a11y-img-alt-bad"></span>

**Incorreto**

Nem a imagem estática nem a imagem de origem dinâmica fornecem um atributo alt.

```vue annotate="remove:2,3"
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

<span id="vue-a11y-img-alt-good"></span>

**Correto**

Imagens informativas recebem um texto alt descritivo; imagens decorativas recebem um alt vazio; a imagem dinâmica vincula sua descrição.

```vue annotate="add:2,3,4,5,6,7,8,9"
<template>
<!-- Informative image -->
<img src="/photo.jpg" alt="Team photo from company retreat" />

<!-- Decorative image (empty alt) -->
<img src="/decoration.svg" alt="" />

<!-- Dynamic alt -->
<img :src="photo" :alt="photoDescription" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) · [Todas as regras](all.md)

### `vue/attribute-hyphenation`

Aplicar um padrão de nomes de atributos em componentes personalizados

[Incorreto](#vue-attribute-hyphenation-bad) · [Correto](#vue-attribute-hyphenation-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/attribute-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-hyphenation-bad"></span>

**Incorreto**

O atributo do componente usa a grafia camelCase firstName.

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**Correto**

A grafia first-name segue a convenção configurada de atributos de componente separados por hífen.

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [Todas as regras](all.md)

### `vue/attribute-order`

Aplicar uma ordem consistente aos atributos

[Incorreto](#vue-attribute-order-bad) · [Correto](#vue-attribute-order-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/attribute-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-order-bad"></span>

**Incorreto**

O manipulador de evento aparece antes da diretiva estrutural v-if e do atributo comum id.

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**Correto**

v-if vem primeiro, seguido de id e do manipulador de evento, conforme a ordem da regra.

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [Todas as regras](all.md)

### `vue/component-definition-name-casing`

Exigir PascalCase ou kebab-case nos nomes de definição de componentes

[Incorreto](#vue-component-definition-name-casing-bad) · [Correto](#vue-component-definition-name-casing-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

O nome de arquivo do componente é verificado. PascalCase e kebab-case são aceitos; o uso misturado de maiúsculas e minúsculas é reportado.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-definition-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-definition-name-casing-bad"></span>

**Incorreto**

O nome de arquivo myComponent.vue mistura uma inicial minúscula com uma letra maiúscula interna, em vez de usar PascalCase ou kebab-case.

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

<span id="vue-component-definition-name-casing-good"></span>

**Correto**

Renomear o arquivo para MyComponent.vue aplica PascalCase; o conteúdo do template permanece igual.

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [Todas as regras](all.md)

### `vue/component-name-in-template-casing`

Aplicar um padrão específico de maiúsculas e minúsculas aos nomes de componentes nos templates

[Incorreto](#vue-component-name-in-template-casing-bad) · [Correto](#vue-component-name-in-template-casing-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-name-in-template-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-name-in-template-casing-bad"></span>

**Incorreto**

O componente é escrito em kebab-case e camelCase, embora a convenção seja PascalCase.

```vue annotate="remove:5,6"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <my-component />
  <myComponent />
</template>
```

<span id="vue-component-name-in-template-casing-good"></span>

**Correto**

MyComponent usa PascalCase; a sintaxe nativa de slot permanece em minúsculas.

```vue annotate="add:5,6,7"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <MyComponent />
  <RouterView />
  <slot />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) · [Todas as regras](all.md)

### `vue/html-button-has-type`

Exigir um type explícito e válido nos elementos button

[Incorreto](#vue-html-button-has-type-bad) · [Correto](#vue-html-button-has-type-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/html-button-has-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-button-has-type-bad"></span>

**Incorreto**

Um botão omite type e outro fornece o tipo foo, que não é aceito.

```vue annotate="remove:2,3"
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

<span id="vue-html-button-has-type-good"></span>

**Correto**

Os botões especificam button, submit ou reset; um type vinculado é tratado como dinâmico.

```vue annotate="add:2,3,4,5"
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) · [Todas as regras](all.md)

### `vue/html-quotes`

Aplicar um padrão de aspas aos atributos HTML

[Incorreto](#vue-html-quotes-bad) · [Correto](#vue-html-quotes-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/html-quotes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-quotes-bad"></span>

**Incorreto**

Os atributos usam aspas simples ou nenhuma aspa, em vez da convenção de aspas duplas.

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**Correto**

Tanto os atributos comuns quanto as expressões de diretivas usam aspas duplas.

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [Todas as regras](all.md)

### `vue/html-self-closing`

Aplicar um padrão de tags com fechamento automático

[Incorreto](#vue-html-self-closing-bad) · [Correto](#vue-html-self-closing-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/html-self-closing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-self-closing-bad"></span>

**Incorreto**

O componente vazio usa uma tag de fechamento separada, enquanto os elementos vazios img e br omitem a grafia de fechamento automático configurada.

```vue annotate="remove:2,3,4"
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**Correto**

O componente e os elementos vazios usam a sintaxe de fechamento automático; uma div com conteúdo mantém sua tag de fechamento.

```vue annotate="add:2,3,4,5,6,7"
<template>
  <MyComponent />
  <div></div>
  <div />
  <img />
  <br />
  <div>content</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [Todas as regras](all.md)

### `vue/max-template-complexity`

Limitar a complexidade do próprio template de um componente, tanto ciclomática quanto cognitiva

[Incorreto](#vue-max-template-complexity-bad) · [Correto](#vue-max-template-complexity-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Bad tem complexidade ciclomática 13 e complexidade cognitiva 25 (limites: 11 e 16). Cada componente é medido separadamente; apenas templates HTML inline são suportados.

Veja [o cálculo da complexidade e os limites dos componentes](../guide/cross-file-complexity.md) para conhecer as contribuições às duas pontuações do exemplo.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/max-template-complexity": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-max-template-complexity-bad"></span>

**Incorreto**

As ramificações, o laço, o conteúdo de slot e as decisões em expressões escritos pelo componente pai produzem pontuações de 13 e 25, acima dos limites padrão de 11 e 16.

```vue annotate="remove:1,2,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19"
<script setup lang="ts">
defineProps<{ rows: Row[] }>();
</script>
<template>
  <section>
    <h1>{{ user ? user.name : 'Guest' }}</h1>
    <DataTable :rows="rows">
      <template #cell="{ row, column }">
        <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">{{ row.status ?? 'unknown' }}</span>
        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
        <template v-else>
          <em v-for="tag in row.tags" :key="tag">
            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? '!' : '' }}</b>
          </em>
        </template>
      </template>
    </DataTable>
    <p v-if="!rows.length && !loading">No data</p>
  </section>
</template>
```

<span id="vue-max-template-complexity-good"></span>

**Correto**

O template pai delega a renderização a RowList e mantém um v-if; suas próprias pontuações são 2 e 1.

```vue annotate="add:2"
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) · [Todas as regras](all.md)

### `vue/multi-word-component-names`

Exigir nomes de componentes com mais de uma palavra

[Incorreto](#vue-multi-word-component-names-bad) · [Correto](#vue-multi-word-component-names-good)

Severidade padrão: `error`  
Predefinições: `essential`, `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

A ocorrência diagnosticada é o nome de arquivo. Renomeie o mesmo componente; alterar uma tag filha não corrige o problema.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/multi-word-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-multi-word-component-names-bad"></span>

**Incorreto**

Item.vue dá ao componente um nome de uma única palavra.

`Item.vue`

```vue
<template><p>Item</p></template>
```

<span id="vue-multi-word-component-names-good"></span>

**Correto**

TodoItem.vue dá ao mesmo template um nome de componente com mais de uma palavra.

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [Todas as regras](all.md)

### `vue/mustache-interpolation-spacing`

Aplicar espaçamento consistente dentro das interpolações com chaves duplas

[Incorreto](#vue-mustache-interpolation-spacing-bad) · [Correto](#vue-mustache-interpolation-spacing-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/mustache-interpolation-spacing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-mustache-interpolation-spacing-bad"></span>

**Incorreto**

A interpolação de texto não tem um espaço junto a um ou a ambos os delimitadores.

```vue annotate="remove:2,3,4"
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**Correto**

Espaços separam a expressão dos delimitadores de abertura e fechamento das chaves duplas.

```vue annotate="add:2,3,4"
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [Todas as regras](all.md)

### `vue/no-array-index-key`

Proibir o uso direto da variável de índice de v-for como :key

[Incorreto](#vue-no-array-index-key-bad) · [Correto](#vue-no-array-index-key-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-array-index-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-array-index-key-bad"></span>

**Incorreto**

A chave da lista é seu índice atual, então a identidade do item muda quando a lista é reordenada.

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

<span id="vue-no-array-index-key-good"></span>

**Correto**

A chave vem de item.id, preservando a identidade de cada item quando sua posição muda.

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) · [Todas as regras](all.md)

### `vue/no-bare-strings-in-template`

Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado

[Incorreto](#vue-no-bare-strings-in-template-bad) · [Correto](#vue-no-bare-strings-in-template-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-bare-strings-in-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-bare-strings-in-template-bad"></span>

**Incorreto**

O texto visível e os atributos de identificação incorporam strings sem tradução diretamente no template.

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

<span id="vue-no-bare-strings-in-template-good"></span>

**Correto**

O conteúdo traduzível chama $t; os exemplos com pontuação e apenas números são exceções permitidas.

```vue annotate="add:2,3,4,5,6"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<div>123</div>
<button :title="$t('close')">{{ $t('x') }}</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47) · [Todas as regras](all.md)

### `vue/no-boolean-attr-value`

Proibir valores explícitos em atributos HTML booleanos

[Incorreto](#vue-no-boolean-attr-value-bad) · [Correto](#vue-no-boolean-attr-value-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-boolean-attr-value": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-boolean-attr-value-bad"></span>

**Incorreto**

Os atributos booleanos disabled e checked contêm valores de string redundantes.

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**Correto**

A presença de cada atributo booleano expressa o mesmo estado ativado, sem um valor.

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [Todas as regras](all.md)

### `vue/no-child-content`

Proibir conteúdo filho ao usar v-html ou v-text

[Incorreto](#vue-no-child-content-bad) · [Correto](#vue-no-child-content-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-child-content": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-child-content-bad"></span>

**Incorreto**

v-text substitui o conteúdo do parágrafo, então o texto alternativo escrito no template não pode ser preservado por essa diretiva.

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**Correto**

Remover o texto filho deixa v-text como a única fonte de conteúdo do parágrafo.

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [Todas as regras](all.md)

### `vue/no-deprecated-filter`

Proibir a sintaxe obsoleta de filtros do Vue 2 com o operador de barra vertical

[Incorreto](#vue-no-deprecated-filter-bad) · [Correto](#vue-no-deprecated-filter-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-filter": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-filter-bad"></span>

**Incorreto**

A barra vertical usa a sintaxe de filtros removida do Vue para aplicar capitalize.

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

<span id="vue-no-deprecated-filter-good"></span>

**Correto**

Chamar capitalize(message) aplica a transformação como uma expressão comum.

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) · [Todas as regras](all.md)

### `vue/no-deprecated-functional-template`

Proibir o atributo `functional` no `<template>` de um SFC

[Incorreto](#vue-no-deprecated-functional-template-bad) · [Correto](#vue-no-deprecated-functional-template-good)

Severidade padrão: `error`  
Predefinições: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-functional-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-functional-template-bad"></span>

**Incorreto**

O template do SFC tem o atributo functional removido e lê o antigo contexto props.

```vue annotate="remove:1,2"
<template functional>
  <div>{{ props.msg }}</div>
</template>
```

<span id="vue-no-deprecated-functional-template-good"></span>

**Correto**

O template comum omite functional e lê diretamente a variável msg exposta pelo componente.

```vue annotate="add:1,2"
<template>
  <div>{{ msg }}</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) · [Todas as regras](all.md)

### `vue/no-deprecated-html-element-is`

Proibir o atributo `is` em elementos HTML nativos

[Incorreto](#vue-no-deprecated-html-element-is-bad) · [Correto](#vue-no-deprecated-html-element-is-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-html-element-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-html-element-is-bad"></span>

**Incorreto**

Uma div nativa usa o antigo atributo is sem prefixo para solicitar um componente Vue.

```vue annotate="remove:2"
<template>
  <div is="MyComponent" />
</template>
```

<span id="vue-no-deprecated-html-element-is-good"></span>

**Correto**

Um componente dinâmico usa :is; a forma no elemento nativo usa explicitamente o prefixo vue:.

```vue annotate="add:2,3"
<template>
  <component :is="MyComponent" />
  <div is="vue:MyComponent" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) · [Todas as regras](all.md)

### `vue/no-deprecated-inline-template`

Proibir o atributo obsoleto `inline-template`

[Incorreto](#vue-no-deprecated-inline-template-bad) · [Correto](#vue-no-deprecated-inline-template-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-inline-template-bad"></span>

**Incorreto**

Card usa o atributo obsoleto inline-template para o conteúdo fornecido.

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

<span id="vue-no-deprecated-inline-template-good"></span>

**Correto**

O mesmo conteúdo é passado normalmente, sem o atributo inline-template.

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) · [Todas as regras](all.md)

### `vue/no-deprecated-router-link-tag-prop`

Proibir a prop `tag` em &lt;router-link&gt;

[Incorreto](#vue-no-deprecated-router-link-tag-prop-bad) · [Correto](#vue-no-deprecated-router-link-tag-prop-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-router-link-tag-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-router-link-tag-prop-bad"></span>

**Incorreto**

RouterLink usa a prop tag removida para solicitar um elemento button.

```vue annotate="remove:2"
<template>
  <router-link to="/home" tag="button">Home</router-link>
</template>
```

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

**Correto**

O slot fornece navigate a um botão escrito explicitamente no template.

```vue annotate="add:2,3,4"
<template>
  <router-link to="/home" v-slot="{ navigate }">
    <button @click="navigate">Home</button>
  </router-link>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [Todas as regras](all.md)

### `vue/no-deprecated-scope-attribute`

Proibir o atributo obsoleto `scope` em &lt;template&gt;

[Incorreto](#vue-no-deprecated-scope-attribute-bad) · [Correto](#vue-no-deprecated-scope-attribute-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-scope-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-scope-attribute-bad"></span>

**Incorreto**

O template do slot declara props pelo atributo obsoleto scope.

```vue annotate="remove:2"
<template>
<Card><template scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-scope-attribute-good"></span>

**Correto**

A diretiva do slot padrão declara a mesma variável props pela sintaxe atual de slots.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) · [Todas as regras](all.md)

### `vue/no-deprecated-slot-attribute`

Proibir o atributo obsoleto `slot`

[Incorreto](#vue-no-deprecated-slot-attribute-bad) · [Correto](#vue-no-deprecated-slot-attribute-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-slot-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-slot-attribute-bad"></span>

**Incorreto**

O slot header é selecionado pelo antigo atributo slot.

```vue annotate="remove:3,4"
<template>
  <Foo>
    <template slot="header"><h1>Title</h1></template>
    <div :slot="name">Title</div>
  </Foo>
</template>
```

<span id="vue-no-deprecated-slot-attribute-good"></span>

**Correto**

v-slot:header seleciona explicitamente o slot header com a diretiva atual.

```vue annotate="add:3"
<template>
  <Foo>
    <template v-slot:header><h1>Title</h1></template>
  </Foo>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) · [Todas as regras](all.md)

### `vue/no-deprecated-slot-scope-attribute`

Proibir o atributo obsoleto `slot-scope`

[Incorreto](#vue-no-deprecated-slot-scope-attribute-bad) · [Correto](#vue-no-deprecated-slot-scope-attribute-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-slot-scope-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-slot-scope-attribute-bad"></span>

**Incorreto**

O template recebe as props do slot pelo atributo obsoleto slot-scope.

```vue annotate="remove:2"
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

**Correto**

A diretiva #default recebe essas props sem slot-scope.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) · [Todas as regras](all.md)

### `vue/no-deprecated-v-bind-sync`

Proibir o modificador obsoleto `.sync` em `v-bind`

[Incorreto](#vue-no-deprecated-v-bind-sync-bad) · [Correto](#vue-no-deprecated-v-bind-sync-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-bind-sync": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-bind-sync-bad"></span>

**Incorreto**

As vinculações usam o modificador .sync removido, inclusive em combinação com .camel.

```vue annotate="remove:2,3,4"
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

<span id="vue-no-deprecated-v-bind-sync-good"></span>

**Correto**

Use uma vinculação comum unidirecional de title ou v-model:title quando for necessário um canal de atualização.

```vue annotate="add:2,3"
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) · [Todas as regras](all.md)

### `vue/no-deprecated-v-on-native-modifier`

Proibir o modificador obsoleto `.native` em `v-on`

[Incorreto](#vue-no-deprecated-v-on-native-modifier-bad) · [Correto](#vue-no-deprecated-v-on-native-modifier-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-on-native-modifier": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-on-native-modifier-bad"></span>

**Incorreto**

Os manipuladores do componente usam o modificador de evento .native removido.

```vue annotate="remove:2,3,4"
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

**Correto**

Os manipuladores omitem .native e preservam outros modificadores de evento, como .stop.

```vue annotate="add:2,3"
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) · [Todas as regras](all.md)

### `vue/no-deprecated-v-on-number-modifiers`

Proibir modificadores numéricos obsoletos de `keyCode` em `v-on`

[Incorreto](#vue-no-deprecated-v-on-number-modifiers-bad) · [Correto](#vue-no-deprecated-v-on-number-modifiers-good)

Severidade padrão: `error`  
Predefinições: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-on-number-modifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-on-number-modifiers-bad"></span>

**Incorreto**

Os manipuladores de teclado identificam as teclas pelos códigos numéricos removidos 13 e 27.

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

**Correto**

Os manipuladores usam os modificadores de tecla nomeados enter e esc.

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [Todas as regras](all.md)

### `vue/no-dupe-v-else-if`

Proibir condições duplicadas em cadeias de `v-if` / `v-else-if`

[Incorreto](#vue-no-dupe-v-else-if-bad) · [Correto](#vue-no-dupe-v-else-if-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-dupe-v-else-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-dupe-v-else-if-bad"></span>

**Incorreto**

O else-if repete a condição ready já testada pelo primeiro ramo, tornando esse ramo posterior inacessível.

```vue annotate="remove:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**Correto**

O segundo ramo testa loading, um estado distinto que pode alcançar o else-if.

```vue annotate="add:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [Todas as regras](all.md)

### `vue/no-duplicate-attributes`

Proibir atributos duplicados no mesmo elemento

[Incorreto](#vue-no-duplicate-attributes-bad) · [Correto](#vue-no-duplicate-attributes-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-duplicate-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-duplicate-attributes-bad"></span>

**Incorreto**

O mesmo botão declara class duas vezes, em vez de usar um único valor combinado de class.

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**Correto**

Os dois nomes de classe aparecem em um único atributo class.

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [Todas as regras](all.md)

### `vue/no-empty-component-block`

Proibir blocos vazios em SFCs

[Incorreto](#vue-no-empty-component-block-bad) · [Correto](#vue-no-empty-component-block-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-empty-component-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-empty-component-block-bad"></span>

**Incorreto**

Os blocos template, script e style não contêm conteúdo significativo.

```vue annotate="remove:1,3,5"
<template></template>

<script></script>

<style>
</style>
```

<span id="vue-no-empty-component-block-good"></span>

**Correto**

Cada bloco mantido contém marcação, declarações de script ou declarações de estilo de fato.

```vue annotate="add:1,2,3,5,6,7,9,10"
<template>
  <div>Hello</div>
</template>

<script setup>
const message = "Hello";
</script>

<style scoped>
.button { color: red; }
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) · [Todas as regras](all.md)

### `vue/no-inline-style`

Desencorajar o uso de atributos de estilo inline

[Incorreto](#vue-no-inline-style-bad) · [Correto](#vue-no-inline-style-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-inline-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-inline-style-bad"></span>

**Incorreto**

O atributo style estático incorpora a declaração de cor no elemento.

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**Correto**

Classes expressam a cor fixa; a largura dependente de ratio permanece como uma vinculação dinâmica de estilo, fora da verificação de atributos estáticos.

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [Todas as regras](all.md)

### `vue/no-invalid-html-attribute`

Proibir valores estáticos inválidos para atributos HTML

[Incorreto](#vue-no-invalid-html-attribute-bad) · [Correto](#vue-no-invalid-html-attribute-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-invalid-html-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-invalid-html-attribute-bad"></span>

**Incorreto**

A âncora usa stylesheet como valor de rel, embora esse valor pertença a elementos link de folhas de estilo.

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

<span id="vue-no-invalid-html-attribute-good"></span>

**Correto**

A âncora usa help, um valor de rel apropriado para um recurso de ajuda vinculado.

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) · [Todas as regras](all.md)

### `vue/no-lone-template`

Proibir elementos `<template>` desnecessários

[Incorreto](#vue-no-lone-template-bad) · [Correto](#vue-no-lone-template-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-lone-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-lone-template-bad"></span>

**Incorreto**

O template interno não tem uma diretiva nem uma função de slot que lhe dê uma finalidade estrutural.

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**Correto**

Remover o invólucro desnecessário deixa o parágrafo diretamente dentro da div.

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [Todas as regras](all.md)

### `vue/no-multi-spaces`

Proibir vários espaços consecutivos

[Incorreto](#vue-no-multi-spaces-bad) · [Correto](#vue-no-multi-spaces-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multi-spaces": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multi-spaces-bad"></span>

**Incorreto**

Dois espaços separam os atributos ou o nome do elemento e o primeiro atributo.

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**Correto**

Espaços únicos separam os mesmos atributos.

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [Todas as regras](all.md)

### `vue/no-multiple-objects-in-class`

Proibir vários objetos literais dentro de uma vinculação de array em :class

[Incorreto](#vue-no-multiple-objects-in-class-bad) · [Correto](#vue-no-multiple-objects-in-class-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multiple-objects-in-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multiple-objects-in-class-bad"></span>

**Incorreto**

Um array de classes contém dois objetos literais no nível superior que podem ser combinados.

```vue annotate="remove:2,3"
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

<span id="vue-no-multiple-objects-in-class-good"></span>

**Correto**

Um único objeto contém as condições das classes; arrays com um objeto e uma string ou com entradas não literais continuam permitidos.

```vue annotate="add:2,3,4"
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) · [Todas as regras](all.md)

### `vue/no-multiple-template-root`

Proibir vários nós raiz em um template

[Incorreto](#vue-no-multiple-template-root-bad) · [Correto](#vue-no-multiple-template-root-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Ative apenas quando houver um contrato de raiz única. O Vue 3 normalmente suporta fragmentos.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multiple-template-root": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multiple-template-root-bad"></span>

**Incorreto**

A convenção opcional de raiz única encontra dois parágrafos irmãos na raiz do template.

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

<span id="vue-no-multiple-template-root-good"></span>

**Correto**

Uma section envolve os parágrafos em uma única raiz; ative essa convenção apenas quando houver um contrato de raiz única.

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) · [Todas as regras](all.md)

### `vue/no-mutating-props`

Proibir a mutação de props de componentes

[Incorreto](#vue-no-mutating-props-bad) · [Correto](#vue-no-mutating-props-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-mutating-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-mutating-props-bad"></span>

**Incorreto**

Incrementar props.count escreve diretamente em um valor fornecido pelo componente pai.

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**Correto**

O componente emite update:count com o próximo valor, deixando o pai responsável por atualizar a prop.

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L42) · [Todas as regras](all.md)

### `vue/no-negated-v-if-condition`

Proibir uma condição negada em v-if quando a cadeia tiver v-else

[Incorreto](#vue-no-negated-v-if-condition-bad) · [Correto](#vue-no-negated-v-if-condition-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-negated-v-if-condition": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-negated-v-if-condition-bad"></span>

**Incorreto**

Os ramos emparelhados v-if e v-else começam com uma condição negada.

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

<span id="vue-no-negated-v-if-condition-good"></span>

**Correto**

Uma condição positiva ok vem primeiro; ao inverter uma condição, coloque primeiro o ramo originalmente oposto. Um v-if negado isolado e comparações !== continuam permitidos.

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) · [Todas as regras](all.md)

### `vue/no-non-component-keep-alive-child`

Proibir invólucros de elementos comuns diretamente abaixo de `<KeepAlive>`

[Incorreto](#vue-no-non-component-keep-alive-child-bad) · [Correto](#vue-no-non-component-keep-alive-child-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-non-component-keep-alive-child": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-non-component-keep-alive-child-bad"></span>

**Incorreto**

KeepAlive envolve condicionalmente uma div nativa, em vez de armazenar UserCard diretamente em cache.

```vue annotate="remove:3"
<template>
  <KeepAlive>
    <div v-if="ready">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

<span id="vue-no-non-component-keep-alive-child-good"></span>

**Correto**

O primeiro exemplo torna UserCard o filho condicional. O invólucro com v-show ilustra uma estrutura fora desta verificação de filhos condicionais, sem prometer que o invólucro nativo seja armazenado em cache.

```vue annotate="add:3,4,5,6"
<template>
  <KeepAlive>
    <UserCard v-if="ready" />
  </KeepAlive>
  <KeepAlive>
    <div v-show="opened">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L14) · [Todas as regras](all.md)

### `vue/no-preprocessor-lang`

Desencorajar o uso de preprocessadores CSS em favor de CSS moderno

[Incorreto](#vue-no-preprocessor-lang-bad) · [Correto](#vue-no-preprocessor-lang-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Não implementada no lint de SFC  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Suporte atual: `no-sfc-finding`

Esta entrada do catálogo atualmente não emite uma ocorrência específica da regra pelo lint de SFCs. O par Bad/Good descreve a convenção pretendida, e não uma ocorrência executável. Ativar o ID não fornece a verificação de SFC ausente.

**ID configurado (sem diagnóstico de SFC atualmente)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-preprocessor-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-preprocessor-lang-bad"></span>

**Incorreto**

O bloco style seleciona SCSS com lang. Isso descreve a convenção pretendida de não usar preprocessadores; o processamento atual de SFCs não emite esta regra.

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**Correto**

As mesmas declarações CSS omitem o lang do preprocessador. Essa é a correção da convenção, e não uma diferença executável de diagnósticos entre Bad/Good hoje.

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

O exemplo correto ilustra a convenção pretendida; o fluxo atual de SFC não emite o diagnóstico específico da regra para nenhum dos exemplos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [Todas as regras](all.md)

### `vue/no-reserved-component-names`

Proibir o uso de nomes reservados como nomes de componentes

[Incorreto](#vue-no-reserved-component-names-bad) · [Correto](#vue-no-reserved-component-names-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-reserved-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-reserved-component-names-bad"></span>

**Incorreto**

O nome de componente button entra em conflito com o nome de um elemento HTML nativo.

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**Correto**

AppButton é um nome de componente da aplicação e não reutiliza o nome nativo button.

```vue annotate="add:1,2,4,5,6,7,8,9"
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) · [Todas as regras](all.md)

### `vue/no-root-v-if`

Proibir v-if no único elemento raiz de um template

[Incorreto](#vue-no-root-v-if-bad) · [Correto](#vue-no-root-v-if-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-root-v-if": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-root-v-if-bad"></span>

**Incorreto**

A própria raiz do componente aparece e desaparece sob v-if.

```vue annotate="remove:2"
<template>
  <div v-if="show">content</div>
</template>
```

<span id="vue-no-root-v-if-good"></span>

**Correto**

Uma div externa estável permanece como raiz, enquanto o parágrafo aninhado recebe a condição de visibilidade.

```vue annotate="add:2,3,4"
<template>
  <div>
    <p v-if="show">content</p>
  </div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [Todas as regras](all.md)

### `vue/no-script-non-standard-lang`

Desencorajar valores não padronizados de lang em scripts

[Incorreto](#vue-no-script-non-standard-lang-bad) · [Correto](#vue-no-script-non-standard-lang-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Não implementada no lint de SFC  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Suporte atual: `no-sfc-finding`

Esta entrada do catálogo atualmente não emite uma ocorrência específica da regra pelo lint de SFCs. O par Bad/Good descreve a convenção pretendida, e não uma ocorrência executável. Ativar o ID não fornece a verificação de SFC ausente.

**ID configurado (sem diagnóstico de SFC atualmente)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-script-non-standard-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-script-non-standard-lang-bad"></span>

**Incorreto**

O script usa sintaxe CoffeeScript com lang=coffee. O processamento atual de SFCs não emite esta regra do catálogo para essa linguagem.

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**Correto**

O script usa uma declaração TypeScript comum com lang=ts, ilustrando a convenção de linguagem pretendida.

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

O exemplo correto ilustra a convenção pretendida; o fluxo atual de SFC não emite o diagnóstico específico da regra para nenhum dos exemplos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [Todas as regras](all.md)

### `vue/no-src-attribute`

Desencorajar o atributo src em blocos de SFCs

[Incorreto](#vue-no-src-attribute-bad) · [Correto](#vue-no-src-attribute-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-src-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-src-attribute-bad"></span>

**Incorreto**

Os blocos do SFC delegam o conteúdo de template, script e style a arquivos src.

```vue annotate="remove:1,2,3"
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**Correto**

Cada bloco do SFC contém seu próprio conteúdo, sem um atributo src externo.

```vue annotate="add:1,2,3,4,5,6,7,8,9,10,11,12,13"
<template>
  <p>Hello</p>
</template>

<script setup lang="ts">
const label = "Hello";
</script>

<style scoped>
p {
  color: red;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) · [Todas as regras](all.md)

### `vue/no-static-inline-styles`

Proibir atributos estáticos de estilo inline

[Incorreto](#vue-no-static-inline-styles-bad) · [Correto](#vue-no-static-inline-styles-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-static-inline-styles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-static-inline-styles-bad"></span>

**Incorreto**

O parágrafo contém a declaração de cor constante no atributo style.

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

<span id="vue-no-static-inline-styles-good"></span>

**Correto**

Uma classe notice e uma folha de estilo com escopo mantêm a cor constante fora do atributo no template.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [Todas as regras](all.md)

### `vue/no-template-key`

Proibir o atributo `key` em `<template>`

[Incorreto](#vue-no-template-key-bad) · [Correto](#vue-no-template-key-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-key-bad"></span>

**Incorreto**

Um invólucro template sem laço tem uma key, embora não seja o limite da iteração com chave.

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**Correto**

A key pertence a uma iteração template v-for, na qual identifica cada fragmento repetido.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [Todas as regras](all.md)

### `vue/no-template-lang`

Desencorajar o atributo lang no bloco template

[Incorreto](#vue-no-template-lang-bad) · [Correto](#vue-no-template-lang-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Não implementada no lint de SFC  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Suporte atual: `no-sfc-finding`

Esta entrada do catálogo atualmente não emite uma ocorrência específica da regra pelo lint de SFCs. O par Bad/Good descreve a convenção pretendida, e não uma ocorrência executável. Ativar o ID não fornece a verificação de SFC ausente.

**ID configurado (sem diagnóstico de SFC atualmente)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-lang-bad"></span>

**Incorreto**

O template seleciona Pug por meio de lang. Essa é uma convenção pretendida de usar apenas HTML; o processamento atual de SFCs não gera diagnósticos para este ID do catálogo.

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**Correto**

Um template HTML comum omite lang e usa o parágrafo diretamente. Isso ilustra a convenção sem afirmar que há uma ocorrência diagnosticada atualmente em SFCs.

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

O exemplo correto ilustra a convenção pretendida; o fluxo atual de SFC não emite o diagnóstico específico da regra para nenhum dos exemplos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [Todas as regras](all.md)

### `vue/no-template-shadow`

Proibir nomes de variáveis que ocultam variáveis de um escopo externo

[Incorreto](#vue-no-template-shadow-bad) · [Correto](#vue-no-template-shadow-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

A verificação atual compara variáveis de v-for aninhados. Ela não reporta uma variável de um único v-for apenas porque compartilha o nome de uma variável do script.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-shadow": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-shadow-bad"></span>

**Incorreto**

O v-for interno declara item novamente e oculta a variável item externa dentro do laço aninhado.

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**Correto**

O laço interno declara child, deixando item disponível para a linha externa e child para a linha aninhada.

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [Todas as regras](all.md)

### `vue/no-template-target-blank`

Proibir target="_blank" sem rel="noopener noreferrer"

[Incorreto](#vue-no-template-target-blank-bad) · [Correto](#vue-no-template-target-blank-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-target-blank": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-target-blank-bad"></span>

**Incorreto**

O link externo abre um novo contexto de navegação sem a proteção rel esperada.

```vue annotate="remove:2"
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

<span id="vue-no-template-target-blank-good"></span>

**Correto**

O mesmo link inclui noopener noreferrer junto de target=_blank.

```vue annotate="add:2"
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [Todas as regras](all.md)

### `vue/no-textarea-mustache`

Proibir interpolação com chaves duplas em `<textarea>`

[Incorreto](#vue-no-textarea-mustache-bad) · [Correto](#vue-no-textarea-mustache-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-textarea-mustache-bad"></span>

**Incorreto**

O textarea coloca message em uma interpolação filha, em vez de vincular seu valor.

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**Correto**

v-model vincula o valor editável do textarea a message.

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [Todas as regras](all.md)

### `vue/no-undefined-refs`

Proibir referências a variáveis não definidas nos templates

[Incorreto](#vue-no-undefined-refs-bad) · [Correto](#vue-no-undefined-refs-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-undefined-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-undefined-refs-bad"></span>

**Incorreto**

O template lê missing, embora o script declare apenas message.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

<span id="vue-no-undefined-refs-good"></span>

**Correto**

A interpolação lê a variável message existente.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) · [Todas as regras](all.md)

### `vue/no-unsafe-url`

Alertar sobre vinculações de URL potencialmente inseguras

[Incorreto](#vue-no-unsafe-url-bad) · [Correto](#vue-no-unsafe-url-good)

Severidade padrão: `warning`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unsafe-url": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unsafe-url-bad"></span>

**Incorreto**

O destino da âncora começa com o esquema executável javascript:.

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**Correto**

A âncora usa o destino local comum de navegação /next.

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [Todas as regras](all.md)

### `vue/no-unsandboxed-iframe`

Exigir um atributo sandbox nos elementos iframe

[Incorreto](#vue-no-unsandboxed-iframe-bad) · [Correto](#vue-no-unsandboxed-iframe-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unsandboxed-iframe": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unsandboxed-iframe-bad"></span>

**Incorreto**

O frame incorporado não tem um atributo sandbox que limite suas capacidades.

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

<span id="vue-no-unsandboxed-iframe-good"></span>

**Correto**

sandbox aplica restrições; allow-scripts habilita explicitamente essa única capacidade quando necessário.

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) · [Todas as regras](all.md)

### `vue/no-unused-components`

Proibir o registro de componentes não usados nos templates

[Incorreto](#vue-no-unused-components-bad) · [Correto](#vue-no-unused-components-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-components-bad"></span>

**Incorreto**

UserAvatar é importado como componente, mas o template nunca o renderiza.

```vue annotate="remove:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

<span id="vue-no-unused-components-good"></span>

**Correto**

O template renderiza o UserAvatar importado e passa a variável user.

```vue annotate="add:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) · [Todas as regras](all.md)

### `vue/no-unused-properties`

Proibir propriedades não usadas definidas em defineProps

[Incorreto](#vue-no-unused-properties-bad) · [Correto](#vue-no-unused-properties-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-properties-bad"></span>

**Incorreto**

O componente declara description como prop, mas renderiza apenas title.

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

<span id="vue-no-unused-properties-good"></span>

**Correto**

As duas props declaradas são referenciadas pelo template.

```vue annotate="add:7"
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) · [Todas as regras](all.md)

### `vue/no-unused-refs`

Reportar refs de template (ref="x") nunca referenciadas em &lt;script&gt;

[Incorreto](#vue-no-unused-refs-bad) · [Correto](#vue-no-unused-refs-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-refs-bad"></span>

**Incorreto**

O template declara o nome de ref unused sem uma variável de referência correspondente no script.

```vue annotate="remove:1,3"
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

<span id="vue-no-unused-refs-good"></span>

**Correto**

A ref de template inputEl tem uma variável ref de mesmo nome em script setup.

```vue annotate="add:1,3,4"
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) · [Todas as regras](all.md)

### `vue/no-unused-setup-bindings`

Proibir variáveis de script setup que nunca são lidas

[Incorreto](#vue-no-unused-setup-bindings-bad) · [Correto](#vue-no-unused-setup-bindings-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-setup-bindings": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-setup-bindings-bad"></span>

**Incorreto**

A variável message de script setup nunca é lida pelo template.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

<span id="vue-no-unused-setup-bindings-good"></span>

**Correto**

O parágrafo interpola message, usando a variável declarada.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [Todas as regras](all.md)

### `vue/no-unused-vars`

Proibir definições de variáveis não usadas nas diretivas v-for e v-slot

[Incorreto](#vue-no-unused-vars-bad) · [Correto](#vue-no-unused-vars-good)

Severidade padrão: `warning`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-vars": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-vars-bad"></span>

**Incorreto**

O laço declara um index não usado e o slot declara foo sem referenciá-lo.

```vue annotate="remove:2,3,4"
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

<span id="vue-no-unused-vars-good"></span>

**Correto**

Os exemplos usam index ou o marcam como intencionalmente não usado por meio de _index, e o slot renderiza data. Chaves de índice são apenas um exemplo de uso aqui, e não uma recomendação para manter a identidade estável dos itens.

```vue annotate="add:2,3,4,5"
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) · [Todas as regras](all.md)

### `vue/no-use-v-else-with-v-for`

Proibir `v-else-if` ou `v-else` no mesmo elemento que `v-for`

[Incorreto](#vue-no-use-v-else-with-v-for-bad) · [Correto](#vue-no-use-v-else-with-v-for-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-use-v-else-with-v-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-use-v-else-with-v-for-bad"></span>

**Incorreto**

O ramo else e a iteração v-for estão associados ao mesmo parágrafo.

```vue annotate="remove:3"
<template>
<p v-if="ready">Ready</p>
<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

<span id="vue-no-use-v-else-with-v-for-good"></span>

**Correto**

Um template separado contém v-else, e seu parágrafo filho contém v-for.

```vue annotate="add:3"
<template>
<p v-if="ready">Ready</p>
<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) · [Todas as regras](all.md)

### `vue/no-use-v-if-with-v-for`

Proibir `v-if` no mesmo elemento que `v-for`

[Incorreto](#vue-no-use-v-if-with-v-for-bad) · [Correto](#vue-no-use-v-if-with-v-for-good)

Severidade padrão: `warning`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-use-v-if-with-v-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-use-v-if-with-v-for-bad"></span>

**Incorreto**

O mesmo elemento de lista combina v-if e v-for e testa a visibilidade por meio da variável do laço.

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**Correto**

Uma coleção computada filtra os itens visíveis antes que o template itere sobre eles.

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) · [Todas as regras](all.md)

### `vue/no-useless-mustaches`

Proibir interpolação com chaves duplas cuja expressão seja uma string literal constante

[Incorreto](#vue-no-useless-mustaches-bad) · [Correto](#vue-no-useless-mustaches-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-useless-mustaches": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-mustaches-bad"></span>

**Incorreto**

A interpolação contém apenas uma string constante e não precisa avaliar uma expressão.

```vue annotate="remove:2,3,4"
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

<span id="vue-no-useless-mustaches-good"></span>

**Correto**

O texto literal é escrito diretamente; expressões com variáveis, strings de template interpoladas e espaços separadores intencionais continuam sendo casos de interpolação.

```vue annotate="add:2,3,4,5"
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) · [Todas as regras](all.md)

### `vue/no-useless-template-attributes`

Proibir atributos sem efeito em elementos `<template>`

[Incorreto](#vue-no-useless-template-attributes-bad) · [Correto](#vue-no-useless-template-attributes-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-useless-template-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-template-attributes-bad"></span>

**Incorreto**

O template condicional tem uma class, mas esse invólucro estrutural não renderiza um elemento DOM para recebê-la.

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**Correto**

A class passa para o parágrafo que é realmente renderizado, enquanto v-if permanece no template estrutural.

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [Todas as regras](all.md)

### `vue/no-useless-v-bind`

Proibir um v-bind cujo valor seja uma string literal simples

[Incorreto](#vue-no-useless-v-bind-bad) · [Correto](#vue-no-useless-v-bind-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-useless-v-bind": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-v-bind-bad"></span>

**Incorreto**

A vinculação foo avalia uma string constante entre aspas ou uma string de template sem interpolação.

```vue annotate="remove:2,3"
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

<span id="vue-no-useless-v-bind-good"></span>

**Correto**

O valor constante vira um atributo estático; valores com variáveis e interpolações mantêm sua vinculação.

```vue annotate="add:2,3,4"
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [Todas as regras](all.md)

### `vue/no-v-for-template-key-on-child`

Proibir `key` no filho de um `<template v-for>`

[Incorreto](#vue-no-v-for-template-key-on-child-bad) · [Correto](#vue-no-v-for-template-key-on-child-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-for-template-key-on-child": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-for-template-key-on-child-bad"></span>

**Incorreto**

O parágrafo filho tem a key, enquanto a própria iteração template não tem chave.

```vue annotate="remove:2"
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

<span id="vue-no-v-for-template-key-on-child-good"></span>

**Correto**

A key passa para template v-for, identificando o fragmento repetido completo.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) · [Todas as regras](all.md)

### `vue/no-v-html`

Alertar sobre v-html para prevenir vulnerabilidades XSS

[Incorreto](#vue-no-v-html-bad) · [Correto](#vue-no-v-html-good)

Severidade padrão: `warning`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-html": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-html-bad"></span>

**Incorreto**

v-html interpreta content como HTML, em vez de texto comum.

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**Correto**

A interpolação com chaves duplas exibe content como texto escapado, em vez de injetar HTML.

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [Todas as regras](all.md)

### `vue/no-v-text`

Proibir a diretiva v-text; preferir interpolação com chaves duplas

[Incorreto](#vue-no-v-text-bad) · [Correto](#vue-no-v-text-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-text-bad"></span>

**Incorreto**

O conteúdo da div é fornecido pela diretiva v-text.

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

<span id="vue-no-v-text-good"></span>

**Correto**

A interpolação com chaves duplas expressa a mesma vinculação de texto diretamente no conteúdo do elemento.

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) · [Todas as regras](all.md)

### `vue/no-v-text-v-html-on-component`

Proibir v-text / v-html em elementos de componente

[Incorreto](#vue-no-v-text-v-html-on-component-bad) · [Correto](#vue-no-v-text-v-html-on-component-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-text-v-html-on-component": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-text-v-html-on-component-bad"></span>

**Incorreto**

A tag do componente recebe v-html ou v-text, que substitui o conteúdo do elemento em vez de fornecer slots ao componente.

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**Correto**

Elementos HTML nativos podem receber as diretivas; MyComponent recebe seu conteúdo pelo slot padrão.

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [Todas as regras](all.md)

### `vue/permitted-contents`

Aplicar as regras do modelo de conteúdo HTML

[Incorreto](#vue-permitted-contents-bad) · [Correto](#vue-permitted-contents-good)

Severidade padrão: `error`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/permitted-contents": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-permitted-contents-bad"></span>

**Incorreto**

Os exemplos colocam conteúdo em bloco dentro de p, omitem o corpo da tabela, aninham controles interativos ou colocam uma div diretamente dentro de ul.

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

<span id="vue-permitted-contents-good"></span>

**Correto**

Os exemplos usam conteúdo inline no parágrafo, um tbody explícito e filhos li. O componente personalizado MyItem não é tratado como um filho nativo conhecido de ul.

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [Todas as regras](all.md)

### `vue/prefer-props-shorthand`

Recomendar sintaxe abreviada para props (Vue 3.4+)

[Incorreto](#vue-prefer-props-shorthand-bad) · [Correto](#vue-prefer-props-shorthand-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prefer-props-shorthand": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prefer-props-shorthand-bad"></span>

**Incorreto**

Cada vinculação repete o nome da variável correspondente, inclusive o equivalente camelCase de um argumento separado por hífen.

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

<span id="vue-prefer-props-shorthand-good"></span>

**Correto**

A forma abreviada de vinculação de mesmo nome do Vue 3.4+ remove as expressões repetidas; uma variável de origem diferente, como bar, permanece explícita.

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) · [Todas as regras](all.md)

### `vue/prefer-true-attribute-shorthand`

Preferir a forma abreviada para um atributo booleano vinculado a `true`

[Incorreto](#vue-prefer-true-attribute-shorthand-bad) · [Correto](#vue-prefer-true-attribute-shorthand-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prefer-true-attribute-shorthand": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prefer-true-attribute-shorthand-bad"></span>

**Incorreto**

Um atributo booleano nativo disabled vincula o valor constante true.

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

<span id="vue-prefer-true-attribute-shorthand-good"></span>

**Correto**

O atributo nativo usa sua forma booleana abreviada. Vinculações false e props de componentes mantêm seus valores explícitos.

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) · [Todas as regras](all.md)

### `vue/prop-name-casing`

Aplicar um padrão de maiúsculas e minúsculas aos nomes de props declaradas

[Incorreto](#vue-prop-name-casing-bad) · [Correto](#vue-prop-name-casing-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Verifica os nomes de props declaradas, e não o padrão de maiúsculas e minúsculas dos atributos passados a um filho.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prop-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prop-name-casing-bad"></span>

**Incorreto**

O nome de prop declarado user_name usa uma grafia separada por sublinhado.

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**Correto**

A declaração e sua referência no template usam o nome camelCase userName.

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [Todas as regras](all.md)

### `vue/require-component-is`

Exigir `v-bind:is` em elementos `<component>`

[Incorreto](#vue-require-component-is-bad) · [Correto](#vue-require-component-is-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-component-is-bad"></span>

**Incorreto**

O `<component>` dinâmico não tem um destino `is`, então o Vue não pode escolher um componente para renderizar.

```vue annotate="remove:2"
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**Correto**

`:is="currentComponent"` fornece a seleção do componente; a vinculação pode mudar durante a execução.

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [Todas as regras](all.md)

### `vue/require-component-registration`

Exigir importação ou registro explícito de componentes

[Incorreto](#vue-require-component-registration-bad) · [Correto](#vue-require-component-registration-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

Liste nomes explícitos de componentes fornecidos por plugins da aplicação ou pelo previewSetup do Musea. Grafias PascalCase e kebab-case são aceitas; expressões regulares não são interpretadas. As opções não ativam a regra. Camadas posteriores substituem a lista; uma lista vazia remove os nomes herdados.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-registration": "warn"
      },
      "ruleOptions": {
        "vue/require-component-registration": {
          "globals": [
            "MyButton",
            "MyIcon"
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-component-registration-bad"></span>

**Incorreto**

`MissingWidget` não está registrado nem incluído na lista configurada de componentes globais permitidos.

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**Correto**

`MyButton` consta na opção `globals` do exemplo. Essa opção isenta um componente global conhecido; ela não o registra nem o importa.

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [Todas as regras](all.md)

### `vue/require-scoped-style`

Exigir o atributo scoped nas tags style

[Incorreto](#vue-require-scoped-style-bad) · [Correto](#vue-require-scoped-style-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-scoped-style-bad"></span>

**Incorreto**

O estilo `.button` não tem escopo e pode afetar elementos correspondentes fora deste componente.

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**Correto**

Adicionar `scoped` aplica o escopo de componente do Vue ao mesmo seletor e às mesmas declarações.

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [Todas as regras](all.md)

### `vue/require-toggle-inside-transition`

Exigir uma alternância no elemento envolvido por `<transition>`

[Incorreto](#vue-require-toggle-inside-transition-bad) · [Correto](#vue-require-toggle-inside-transition-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-toggle-inside-transition": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-toggle-inside-transition-bad"></span>

**Incorreto**

O filho estático dentro de `<Transition>` não tem visibilidade condicional nem seleção dinâmica para provocar uma mudança de entrada ou saída.

```vue annotate="remove:3"
<template>
<transition>
  <div>content</div>
</transition>
</template>
```

<span id="vue-require-toggle-inside-transition-good"></span>

**Correto**

`v-if="show"` altera a existência do filho, fornecendo um limite de entrada ou saída à transição.

```vue annotate="add:3"
<template>
<transition>
  <div v-if="show">content</div>
</transition>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) · [Todas as regras](all.md)

### `vue/require-v-for-key`

Exigir `v-bind:key` nas diretivas `v-for`

[Incorreto](#vue-require-v-for-key-bad) · [Correto](#vue-require-v-for-key-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-v-for-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-v-for-key-bad"></span>

**Incorreto**

Cada `<li>` repetido não tem uma chave que identifique seu item correspondente durante atualizações da lista.

```vue annotate="remove:2"
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**Correto**

`:key="item.id"` dá a cada nó repetido a identidade do item, em vez de sua posição atual.

```vue annotate="add:2"
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) · [Todas as regras](all.md)

### `vue/scoped-event-names`

Recomendar nomes de eventos com escopo no formato context:event

[Incorreto](#vue-scoped-event-names-bad) · [Correto](#vue-scoped-event-names-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/scoped-event-names": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-scoped-event-names-bad"></span>

**Incorreto**

`playAudio`, `pauseAudio` e `reloadAudio` codificam seu escopo como sufixos camelCase, em vez de seguir a convenção de eventos separados por dois-pontos da regra.

```vue annotate="remove:3,4,5"
<template>
  <AudioPlayer
    @playAudio="play"
    @pauseAudio="pause"
    @reloadAudio="reload"
  />
</template>
```

<span id="vue-scoped-event-names-good"></span>

**Correto**

`audio:play`, `audio:pause` e `audio:reload` compartilham um escopo explícito `audio:`. O componente emissor deve usar os mesmos nomes.

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) · [Todas as regras](all.md)

### `vue/sfc-element-order`

Aplicar uma ordem consistente aos elementos de nível superior dos SFCs

[Incorreto](#vue-sfc-element-order-bad) · [Correto](#vue-sfc-element-order-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/sfc-element-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-sfc-element-order-bad"></span>

**Incorreto**

O bloco style precede o bloco script, contrariando a ordem configurada dos blocos do SFC.

```vue annotate="remove:2,6,7,8"
<style scoped>
.panel {
  color: red;
}
</style>
<script setup lang="ts">
const label = "Save";
</script>
```

<span id="vue-sfc-element-order-good"></span>

**Correto**

Os blocos seguem script → template → style. Projetos podem escolher outra ordem pela opção tipada desta regra.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts">
const label = "Save";
</script>

<template>
  <p>{{ label }}</p>
</template>

<style scoped>
p {
  color: red;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) · [Todas as regras](all.md)

### `vue/single-style-block`

Recomendar um único bloco style

[Incorreto](#vue-single-style-block-bad) · [Correto](#vue-single-style-block-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/single-style-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-single-style-block-bad"></span>

**Incorreto**

O componente divide seus estilos com escopo de panel e title entre dois blocos style.

```vue annotate="remove:5,6,7"
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

<span id="vue-single-style-block-good"></span>

**Correto**

Os dois seletores mantêm o escopo em um único bloco style, atendendo à convenção de bloco único sem remover nenhum estilo.

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) · [Todas as regras](all.md)

### `vue/slot-name-casing`

Exigir kebab-case nos slots nomeados usados por v-slot

[Incorreto](#vue-slot-name-casing-bad) · [Correto](#vue-slot-name-casing-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/slot-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-slot-name-casing-bad"></span>

**Incorreto**

O slot nomeado `mySlot` usa camelCase onde a regra exige um nome separado por hífen.

```vue annotate="remove:2"
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

<span id="vue-slot-name-casing-good"></span>

**Correto**

`#my-slot` usa kebab-case. Renomeie o ponto de inserção do slot correspondente para o mesmo nome.

```vue annotate="add:2"
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [Todas as regras](all.md)

### `vue/this-in-template`

Proibir `this.` nas expressões do template

[Incorreto](#vue-this-in-template-bad) · [Correto](#vue-this-in-template-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/this-in-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-this-in-template-bad"></span>

**Incorreto**

As expressões do template acessam explicitamente `this.message`, `this.className` e `this.handleClick`, embora o Vue exponha essas variáveis diretamente.

```vue annotate="remove:2,3,4"
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

<span id="vue-this-in-template-good"></span>

**Correto**

Use `message`, `className` e `handleClick` diretamente. A string literal `'this.is.a.string'` permanece igual porque não é um acesso a membro.

```vue annotate="add:2,3,4,5"
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) · [Todas as regras](all.md)

### `vue/use-unique-element-ids`

Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos

[Incorreto](#vue-use-unique-element-ids-bad) · [Correto](#vue-use-unique-element-ids-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-unique-element-ids": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-unique-element-ids-bad"></span>

**Incorreto**

O ID literal `email` é reutilizado por todas as instâncias deste componente, o que pode direcionar seu rótulo ao elemento errado quando várias instâncias são renderizadas.

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**Correto**

`useId()` produz o `emailId` da instância; vincule o mesmo valor ao `for` do rótulo e ao `id` do input.

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [Todas as regras](all.md)

### `vue/use-v-on-exact`

Exigir o modificador `.exact` em `v-on` quando houver manipuladores baseados em modificadores

[Incorreto](#vue-use-v-on-exact-bad) · [Correto](#vue-use-v-on-exact-good)

Severidade padrão: `warning`  
Predefinições: `essential`, `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-v-on-exact-bad"></span>

**Incorreto**

O manipulador de clique comum também pode executar em Ctrl-clique, sobrepondo-se ao manipulador separado `.ctrl`.

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**Correto**

`.exact` limita o manipulador de clique comum a cliques sem teclas modificadoras; o manipulador específico de Ctrl permanece separado.

```vue annotate="add:2,3,4,5,6"
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) · [Todas as regras](all.md)

### `vue/v-bind-style`

Aplicar um padrão de sintaxe à diretiva `v-bind`

[Incorreto](#vue-v-bind-style-bad) · [Correto](#vue-v-bind-style-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-bind-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-bind-style-bad"></span>

**Incorreto**

`v-bind:class` usa a forma longa onde o padrão de vinculação configurado exige a forma abreviada com dois-pontos.

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**Correto**

`:class` mantém a mesma expressão com a forma abreviada exigida; esta regra trata da grafia, e não do tipo do valor.

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [Todas as regras](all.md)

### `vue/v-on-event-hyphenation`

Exigir hífens nos nomes de eventos personalizados em v-on de componentes

[Incorreto](#vue-v-on-event-hyphenation-bad) · [Correto](#vue-v-on-event-hyphenation-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-event-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-event-hyphenation-bad"></span>

**Incorreto**

O listener do componente personalizado usa `@myEvent` em vez de um nome de evento separado por hífen.

```vue annotate="remove:2,3"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
</template>
```

<span id="vue-v-on-event-hyphenation-good"></span>

**Correto**

`@my-event` usa a grafia exigida para eventos personalizados. Os listeners em elementos nativos e os argumentos de evento dinâmicos mostrados abaixo ficam fora desta verificação.

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) · [Todas as regras](all.md)

### `vue/v-on-handler-style`

Exigir manipuladores de v-on escritos como referência a método ou função inline

[Incorreto](#vue-v-on-handler-style-bad) · [Correto](#vue-v-on-handler-style-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-handler-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-handler-style-bad"></span>

**Incorreto**

Os manipuladores colocam mutações e várias instruções diretamente no atributo de evento.

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

<span id="vue-v-on-handler-style-good"></span>

**Correto**

Use uma referência a manipulador ou uma expressão de função, comum ou de seta, quando for necessária lógica inline. O limite da função torna explícita a forma do manipulador.

```vue annotate="add:2,3,4,5"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) · [Todas as regras](all.md)

### `vue/v-on-style`

Aplicar um padrão de sintaxe à diretiva `v-on`

[Incorreto](#vue-v-on-style-bad) · [Correto](#vue-v-on-style-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-style-bad"></span>

**Incorreto**

`v-on:click` usa a forma longa de listener de evento onde a regra exige a forma abreviada.

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**Correto**

`@click` mantém o mesmo manipulador e usa a forma abreviada configurada.

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [Todas as regras](all.md)

### `vue/v-slot-style`

Aplicar um padrão de sintaxe à diretiva `v-slot`

[Incorreto](#vue-v-slot-style-bad) · [Correto](#vue-v-slot-style-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-slot-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-slot-style-bad"></span>

**Incorreto**

O componente usa `#default` e o template usa `v-slot:header`, contrariando os padrões da regra para cada contexto.

```vue annotate="remove:2,4"
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

<span id="vue-v-slot-style-good"></span>

**Correto**

Use `v-slot` para o slot padrão do componente e `#header` para o slot nomeado do template.

```vue annotate="add:2,4"
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [Todas as regras](all.md)

### `vue/valid-attribute-name`

Exigir nomes de atributos válidos

[Incorreto](#vue-valid-attribute-name-bad) · [Correto](#vue-valid-attribute-name-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Diagnóstico do exemplo incorreto: `parser/template`

A grafia malformada de um atributo é diagnosticada por parser/template antes que esta regra defensiva veja um atributo. Portanto, Bad reporta parser/template; não promete uma ocorrência separada de vue/valid-attribute-name.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-attribute-name": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-attribute-name-bad"></span>

**Incorreto**

A aspa dentro de `my"attr` torna o nome do atributo malformado. Este exemplo produz o diagnóstico `parser/template` do parser, sem prometer um diagnóstico separado da regra.

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**Correto**

`my-attr` é um nome de atributo bem formado, então o parser do template pode ler o atributo e seu valor.

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [Todas as regras](all.md)

### `vue/valid-template-root`

Exigir uma raiz `<template>` válida para a semântica de fragmentos do Vue 3

[Incorreto](#vue-valid-template-root-bad) · [Correto](#vue-valid-template-root-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-template-root": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-template-root-bad"></span>

**Incorreto**

Um `<template>` aninhado comum ocupa a raiz do template sem uma diretiva que lhe dê uma função de renderização.

```vue annotate="remove:2"
<template>
  <template>content</template>
</template>
```

<span id="vue-valid-template-root-good"></span>

**Correto**

A `<div>` é um elemento raiz renderizável. Este exemplo não impõe uma restrição universal de raiz única aos fragmentos do Vue 3.

```vue annotate="add:2"
<template>
  <div>content</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [Todas as regras](all.md)

### `vue/valid-v-bind`

Exigir diretivas `v-bind` válidas

[Incorreto](#vue-valid-v-bind-bad) · [Correto](#vue-valid-v-bind-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-bind": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-bind-bad"></span>

**Incorreto**

O `v-bind` sem argumento não tem uma expressão de objeto, e a forma com argumento vazio não tem um nome de atributo.

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

<span id="vue-valid-v-bind-good"></span>

**Correto**

Forneça um atributo e uma expressão, vincule um objeto ou use a forma abreviada de mesmo nome do Vue 3.4+, como `:loading`.

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) · [Todas as regras](all.md)

### `vue/valid-v-cloak`

Exigir diretivas `v-cloak` válidas

[Incorreto](#vue-valid-v-cloak-bad) · [Correto](#vue-valid-v-cloak-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-cloak": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-cloak-bad"></span>

**Incorreto**

`v-cloak` recebe um valor, argumento ou modificador, embora não aceite nenhum deles.

```vue annotate="remove:2,3,4"
<template>
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

<span id="vue-valid-v-cloak-good"></span>

**Correto**

Use `v-cloak` sozinho; CSS pode ocultar o elemento até que o Vue remova esse atributo após a montagem.

```vue annotate="add:2"
<template>
<div v-cloak></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) · [Todas as regras](all.md)

### `vue/valid-v-else`

Exigir diretivas `v-else` válidas

[Incorreto](#vue-valid-v-else-bad) · [Correto](#vue-valid-v-else-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-else": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-else-bad"></span>

**Incorreto**

Os exemplos atribuem uma expressão a `v-else`, combinam-no com `v-if` ou omitem o ramo condicional adjacente que deve precedê-lo.

```vue annotate="remove:2,3"
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

<span id="vue-valid-v-else-good"></span>

**Correto**

Coloque `v-else` sozinho imediatamente após o ramo `v-if` correspondente.

```vue annotate="add:2"
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) · [Todas as regras](all.md)

### `vue/valid-v-for`

Exigir diretivas `v-for` válidas

[Incorreto](#vue-valid-v-for-bad) · [Correto](#vue-valid-v-for-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-for": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-for-bad"></span>

**Incorreto**

Os laços omitem a expressão de iteração ou adicionam o modificador `.stop`, que não é aceito.

```vue annotate="remove:2,3,4"
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

<span id="vue-valid-v-for-good"></span>

**Correto**

Use `item in items` ou `(item, index) of items` com uma expressão de iteração completa e as chaves mostradas.

```vue annotate="add:2,3"
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) · [Todas as regras](all.md)

### `vue/valid-v-html`

Exigir diretivas `v-html` válidas

[Incorreto](#vue-valid-v-html-bad) · [Correto](#vue-valid-v-html-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-html": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-html-bad"></span>

**Incorreto**

`v-html` não tem sua expressão ou usa um argumento ou modificador que esta diretiva não aceita.

```vue annotate="remove:2,3,4"
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

<span id="vue-valid-v-html-good"></span>

**Correto**

`v-html="html"` fornece uma expressão válida. A validade sintática não sanitiza HTML nem torna seguro conteúdo não confiável.

```vue annotate="add:2"
<template>
<div v-html="html"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) · [Todas as regras](all.md)

### `vue/valid-v-if`

Exigir diretivas `v-if` válidas

[Incorreto](#vue-valid-v-if-bad) · [Correto](#vue-valid-v-if-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-if-bad"></span>

**Incorreto**

As condições omitem uma expressão ou combinam `v-if` com uma diretiva else no mesmo nó.

```vue annotate="remove:2,3,4"
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

<span id="vue-valid-v-if-good"></span>

**Correto**

Cada `v-if` tem uma condição não vazia, como `ready` ou `count > 0`, sem uma diretiva else incompatível.

```vue annotate="add:2,3"
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) · [Todas as regras](all.md)

### `vue/valid-v-memo`

Exigir diretivas `v-memo` válidas

[Incorreto](#vue-valid-v-memo-bad) · [Correto](#vue-valid-v-memo-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-memo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-memo-bad"></span>

**Incorreto**

`v-memo` sozinho não fornece ao Vue uma expressão de dependências para decidir quando reutilizar a subárvore.

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

<span id="vue-valid-v-memo-good"></span>

**Correto**

`v-memo="[valueA, valueB]"` fornece o array de dependências usado para memorização.

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [Todas as regras](all.md)

### `vue/valid-v-model`

Exigir diretivas `v-model` válidas

[Incorreto](#vue-valid-v-model-bad) · [Correto](#vue-valid-v-model-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-model": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-model-bad"></span>

**Incorreto**

Uma `<div>` nativa não pode usar `v-model` como controle de formulário, e uma diretiva de input sem valor não tem uma expressão de destino gravável.

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

<span id="vue-valid-v-model-good"></span>

**Correto**

Vincule o input, select, textarea ou componente personalizado às variáveis graváveis mostradas.

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) · [Todas as regras](all.md)

### `vue/valid-v-on`

Exigir diretivas `v-on` válidas

[Incorreto](#vue-valid-v-on-bad) · [Correto](#vue-valid-v-on-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-on": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-on-bad"></span>

**Incorreto**

As formas de listener omitem um argumento de evento ou a expressão obrigatória de manipulador ou objeto.

```vue annotate="remove:2,3,4"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

<span id="vue-valid-v-on-good"></span>

**Correto**

Use um evento com seu manipulador ou passe um objeto de listeners para `v-on` sem argumento.

```vue annotate="add:2,3"
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [Todas as regras](all.md)

### `vue/valid-v-once`

Exigir diretivas `v-once` válidas

[Incorreto](#vue-valid-v-once-bad) · [Correto](#vue-valid-v-once-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-once": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-once-bad"></span>

**Incorreto**

`v-once` tem um valor, argumento ou modificador, embora esta diretiva seja um marcador sem valor para renderização única.

```vue annotate="remove:2,3,4"
<template>
<div v-once="foo"></div>
<div v-once:arg></div>
<div v-once.mod></div>
</template>
```

<span id="vue-valid-v-once-good"></span>

**Correto**

`v-once` sozinho marca a subárvore para renderização única, sem sintaxe não suportada.

```vue annotate="add:2"
<template>
<div v-once></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) · [Todas as regras](all.md)

### `vue/valid-v-show`

Exigir diretivas `v-show` válidas

[Incorreto](#vue-valid-v-show-bad) · [Correto](#vue-valid-v-show-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-show": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-show-bad"></span>

**Incorreto**

`v-show` não tem sua expressão de visibilidade ou é colocado em um `<template>` que não tem um elemento DOM cujo display possa ser alterado.

```vue annotate="remove:2,3"
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

<span id="vue-valid-v-show-good"></span>

**Correto**

Aplique a expressão de visibilidade a um elemento renderizado, como `<div>`.

```vue annotate="add:2,3"
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [Todas as regras](all.md)

### `vue/valid-v-slot`

Exigir diretivas `v-slot` válidas

[Incorreto](#vue-valid-v-slot-bad) · [Correto](#vue-valid-v-slot-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-slot": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-slot-bad"></span>

**Incorreto**

A diretiva de slot está em uma `<div>` nativa ou entra em conflito com outras declarações de slots padrão ou nomeados.

```vue annotate="remove:2,3,4"
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

<span id="vue-valid-v-slot-good"></span>

**Correto**

Declare o slot padrão de um componente nele próprio, ou seu slot nomeado em um filho `<template #header>`.

```vue annotate="add:2,3,4,5"
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) · [Todas as regras](all.md)

### `vue/valid-v-text`

Exigir diretivas `v-text` válidas

[Incorreto](#vue-valid-v-text-bad) · [Correto](#vue-valid-v-text-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-text": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-text-bad"></span>

**Incorreto**

`v-text` não tem sua expressão de texto ou usa um argumento ou modificador que não é aceito.

```vue annotate="remove:2,3,4"
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

<span id="vue-valid-v-text-good"></span>

**Correto**

`v-text="msg"` é sintaticamente válido. A regra de estilo separada `vue/no-v-text` ainda pode preferir interpolação.

```vue annotate="add:2"
<template>
<div v-text="msg"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [Todas as regras](all.md)

### `vue/warn-custom-block`

Alertar sobre blocos personalizados em arquivos SFC

[Incorreto](#vue-warn-custom-block-bad) · [Correto](#vue-warn-custom-block-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/warn-custom-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-block-bad"></span>

**Incorreto**

O SFC contém um bloco personalizado `<i18n>`, que precisa de uma integração externa além do processamento comum de template, script e style.

```vue annotate="remove:1,2,3,4"
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

<span id="vue-warn-custom-block-good"></span>

**Correto**

O exemplo usa blocos padrão de template e script setup. Este aviso opcional de portabilidade não significa que todo bloco personalizado seja inválido no Vue.

```vue annotate="add:4,5,6,7"
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [Todas as regras](all.md)

### `vue/warn-custom-directive`

Alertar sobre diretivas personalizadas que precisam de registro

[Incorreto](#vue-warn-custom-directive-bad) · [Correto](#vue-warn-custom-directive-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/warn-custom-directive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-directive-bad"></span>

**Incorreto**

`v-focus`, `v-mask` e `v-click-outside` exigem implementações de diretivas específicas do projeto, que esta convenção opcional sinaliza.

```vue annotate="remove:2,3,4"
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**Correto**

O exemplo usa as diretivas nativas `v-if`, `v-model` e `v-on`. Uma diretiva personalizada registrada corretamente ainda pode ser válida no Vue quando esta política estiver desativada.

```vue annotate="add:2,3,4"
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [Todas as regras](all.md)
