---
title: Règles Vue
---

# Règles Vue

Chaque règle Vue présente son objectif, sa configuration et ses mauvais et bons exemples sur cette page. Les lignes surlignées indiquent les modifications ; le code copié conserve la source complète.

<span id="syntaxe-et-règles-de-style"></span>

| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [Mauvais](#vue-a11y-img-alt-bad) · [Bon](#vue-a11y-img-alt-good) | Exiger un attribut alt sur les images pour les rendre accessibles |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [Mauvais](#vue-attribute-hyphenation-bad) · [Bon](#vue-attribute-hyphenation-good) | Imposer un style de nommage des attributs sur les composants personnalisés |
| [`vue/attribute-order`](#vue-attribute-order) | [Mauvais](#vue-attribute-order-bad) · [Bon](#vue-attribute-order-good) | Imposer un ordre cohérent des attributs |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [Mauvais](#vue-component-definition-name-casing-bad) · [Bon](#vue-component-definition-name-casing-good) | Imposer PascalCase ou kebab-case aux noms de définition des composants |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [Mauvais](#vue-component-name-in-template-casing-bad) · [Bon](#vue-component-name-in-template-casing-good) | Imposer une casse précise aux noms de composants dans les templates |
| [`vue/html-button-has-type`](#vue-html-button-has-type) | [Mauvais](#vue-html-button-has-type-bad) · [Bon](#vue-html-button-has-type-good) | Exiger un type explicite et valide sur les éléments button |
| [`vue/html-quotes`](#vue-html-quotes) | [Mauvais](#vue-html-quotes-bad) · [Bon](#vue-html-quotes-good) | Imposer un style de guillemets pour les attributs HTML |
| [`vue/html-self-closing`](#vue-html-self-closing) | [Mauvais](#vue-html-self-closing-bad) · [Bon](#vue-html-self-closing-good) | Imposer un style de fermeture automatique des balises |
| [`vue/max-template-complexity`](#vue-max-template-complexity) | [Mauvais](#vue-max-template-complexity-bad) · [Bon](#vue-max-template-complexity-good) | Limiter la complexité propre du template d’un composant, cyclomatique et cognitive |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [Mauvais](#vue-multi-word-component-names-bad) · [Bon](#vue-multi-word-component-names-good) | Exiger des noms de composants composés de plusieurs mots |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [Mauvais](#vue-mustache-interpolation-spacing-bad) · [Bon](#vue-mustache-interpolation-spacing-good) | Imposer un espacement cohérent dans les interpolations à doubles accolades |
| [`vue/no-array-index-key`](#vue-no-array-index-key) | [Mauvais](#vue-no-array-index-key-bad) · [Bon](#vue-no-array-index-key-good) | Interdire l’utilisation directe de la variable d’index de v-for comme :key |
| [`vue/no-bare-strings-in-template`](#vue-no-bare-strings-in-template) | [Mauvais](#vue-no-bare-strings-in-template-bad) · [Bon](#vue-no-bare-strings-in-template-good) | Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [Mauvais](#vue-no-boolean-attr-value-bad) · [Bon](#vue-no-boolean-attr-value-good) | Interdire les valeurs explicites des attributs HTML booléens |
| [`vue/no-child-content`](#vue-no-child-content) | [Mauvais](#vue-no-child-content-bad) · [Bon](#vue-no-child-content-good) | Interdire le contenu enfant lors de l’utilisation de v-html ou v-text |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [Mauvais](#vue-no-deprecated-filter-bad) · [Bon](#vue-no-deprecated-filter-good) | Interdire la syntaxe obsolète des filtres de Vue 2 utilisant l’opérateur pipe |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [Mauvais](#vue-no-deprecated-functional-template-bad) · [Bon](#vue-no-deprecated-functional-template-good) | Interdire l’attribut `functional` sur le `<template>` d’un SFC |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [Mauvais](#vue-no-deprecated-html-element-is-bad) · [Bon](#vue-no-deprecated-html-element-is-good) | Interdire l’attribut `is` sur les éléments HTML natifs |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [Mauvais](#vue-no-deprecated-inline-template-bad) · [Bon](#vue-no-deprecated-inline-template-good) | Interdire l’attribut obsolète `inline-template` |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [Mauvais](#vue-no-deprecated-router-link-tag-prop-bad) · [Bon](#vue-no-deprecated-router-link-tag-prop-good) | Interdire la prop `tag` sur &lt;router-link&gt; |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [Mauvais](#vue-no-deprecated-scope-attribute-bad) · [Bon](#vue-no-deprecated-scope-attribute-good) | Interdire l’attribut obsolète `scope` sur &lt;template&gt; |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [Mauvais](#vue-no-deprecated-slot-attribute-bad) · [Bon](#vue-no-deprecated-slot-attribute-good) | Interdire l’attribut obsolète `slot` |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [Mauvais](#vue-no-deprecated-slot-scope-attribute-bad) · [Bon](#vue-no-deprecated-slot-scope-attribute-good) | Interdire l’attribut obsolète `slot-scope` |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [Mauvais](#vue-no-deprecated-v-bind-sync-bad) · [Bon](#vue-no-deprecated-v-bind-sync-good) | Interdire le modificateur obsolète `.sync` sur `v-bind` |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [Mauvais](#vue-no-deprecated-v-on-native-modifier-bad) · [Bon](#vue-no-deprecated-v-on-native-modifier-good) | Interdire le modificateur obsolète `.native` sur `v-on` |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [Mauvais](#vue-no-deprecated-v-on-number-modifiers-bad) · [Bon](#vue-no-deprecated-v-on-number-modifiers-good) | Interdire les modificateurs numériques obsolètes `keyCode` sur `v-on` |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [Mauvais](#vue-no-dupe-v-else-if-bad) · [Bon](#vue-no-dupe-v-else-if-good) | Interdire les conditions répétées dans les chaînes `v-if` / `v-else-if` |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [Mauvais](#vue-no-duplicate-attributes-bad) · [Bon](#vue-no-duplicate-attributes-good) | Interdire les attributs en double sur un même élément |
| [`vue/no-empty-component-block`](#vue-no-empty-component-block) | [Mauvais](#vue-no-empty-component-block-bad) · [Bon](#vue-no-empty-component-block-good) | Interdire les blocs SFC vides |
| [`vue/no-inline-style`](#vue-no-inline-style) | [Mauvais](#vue-no-inline-style-bad) · [Bon](#vue-no-inline-style-good) | Déconseiller l’utilisation d’attributs de style en ligne |
| [`vue/no-invalid-html-attribute`](#vue-no-invalid-html-attribute) | [Mauvais](#vue-no-invalid-html-attribute-bad) · [Bon](#vue-no-invalid-html-attribute-good) | Interdire les valeurs statiques invalides des attributs HTML |
| [`vue/no-lone-template`](#vue-no-lone-template) | [Mauvais](#vue-no-lone-template-bad) · [Bon](#vue-no-lone-template-good) | Interdire les éléments `<template>` inutiles |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [Mauvais](#vue-no-multi-spaces-bad) · [Bon](#vue-no-multi-spaces-good) | Interdire plusieurs espaces consécutifs |
| [`vue/no-multiple-objects-in-class`](#vue-no-multiple-objects-in-class) | [Mauvais](#vue-no-multiple-objects-in-class-bad) · [Bon](#vue-no-multiple-objects-in-class-good) | Interdire plusieurs objets littéraux dans une liaison :class sous forme de tableau |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [Mauvais](#vue-no-multiple-template-root-bad) · [Bon](#vue-no-multiple-template-root-good) | Interdire plusieurs nœuds racines dans un template |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [Mauvais](#vue-no-mutating-props-bad) · [Bon](#vue-no-mutating-props-good) | Interdire la modification des props d’un composant |
| [`vue/no-negated-v-if-condition`](#vue-no-negated-v-if-condition) | [Mauvais](#vue-no-negated-v-if-condition-bad) · [Bon](#vue-no-negated-v-if-condition-good) | Interdire une condition v-if négative lorsque la chaîne comporte un v-else |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [Mauvais](#vue-no-non-component-keep-alive-child-bad) · [Bon](#vue-no-non-component-keep-alive-child-good) | Interdire les enveloppes d’éléments ordinaires directement sous `<KeepAlive>` |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [Mauvais](#vue-no-preprocessor-lang-bad) · [Bon](#vue-no-preprocessor-lang-good) | Déconseiller les préprocesseurs CSS au profit du CSS moderne |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [Mauvais](#vue-no-reserved-component-names-bad) · [Bon](#vue-no-reserved-component-names-good) | Interdire l’utilisation de noms réservés comme noms de composants |
| [`vue/no-root-v-if`](#vue-no-root-v-if) | [Mauvais](#vue-no-root-v-if-bad) · [Bon](#vue-no-root-v-if-good) | Interdire v-if sur l’unique élément racine d’un template |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [Mauvais](#vue-no-script-non-standard-lang-bad) · [Bon](#vue-no-script-non-standard-lang-good) | Déconseiller les valeurs lang non standard dans les scripts |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [Mauvais](#vue-no-src-attribute-bad) · [Bon](#vue-no-src-attribute-good) | Déconseiller l’attribut src sur les blocs SFC |
| [`vue/no-static-inline-styles`](#vue-no-static-inline-styles) | [Mauvais](#vue-no-static-inline-styles-bad) · [Bon](#vue-no-static-inline-styles-good) | Interdire les attributs de style en ligne statiques |
| [`vue/no-template-key`](#vue-no-template-key) | [Mauvais](#vue-no-template-key-bad) · [Bon](#vue-no-template-key-good) | Interdire l’attribut `key` sur `<template>` |
| [`vue/no-template-lang`](#vue-no-template-lang) | [Mauvais](#vue-no-template-lang-bad) · [Bon](#vue-no-template-lang-good) | Déconseiller l’attribut lang sur le bloc template |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [Mauvais](#vue-no-template-shadow-bad) · [Bon](#vue-no-template-shadow-good) | Interdire les noms de variables qui masquent des variables d’une portée externe |
| [`vue/no-template-target-blank`](#vue-no-template-target-blank) | [Mauvais](#vue-no-template-target-blank-bad) · [Bon](#vue-no-template-target-blank-good) | Interdire target="_blank" sans rel="noopener noreferrer" |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [Mauvais](#vue-no-textarea-mustache-bad) · [Bon](#vue-no-textarea-mustache-good) | Interdire l’interpolation à doubles accolades dans `<textarea>` |
| [`vue/no-undefined-refs`](#vue-no-undefined-refs) | [Mauvais](#vue-no-undefined-refs-bad) · [Bon](#vue-no-undefined-refs-good) | Interdire les références à des variables non définies dans les templates |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [Mauvais](#vue-no-unsafe-url-bad) · [Bon](#vue-no-unsafe-url-good) | Signaler les liaisons d’URL potentiellement dangereuses |
| [`vue/no-unsandboxed-iframe`](#vue-no-unsandboxed-iframe) | [Mauvais](#vue-no-unsandboxed-iframe-bad) · [Bon](#vue-no-unsandboxed-iframe-good) | Exiger un attribut sandbox sur les éléments iframe |
| [`vue/no-unused-components`](#vue-no-unused-components) | [Mauvais](#vue-no-unused-components-bad) · [Bon](#vue-no-unused-components-good) | Interdire l’enregistrement de composants inutilisés dans les templates |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [Mauvais](#vue-no-unused-properties-bad) · [Bon](#vue-no-unused-properties-good) | Interdire les propriétés inutilisées définies dans defineProps |
| [`vue/no-unused-refs`](#vue-no-unused-refs) | [Mauvais](#vue-no-unused-refs-bad) · [Bon](#vue-no-unused-refs-good) | Signaler les refs de template (ref="x") jamais référencées dans &lt;script&gt; |
| [`vue/no-unused-setup-bindings`](#vue-no-unused-setup-bindings) | [Mauvais](#vue-no-unused-setup-bindings-bad) · [Bon](#vue-no-unused-setup-bindings-good) | Interdire les liaisons script setup qui ne sont jamais lues |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [Mauvais](#vue-no-unused-vars-bad) · [Bon](#vue-no-unused-vars-good) | Interdire les variables inutilisées définies dans les directives v-for et v-slot |
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [Mauvais](#vue-no-use-v-else-with-v-for-bad) · [Bon](#vue-no-use-v-else-with-v-for-good) | Interdire `v-else-if` ou `v-else` sur le même élément que `v-for` |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [Mauvais](#vue-no-use-v-if-with-v-for-bad) · [Bon](#vue-no-use-v-if-with-v-for-good) | Interdire `v-if` sur le même élément que `v-for` |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [Mauvais](#vue-no-useless-mustaches-bad) · [Bon](#vue-no-useless-mustaches-good) | Interdire une interpolation à doubles accolades dont l’expression est une chaîne littérale constante |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [Mauvais](#vue-no-useless-template-attributes-bad) · [Bon](#vue-no-useless-template-attributes-good) | Interdire les attributs inutiles sur les éléments `<template>` |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [Mauvais](#vue-no-useless-v-bind-bad) · [Bon](#vue-no-useless-v-bind-good) | Interdire un v-bind dont la valeur est une simple chaîne littérale |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [Mauvais](#vue-no-v-for-template-key-on-child-bad) · [Bon](#vue-no-v-for-template-key-on-child-good) | Interdire `key` sur l’enfant d’un `<template v-for>` |
| [`vue/no-v-html`](#vue-no-v-html) | [Mauvais](#vue-no-v-html-bad) · [Bon](#vue-no-v-html-good) | Déconseiller v-html pour prévenir les vulnérabilités XSS |
| [`vue/no-v-text`](#vue-no-v-text) | [Mauvais](#vue-no-v-text-bad) · [Bon](#vue-no-v-text-good) | Interdire la directive v-text ; préférer l’interpolation à doubles accolades |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [Mauvais](#vue-no-v-text-v-html-on-component-bad) · [Bon](#vue-no-v-text-v-html-on-component-good) | Interdire v-text / v-html sur les éléments de composants |
| [`vue/permitted-contents`](#vue-permitted-contents) | [Mauvais](#vue-permitted-contents-bad) · [Bon](#vue-permitted-contents-good) | Faire respecter les règles du modèle de contenu HTML |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [Mauvais](#vue-prefer-props-shorthand-bad) · [Bon](#vue-prefer-props-shorthand-good) | Recommander la syntaxe abrégée des props (Vue 3.4+) |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [Mauvais](#vue-prefer-true-attribute-shorthand-bad) · [Bon](#vue-prefer-true-attribute-shorthand-good) | Préférer la syntaxe abrégée pour un attribut booléen lié à `true` |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [Mauvais](#vue-prop-name-casing-bad) · [Bon](#vue-prop-name-casing-good) | Imposer une casse aux noms des props déclarées |
| [`vue/require-component-is`](#vue-require-component-is) | [Mauvais](#vue-require-component-is-bad) · [Bon](#vue-require-component-is-good) | Exiger `v-bind:is` sur les éléments `<component>` |
| [`vue/require-component-registration`](#vue-require-component-registration) | [Mauvais](#vue-require-component-registration-bad) · [Bon](#vue-require-component-registration-good) | Exiger un import ou un enregistrement explicite des composants |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [Mauvais](#vue-require-scoped-style-bad) · [Bon](#vue-require-scoped-style-good) | Exiger l’attribut scoped sur les balises style |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [Mauvais](#vue-require-toggle-inside-transition-bad) · [Bon](#vue-require-toggle-inside-transition-good) | Exiger un mécanisme de bascule sur l’élément enveloppé par `<transition>` |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [Mauvais](#vue-require-v-for-key-bad) · [Bon](#vue-require-v-for-key-good) | Exiger `v-bind:key` avec les directives `v-for` |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [Mauvais](#vue-scoped-event-names-bad) · [Bon](#vue-scoped-event-names-good) | Recommander des noms d’événements préfixés par leur contexte au format context:event |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [Mauvais](#vue-sfc-element-order-bad) · [Bon](#vue-sfc-element-order-good) | Imposer un ordre cohérent des éléments de premier niveau d’un SFC |
| [`vue/single-style-block`](#vue-single-style-block) | [Mauvais](#vue-single-style-block-bad) · [Bon](#vue-single-style-block-good) | Recommander un seul bloc style |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [Mauvais](#vue-slot-name-casing-bad) · [Bon](#vue-slot-name-casing-good) | Imposer kebab-case aux slots nommés utilisés avec v-slot |
| [`vue/this-in-template`](#vue-this-in-template) | [Mauvais](#vue-this-in-template-bad) · [Bon](#vue-this-in-template-good) | Interdire `this.` dans les expressions des templates |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Mauvais](#vue-use-unique-element-ids-bad) · [Bon](#vue-use-unique-element-ids-good) | Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [Mauvais](#vue-use-v-on-exact-bad) · [Bon](#vue-use-v-on-exact-good) | Imposer le modificateur `.exact` sur `v-on` en présence de gestionnaires utilisant des modificateurs |
| [`vue/v-bind-style`](#vue-v-bind-style) | [Mauvais](#vue-v-bind-style-bad) · [Bon](#vue-v-bind-style-good) | Imposer un style de directive `v-bind` |
| [`vue/v-on-event-hyphenation`](#vue-v-on-event-hyphenation) | [Mauvais](#vue-v-on-event-hyphenation-bad) · [Bon](#vue-v-on-event-hyphenation-good) | Imposer des tirets dans les noms d’événements personnalisés de v-on sur les composants |
| [`vue/v-on-handler-style`](#vue-v-on-handler-style) | [Mauvais](#vue-v-on-handler-style-bad) · [Bon](#vue-v-on-handler-style-good) | Imposer une référence de méthode ou une fonction en ligne pour les gestionnaires v-on |
| [`vue/v-on-style`](#vue-v-on-style) | [Mauvais](#vue-v-on-style-bad) · [Bon](#vue-v-on-style-good) | Imposer un style de directive `v-on` |
| [`vue/v-slot-style`](#vue-v-slot-style) | [Mauvais](#vue-v-slot-style-bad) · [Bon](#vue-v-slot-style-good) | Imposer un style de directive `v-slot` |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [Mauvais](#vue-valid-attribute-name-bad) · [Bon](#vue-valid-attribute-name-good) | Exiger des noms d’attributs valides |
| [`vue/valid-template-root`](#vue-valid-template-root) | [Mauvais](#vue-valid-template-root-bad) · [Bon](#vue-valid-template-root-good) | Exiger une racine `<template>` valide selon la sémantique des fragments de Vue 3 |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [Mauvais](#vue-valid-v-bind-bad) · [Bon](#vue-valid-v-bind-good) | Exiger des directives `v-bind` valides |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [Mauvais](#vue-valid-v-cloak-bad) · [Bon](#vue-valid-v-cloak-good) | Exiger des directives `v-cloak` valides |
| [`vue/valid-v-else`](#vue-valid-v-else) | [Mauvais](#vue-valid-v-else-bad) · [Bon](#vue-valid-v-else-good) | Exiger des directives `v-else` valides |
| [`vue/valid-v-for`](#vue-valid-v-for) | [Mauvais](#vue-valid-v-for-bad) · [Bon](#vue-valid-v-for-good) | Exiger des directives `v-for` valides |
| [`vue/valid-v-html`](#vue-valid-v-html) | [Mauvais](#vue-valid-v-html-bad) · [Bon](#vue-valid-v-html-good) | Exiger des directives `v-html` valides |
| [`vue/valid-v-if`](#vue-valid-v-if) | [Mauvais](#vue-valid-v-if-bad) · [Bon](#vue-valid-v-if-good) | Exiger des directives `v-if` valides |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [Mauvais](#vue-valid-v-memo-bad) · [Bon](#vue-valid-v-memo-good) | Exiger des directives `v-memo` valides |
| [`vue/valid-v-model`](#vue-valid-v-model) | [Mauvais](#vue-valid-v-model-bad) · [Bon](#vue-valid-v-model-good) | Exiger des directives `v-model` valides |
| [`vue/valid-v-on`](#vue-valid-v-on) | [Mauvais](#vue-valid-v-on-bad) · [Bon](#vue-valid-v-on-good) | Exiger des directives `v-on` valides |
| [`vue/valid-v-once`](#vue-valid-v-once) | [Mauvais](#vue-valid-v-once-bad) · [Bon](#vue-valid-v-once-good) | Exiger des directives `v-once` valides |
| [`vue/valid-v-show`](#vue-valid-v-show) | [Mauvais](#vue-valid-v-show-bad) · [Bon](#vue-valid-v-show-good) | Exiger des directives `v-show` valides |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [Mauvais](#vue-valid-v-slot-bad) · [Bon](#vue-valid-v-slot-good) | Exiger des directives `v-slot` valides |
| [`vue/valid-v-text`](#vue-valid-v-text) | [Mauvais](#vue-valid-v-text-bad) · [Bon](#vue-valid-v-text-good) | Exiger des directives `v-text` valides |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [Mauvais](#vue-warn-custom-block-bad) · [Bon](#vue-warn-custom-block-good) | Signaler les blocs personnalisés dans les fichiers SFC |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [Mauvais](#vue-warn-custom-directive-bad) · [Bon](#vue-warn-custom-directive-good) | Signaler les directives personnalisées nécessitant un enregistrement |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `vue/a11y-img-alt`

Exiger un attribut alt sur les images pour les rendre accessibles

[Mauvais](#vue-a11y-img-alt-bad) · [Bon](#vue-a11y-img-alt-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Ni l’image statique ni l’image dont la source est dynamique ne fournit d’attribut alt.

```vue annotate="remove:2,3"
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

<span id="vue-a11y-img-alt-good"></span>

**Bon**

Les images informatives reçoivent un texte alt descriptif, les images décoratives un alt vide, et l’image dynamique lie sa description.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) · [Toutes les règles](all.md)

### `vue/attribute-hyphenation`

Imposer un style de nommage des attributs sur les composants personnalisés

[Mauvais](#vue-attribute-hyphenation-bad) · [Bon](#vue-attribute-hyphenation-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

L’attribut du composant utilise la graphie camelCase firstName.

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**Bon**

La graphie first-name respecte la convention configurée qui sépare les mots des attributs de composants par des tirets.

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [Toutes les règles](all.md)

### `vue/attribute-order`

Imposer un ordre cohérent des attributs

[Mauvais](#vue-attribute-order-bad) · [Bon](#vue-attribute-order-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le gestionnaire d’événement apparaît avant la directive structurelle v-if et l’attribut ordinaire id.

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**Bon**

v-if vient en premier, suivi de id puis du gestionnaire d’événement, conformément à l’ordre défini par la règle.

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [Toutes les règles](all.md)

### `vue/component-definition-name-casing`

Imposer PascalCase ou kebab-case aux noms de définition des composants

[Mauvais](#vue-component-definition-name-casing-bad) · [Bon](#vue-component-definition-name-casing-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Le nom de fichier du composant est contrôlé. PascalCase et kebab-case sont acceptés ; une casse mixte est signalée.

**Configuration (Vite+)**

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

**Mauvais**

Le nom de fichier myComponent.vue combine une initiale minuscule et une majuscule interne au lieu d’utiliser PascalCase ou kebab-case.

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

<span id="vue-component-definition-name-casing-good"></span>

**Bon**

Renommer le fichier en MyComponent.vue adopte PascalCase ; le contenu de son template reste inchangé.

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [Toutes les règles](all.md)

### `vue/component-name-in-template-casing`

Imposer une casse précise aux noms de composants dans les templates

[Mauvais](#vue-component-name-in-template-casing-bad) · [Bon](#vue-component-name-in-template-casing-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

Le composant est écrit en kebab-case et en camelCase alors que la convention impose PascalCase.

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

**Bon**

MyComponent utilise PascalCase ; la syntaxe native slot reste en minuscules.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) · [Toutes les règles](all.md)

### `vue/html-button-has-type`

Exiger un type explicite et valide sur les éléments button

[Mauvais](#vue-html-button-has-type-bad) · [Bon](#vue-html-button-has-type-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un bouton omet type et un autre fournit le type foo, qui n’est pas pris en charge.

```vue annotate="remove:2,3"
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

<span id="vue-html-button-has-type-good"></span>

**Bon**

Les boutons précisent button, submit ou reset ; un type lié est considéré comme dynamique.

```vue annotate="add:2,3,4,5"
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) · [Toutes les règles](all.md)

### `vue/html-quotes`

Imposer un style de guillemets pour les attributs HTML

[Mauvais](#vue-html-quotes-bad) · [Bon](#vue-html-quotes-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les attributs utilisent des apostrophes ou aucun guillemet au lieu des guillemets doubles exigés par la convention.

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**Bon**

Les attributs ordinaires et les expressions des directives utilisent des guillemets doubles.

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [Toutes les règles](all.md)

### `vue/html-self-closing`

Imposer un style de fermeture automatique des balises

[Mauvais](#vue-html-self-closing-bad) · [Bon](#vue-html-self-closing-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

Le composant vide utilise une paire de balises, tandis que les éléments vides img et br omettent la syntaxe autofermante configurée.

```vue annotate="remove:2,3,4"
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**Bon**

Le composant et les éléments vides utilisent une syntaxe autofermante ; un div contenant du contenu conserve sa balise de fermeture.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [Toutes les règles](all.md)

### `vue/max-template-complexity`

Limiter la complexité propre du template d’un composant, cyclomatique et cognitive

[Mauvais](#vue-max-template-complexity-bad) · [Bon](#vue-max-template-complexity-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

L’exemple Mauvais a une complexité cyclomatique de 13 et une complexité cognitive de 25 (limites : 11 et 16). Chaque composant est mesuré séparément ; seuls les templates HTML intégrés sont pris en charge.

Consultez [le calcul de la complexité et les limites des composants](../guide/cross-file-complexity.md) pour connaître les contributions aux deux scores de l’exemple.

**Configuration (Vite+)**

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

**Mauvais**

Les branches, la boucle, le contenu des slots et les décisions dans les expressions écrits dans le parent produisent des scores de 13 et 25, supérieurs aux limites par défaut de 11 et 16.

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

**Bon**

Le template parent délègue le rendu à RowList et conserve un seul v-if ; ses propres scores sont de 2 et 1.

```vue annotate="add:2"
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) · [Toutes les règles](all.md)

### `vue/multi-word-component-names`

Exiger des noms de composants composés de plusieurs mots

[Mauvais](#vue-multi-word-component-names-bad) · [Bon](#vue-multi-word-component-names-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Le diagnostic concerne le nom de fichier. Renommez ce même composant ; modifier une balise enfant ne corrige pas le problème.

**Configuration (Vite+)**

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

**Mauvais**

Item.vue donne au composant un nom composé d’un seul mot.

`Item.vue`

```vue
<template><p>Item</p></template>
```

<span id="vue-multi-word-component-names-good"></span>

**Bon**

TodoItem.vue donne au même template un nom de composant composé de plusieurs mots.

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [Toutes les règles](all.md)

### `vue/mustache-interpolation-spacing`

Imposer un espacement cohérent dans les interpolations à doubles accolades

[Mauvais](#vue-mustache-interpolation-spacing-bad) · [Bon](#vue-mustache-interpolation-spacing-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Il manque un espace à une ou aux deux limites des délimiteurs de l’interpolation de texte.

```vue annotate="remove:2,3,4"
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**Bon**

Des espaces séparent l’expression des délimiteurs d’ouverture et de fermeture à doubles accolades.

```vue annotate="add:2,3,4"
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [Toutes les règles](all.md)

### `vue/no-array-index-key`

Interdire l’utilisation directe de la variable d’index de v-for comme :key

[Mauvais](#vue-no-array-index-key-bad) · [Bon](#vue-no-array-index-key-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La clé de la liste est son index actuel ; l’identité de l’élément change donc lorsque la liste est réordonnée.

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

<span id="vue-no-array-index-key-good"></span>

**Bon**

La clé provient de item.id, ce qui préserve l’identité de chaque élément lorsqu’il change de position.

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) · [Toutes les règles](all.md)

### `vue/no-bare-strings-in-template`

Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé

[Mauvais](#vue-no-bare-strings-in-template-bad) · [Bon](#vue-no-bare-strings-in-template-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le texte visible et les attributs de nommage contiennent des chaînes non traduites directement dans le template.

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

<span id="vue-no-bare-strings-in-template-good"></span>

**Bon**

Le contenu traduisible appelle $t ; les exemples de ponctuation et de valeurs uniquement numériques sont des exceptions autorisées.

```vue annotate="add:2,3,4,5,6"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<div>123</div>
<button :title="$t('close')">{{ $t('x') }}</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47) · [Toutes les règles](all.md)

### `vue/no-boolean-attr-value`

Interdire les valeurs explicites des attributs HTML booléens

[Mauvais](#vue-no-boolean-attr-value-bad) · [Bon](#vue-no-boolean-attr-value-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les attributs booléens disabled et checked contiennent inutilement des valeurs textuelles.

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**Bon**

La présence de chaque attribut booléen exprime le même état actif sans valeur.

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [Toutes les règles](all.md)

### `vue/no-child-content`

Interdire le contenu enfant lors de l’utilisation de v-html ou v-text

[Mauvais](#vue-no-child-content-bad) · [Bon](#vue-no-child-content-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

v-text remplace le contenu du paragraphe ; le texte de repli écrit dans le template ne peut donc pas être conservé avec cette directive.

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**Bon**

Supprimer le texte enfant laisse v-text comme unique source du contenu du paragraphe.

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [Toutes les règles](all.md)

### `vue/no-deprecated-filter`

Interdire la syntaxe obsolète des filtres de Vue 2 utilisant l’opérateur pipe

[Mauvais](#vue-no-deprecated-filter-bad) · [Bon](#vue-no-deprecated-filter-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le pipe utilise la syntaxe supprimée des filtres Vue pour appliquer capitalize.

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

<span id="vue-no-deprecated-filter-good"></span>

**Bon**

L’appel capitalize(message) applique la transformation sous la forme d’une expression ordinaire.

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) · [Toutes les règles](all.md)

### `vue/no-deprecated-functional-template`

Interdire l’attribut `functional` sur le `<template>` d’un SFC

[Mauvais](#vue-no-deprecated-functional-template-bad) · [Bon](#vue-no-deprecated-functional-template-good)

Gravité par défaut: `error`  
Préréglages: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template du SFC possède l’attribut functional supprimé et lit l’ancien contexte props.

```vue annotate="remove:1,2"
<template functional>
<div>{{ props.msg }}</div>
</template>
```

<span id="vue-no-deprecated-functional-template-good"></span>

**Bon**

Le template ordinaire omet functional et lit directement la liaison msg du composant.

```vue annotate="add:1,2"
<template>
<div>{{ msg }}</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) · [Toutes les règles](all.md)

### `vue/no-deprecated-html-element-is`

Interdire l’attribut `is` sur les éléments HTML natifs

[Mauvais](#vue-no-deprecated-html-element-is-bad) · [Bon](#vue-no-deprecated-html-element-is-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un div natif utilise l’ancien attribut is sans préfixe pour demander un composant Vue.

```vue annotate="remove:2"
<template>
<div is="MyComponent" />
</template>
```

<span id="vue-no-deprecated-html-element-is-good"></span>

**Bon**

Un composant dynamique utilise :is ; la syntaxe sur un élément natif utilise explicitement le préfixe vue:.

```vue annotate="add:2,3"
<template>
<component :is="MyComponent" />
<div is="vue:MyComponent" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) · [Toutes les règles](all.md)

### `vue/no-deprecated-inline-template`

Interdire l’attribut obsolète `inline-template`

[Mauvais](#vue-no-deprecated-inline-template-bad) · [Bon](#vue-no-deprecated-inline-template-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Card utilise l’attribut obsolète inline-template pour le contenu qui lui est fourni.

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

<span id="vue-no-deprecated-inline-template-good"></span>

**Bon**

Le même contenu est transmis normalement, sans l’attribut inline-template.

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) · [Toutes les règles](all.md)

### `vue/no-deprecated-router-link-tag-prop`

Interdire la prop `tag` sur &lt;router-link&gt;

[Mauvais](#vue-no-deprecated-router-link-tag-prop-bad) · [Bon](#vue-no-deprecated-router-link-tag-prop-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

RouterLink utilise la prop tag supprimée pour demander un élément button.

```vue annotate="remove:2"
<template>
<router-link to="/home" tag="button">Home</router-link>
</template>
```

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

**Bon**

Le slot fournit navigate à un bouton explicitement écrit dans le template.

```vue annotate="add:2,3,4"
<template>
<router-link to="/home" v-slot="{ navigate }">
<button @click="navigate">Home</button>
</router-link>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [Toutes les règles](all.md)

### `vue/no-deprecated-scope-attribute`

Interdire l’attribut obsolète `scope` sur &lt;template&gt;

[Mauvais](#vue-no-deprecated-scope-attribute-bad) · [Bon](#vue-no-deprecated-scope-attribute-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template du slot déclare props au moyen de l’attribut obsolète scope.

```vue annotate="remove:2"
<template>
<Card><template scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-scope-attribute-good"></span>

**Bon**

La directive du slot par défaut déclare la même liaison props avec la syntaxe actuelle des slots.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) · [Toutes les règles](all.md)

### `vue/no-deprecated-slot-attribute`

Interdire l’attribut obsolète `slot`

[Mauvais](#vue-no-deprecated-slot-attribute-bad) · [Bon](#vue-no-deprecated-slot-attribute-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le slot header est sélectionné au moyen de l’ancien attribut slot.

```vue annotate="remove:3,4"
<template>
<Foo>
<template slot="header"><h1>Title</h1></template>
<div :slot="name">Title</div>
</Foo>
</template>
```

<span id="vue-no-deprecated-slot-attribute-good"></span>

**Bon**

v-slot:header sélectionne explicitement le slot header avec la directive actuelle.

```vue annotate="add:3"
<template>
<Foo>
<template v-slot:header><h1>Title</h1></template>
</Foo>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) · [Toutes les règles](all.md)

### `vue/no-deprecated-slot-scope-attribute`

Interdire l’attribut obsolète `slot-scope`

[Mauvais](#vue-no-deprecated-slot-scope-attribute-bad) · [Bon](#vue-no-deprecated-slot-scope-attribute-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template reçoit les props du slot au moyen de l’attribut obsolète slot-scope.

```vue annotate="remove:2"
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

**Bon**

La directive #default reçoit ces props sans slot-scope.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) · [Toutes les règles](all.md)

### `vue/no-deprecated-v-bind-sync`

Interdire le modificateur obsolète `.sync` sur `v-bind`

[Mauvais](#vue-no-deprecated-v-bind-sync-bad) · [Bon](#vue-no-deprecated-v-bind-sync-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les liaisons utilisent le modificateur .sync supprimé, y compris en combinaison avec .camel.

```vue annotate="remove:2,3,4"
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

<span id="vue-no-deprecated-v-bind-sync-good"></span>

**Bon**

Utilisez une liaison title ordinaire à sens unique, ou v-model:title lorsqu’un canal de mise à jour est nécessaire.

```vue annotate="add:2,3"
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) · [Toutes les règles](all.md)

### `vue/no-deprecated-v-on-native-modifier`

Interdire le modificateur obsolète `.native` sur `v-on`

[Mauvais](#vue-no-deprecated-v-on-native-modifier-bad) · [Bon](#vue-no-deprecated-v-on-native-modifier-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les gestionnaires du composant utilisent le modificateur d’événement .native supprimé.

```vue annotate="remove:2,3,4"
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

**Bon**

Les gestionnaires omettent .native et conservent les autres modificateurs d’événement, tels que .stop.

```vue annotate="add:2,3"
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) · [Toutes les règles](all.md)

### `vue/no-deprecated-v-on-number-modifiers`

Interdire les modificateurs numériques obsolètes `keyCode` sur `v-on`

[Mauvais](#vue-no-deprecated-v-on-number-modifiers-bad) · [Bon](#vue-no-deprecated-v-on-number-modifiers-good)

Gravité par défaut: `error`  
Préréglages: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les gestionnaires de clavier identifient les touches par les codes numériques supprimés 13 et 27.

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

**Bon**

Les gestionnaires utilisent les modificateurs de touches nommés enter et esc.

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [Toutes les règles](all.md)

### `vue/no-dupe-v-else-if`

Interdire les conditions répétées dans les chaînes `v-if` / `v-else-if`

[Mauvais](#vue-no-dupe-v-else-if-bad) · [Bon](#vue-no-dupe-v-else-if-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le else-if répète la condition ready déjà testée par la première branche, ce qui rend cette branche suivante inaccessible.

```vue annotate="remove:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**Bon**

La deuxième branche teste loading, un état distinct qui permet d’atteindre le else-if.

```vue annotate="add:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [Toutes les règles](all.md)

### `vue/no-duplicate-attributes`

Interdire les attributs en double sur un même élément

[Mauvais](#vue-no-duplicate-attributes-bad) · [Bon](#vue-no-duplicate-attributes-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le même bouton déclare class deux fois au lieu de réunir les classes dans une seule valeur.

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**Bon**

Les deux noms de classes figurent dans un unique attribut class.

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [Toutes les règles](all.md)

### `vue/no-empty-component-block`

Interdire les blocs SFC vides

[Mauvais](#vue-no-empty-component-block-bad) · [Bon](#vue-no-empty-component-block-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les blocs template, script et style ne contiennent aucun contenu significatif.

```vue annotate="remove:1,3,5"
<template></template>

<script></script>

<style>
</style>
```

<span id="vue-no-empty-component-block-good"></span>

**Bon**

Chaque bloc conservé contient du balisage, des déclarations de script ou des déclarations de style effectifs.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) · [Toutes les règles](all.md)

### `vue/no-inline-style`

Déconseiller l’utilisation d’attributs de style en ligne

[Mauvais](#vue-no-inline-style-bad) · [Bon](#vue-no-inline-style-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’attribut statique style place la déclaration de couleur dans l’élément.

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**Bon**

Les classes expriment la couleur fixe ; la largeur dépendant de ratio reste une liaison de style dynamique, hors du contrôle des attributs statiques.

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [Toutes les règles](all.md)

### `vue/no-invalid-html-attribute`

Interdire les valeurs statiques invalides des attributs HTML

[Mauvais](#vue-no-invalid-html-attribute-bad) · [Bon](#vue-no-invalid-html-attribute-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le lien utilise stylesheet comme valeur de rel, alors que cette valeur appartient aux éléments link de feuilles de style.

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

<span id="vue-no-invalid-html-attribute-good"></span>

**Bon**

Le lien utilise help, une valeur de rel adaptée à une ressource d’aide liée.

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) · [Toutes les règles](all.md)

### `vue/no-lone-template`

Interdire les éléments `<template>` inutiles

[Mauvais](#vue-no-lone-template-bad) · [Bon](#vue-no-lone-template-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template interne n’a ni directive ni rôle de slot lui donnant une fonction structurelle.

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**Bon**

Supprimer l’enveloppe inutile laisse le paragraphe directement à l’intérieur de div.

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [Toutes les règles](all.md)

### `vue/no-multi-spaces`

Interdire plusieurs espaces consécutifs

[Mauvais](#vue-no-multi-spaces-bad) · [Bon](#vue-no-multi-spaces-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Deux espaces séparent les attributs, ou le nom de l’élément et son premier attribut.

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**Bon**

Un seul espace sépare les mêmes attributs.

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [Toutes les règles](all.md)

### `vue/no-multiple-objects-in-class`

Interdire plusieurs objets littéraux dans une liaison :class sous forme de tableau

[Mauvais](#vue-no-multiple-objects-in-class-bad) · [Bon](#vue-no-multiple-objects-in-class-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un tableau de classes contient deux objets littéraux au premier niveau qui peuvent être fusionnés.

```vue annotate="remove:2,3"
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

<span id="vue-no-multiple-objects-in-class-good"></span>

**Bon**

Un seul objet contient les conditions des classes ; les tableaux comprenant un objet et une chaîne, ou des entrées non littérales, restent autorisés.

```vue annotate="add:2,3,4"
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) · [Toutes les règles](all.md)

### `vue/no-multiple-template-root`

Interdire plusieurs nœuds racines dans un template

[Mauvais](#vue-no-multiple-template-root-bad) · [Bon](#vue-no-multiple-template-root-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Activez cette règle uniquement pour un contrat à racine unique. Vue 3 prend normalement en charge les fragments.

**Configuration (Vite+)**

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

**Mauvais**

La convention à racine unique, activée explicitement, trouve deux paragraphes frères à la racine du template.

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

<span id="vue-no-multiple-template-root-good"></span>

**Bon**

Un élément section enveloppe les paragraphes dans une seule racine ; activez cette convention uniquement lorsqu’un contrat à racine unique est souhaité.

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) · [Toutes les règles](all.md)

### `vue/no-mutating-props`

Interdire la modification des props d’un composant

[Mauvais](#vue-no-mutating-props-bad) · [Bon](#vue-no-mutating-props-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

Incrémenter props.count modifie directement une valeur fournie par le parent.

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**Bon**

Le composant émet update:count avec la nouvelle valeur et laisse au parent la responsabilité de mettre à jour la prop.

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L42) · [Toutes les règles](all.md)

### `vue/no-negated-v-if-condition`

Interdire une condition v-if négative lorsque la chaîne comporte un v-else

[Mauvais](#vue-no-negated-v-if-condition-bad) · [Bon](#vue-no-negated-v-if-condition-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les branches associées v-if et v-else commencent par une condition négative.

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

<span id="vue-no-negated-v-if-condition-good"></span>

**Bon**

Une condition ok positive vient en premier ; lors de l’inversion d’une condition, placez d’abord la branche opposée d’origine. Un v-if négatif isolé et les comparaisons !== restent autorisés.

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) · [Toutes les règles](all.md)

### `vue/no-non-component-keep-alive-child`

Interdire les enveloppes d’éléments ordinaires directement sous `<KeepAlive>`

[Mauvais](#vue-no-non-component-keep-alive-child-bad) · [Bon](#vue-no-non-component-keep-alive-child-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

KeepAlive enveloppe conditionnellement un div natif au lieu de mettre directement UserCard en cache.

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

**Bon**

Le premier exemple fait de UserCard l’enfant conditionnel. L’enveloppe avec v-show illustre une structure hors du contrôle des enfants conditionnels et ne garantit pas la mise en cache de l’enveloppe native.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L14) · [Toutes les règles](all.md)

### `vue/no-preprocessor-lang`

Déconseiller les préprocesseurs CSS au profit du CSS moderne

[Mauvais](#vue-no-preprocessor-lang-bad) · [Bon](#vue-no-preprocessor-lang-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Non implémentée pour le lint des SFC  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Prise en charge actuelle: `no-sfc-finding`

Cette entrée du catalogue n’émet actuellement aucun diagnostic propre à cette règle lors du lint des SFC. La paire Mauvais/Bon décrit la convention souhaitée, et non un diagnostic exécutable. Activer l’identifiant ne fournit pas le contrôle SFC manquant.

**ID configuré (aucun diagnostic SFC actuellement)**

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

**Mauvais**

Le bloc style sélectionne SCSS avec lang. Cela décrit la convention souhaitée sans préprocesseur ; le traitement SFC actuel n’émet pas de diagnostic pour cette règle.

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**Bon**

Les mêmes déclarations CSS omettent le lang du préprocesseur. C’est une correction de la convention, et non une différence de diagnostics Mauvais/Bon exécutable actuellement.

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

Le bon exemple illustre la convention visée ; le traitement actuel des SFC n’émet le diagnostic propre à cette règle pour aucun des deux exemples.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [Toutes les règles](all.md)

### `vue/no-reserved-component-names`

Interdire l’utilisation de noms réservés comme noms de composants

[Mauvais](#vue-no-reserved-component-names-bad) · [Bon](#vue-no-reserved-component-names-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le nom de composant button entre en conflit avec le nom d’un élément HTML natif.

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**Bon**

AppButton est un nom de composant applicatif qui ne réutilise pas le nom natif button.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) · [Toutes les règles](all.md)

### `vue/no-root-v-if`

Interdire v-if sur l’unique élément racine d’un template

[Mauvais](#vue-no-root-v-if-bad) · [Bon](#vue-no-root-v-if-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La racine du composant elle-même apparaît et disparaît sous le contrôle de v-if.

```vue annotate="remove:2"
<template>
<div v-if="show">content</div>
</template>
```

<span id="vue-no-root-v-if-good"></span>

**Bon**

Un div externe stable reste la racine, tandis que le paragraphe imbriqué porte la condition de visibilité.

```vue annotate="add:2,3,4"
<template>
<div>
<p v-if="show">content</p>
</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [Toutes les règles](all.md)

### `vue/no-script-non-standard-lang`

Déconseiller les valeurs lang non standard dans les scripts

[Mauvais](#vue-no-script-non-standard-lang-bad) · [Bon](#vue-no-script-non-standard-lang-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Non implémentée pour le lint des SFC  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Prise en charge actuelle: `no-sfc-finding`

Cette entrée du catalogue n’émet actuellement aucun diagnostic propre à cette règle lors du lint des SFC. La paire Mauvais/Bon décrit la convention souhaitée, et non un diagnostic exécutable. Activer l’identifiant ne fournit pas le contrôle SFC manquant.

**ID configuré (aucun diagnostic SFC actuellement)**

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

**Mauvais**

Le script utilise la syntaxe CoffeeScript avec lang=coffee. Le traitement SFC actuel n’émet pas cette règle du catalogue pour ce langage.

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**Bon**

Le script utilise une déclaration TypeScript ordinaire avec lang=ts, illustrant la convention de langage souhaitée.

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

Le bon exemple illustre la convention visée ; le traitement actuel des SFC n’émet le diagnostic propre à cette règle pour aucun des deux exemples.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [Toutes les règles](all.md)

### `vue/no-src-attribute`

Déconseiller l’attribut src sur les blocs SFC

[Mauvais](#vue-no-src-attribute-bad) · [Bon](#vue-no-src-attribute-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les blocs du SFC délèguent leur contenu template, script et style aux fichiers désignés par src.

```vue annotate="remove:1,2,3"
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**Bon**

Chaque bloc SFC contient son propre contenu sans attribut src externe.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) · [Toutes les règles](all.md)

### `vue/no-static-inline-styles`

Interdire les attributs de style en ligne statiques

[Mauvais](#vue-no-static-inline-styles-bad) · [Bon](#vue-no-static-inline-styles-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le paragraphe porte la déclaration de couleur constante dans son attribut style.

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

<span id="vue-no-static-inline-styles-good"></span>

**Bon**

Une classe notice et une feuille de style scoped définissent la couleur constante en dehors de l’attribut du template.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [Toutes les règles](all.md)

### `vue/no-template-key`

Interdire l’attribut `key` sur `<template>`

[Mauvais](#vue-no-template-key-bad) · [Bon](#vue-no-template-key-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Une enveloppe template sans boucle possède une key, alors qu’elle ne constitue pas la limite d’une itération à clé.

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**Bon**

La clé appartient à une itération template v-for, où elle identifie chaque fragment répété.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [Toutes les règles](all.md)

### `vue/no-template-lang`

Déconseiller l’attribut lang sur le bloc template

[Mauvais](#vue-no-template-lang-bad) · [Bon](#vue-no-template-lang-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Non implémentée pour le lint des SFC  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Prise en charge actuelle: `no-sfc-finding`

Cette entrée du catalogue n’émet actuellement aucun diagnostic propre à cette règle lors du lint des SFC. La paire Mauvais/Bon décrit la convention souhaitée, et non un diagnostic exécutable. Activer l’identifiant ne fournit pas le contrôle SFC manquant.

**ID configuré (aucun diagnostic SFC actuellement)**

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

**Mauvais**

Le template sélectionne Pug avec lang. Il s’agit d’une convention souhaitée limitant les templates au HTML ; le traitement SFC actuel n’émet pas de diagnostic pour cet identifiant du catalogue.

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**Bon**

Un template HTML ordinaire omet lang et utilise directement le paragraphe. Cela illustre la convention sans prétendre qu’un diagnostic SFC est actuellement émis.

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

Le bon exemple illustre la convention visée ; le traitement actuel des SFC n’émet le diagnostic propre à cette règle pour aucun des deux exemples.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [Toutes les règles](all.md)

### `vue/no-template-shadow`

Interdire les noms de variables qui masquent des variables d’une portée externe

[Mauvais](#vue-no-template-shadow-bad) · [Bon](#vue-no-template-shadow-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Le contrôle actuel compare les liaisons de v-for imbriqués. Il ne signale pas une liaison v-for isolée simplement parce qu’elle porte le même nom qu’une liaison du script.

**Configuration (Vite+)**

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

**Mauvais**

Le v-for interne déclare à nouveau item et masque la liaison item externe dans la boucle imbriquée.

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**Bon**

La boucle interne déclare child, laissant item disponible pour la ligne externe et child pour la ligne imbriquée.

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [Toutes les règles](all.md)

### `vue/no-template-target-blank`

Interdire target="_blank" sans rel="noopener noreferrer"

[Mauvais](#vue-no-template-target-blank-bad) · [Bon](#vue-no-template-target-blank-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le lien externe ouvre un nouveau contexte de navigation sans la protection rel attendue.

```vue annotate="remove:2"
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

<span id="vue-no-template-target-blank-good"></span>

**Bon**

Le même lien inclut noopener noreferrer en plus de target=_blank.

```vue annotate="add:2"
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [Toutes les règles](all.md)

### `vue/no-textarea-mustache`

Interdire l’interpolation à doubles accolades dans `<textarea>`

[Mauvais](#vue-no-textarea-mustache-bad) · [Bon](#vue-no-textarea-mustache-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le textarea place message dans une interpolation enfant au lieu de lier sa valeur.

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**Bon**

v-model lie la valeur modifiable du textarea à message.

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [Toutes les règles](all.md)

### `vue/no-undefined-refs`

Interdire les références à des variables non définies dans les templates

[Mauvais](#vue-no-undefined-refs-bad) · [Bon](#vue-no-undefined-refs-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template lit missing, alors que le script ne déclare que message.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

<span id="vue-no-undefined-refs-good"></span>

**Bon**

L’interpolation lit la liaison message existante.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) · [Toutes les règles](all.md)

### `vue/no-unsafe-url`

Signaler les liaisons d’URL potentiellement dangereuses

[Mauvais](#vue-no-unsafe-url-bad) · [Bon](#vue-no-unsafe-url-good)

Gravité par défaut: `warning`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La destination du lien commence par le schéma exécutable javascript:.

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**Bon**

Le lien utilise la destination de navigation locale ordinaire /next.

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [Toutes les règles](all.md)

### `vue/no-unsandboxed-iframe`

Exiger un attribut sandbox sur les éléments iframe

[Mauvais](#vue-no-unsandboxed-iframe-bad) · [Bon](#vue-no-unsandboxed-iframe-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le cadre intégré n’a pas d’attribut sandbox limitant ses capacités.

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

<span id="vue-no-unsandboxed-iframe-good"></span>

**Bon**

sandbox applique des restrictions ; allow-scripts autorise explicitement cette seule capacité lorsque cela est nécessaire.

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) · [Toutes les règles](all.md)

### `vue/no-unused-components`

Interdire l’enregistrement de composants inutilisés dans les templates

[Mauvais](#vue-no-unused-components-bad) · [Bon](#vue-no-unused-components-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

UserAvatar est importé comme composant, mais le template ne l’affiche jamais.

```vue annotate="remove:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

<span id="vue-no-unused-components-good"></span>

**Bon**

Le template affiche le composant UserAvatar importé et lui transmet la liaison user.

```vue annotate="add:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) · [Toutes les règles](all.md)

### `vue/no-unused-properties`

Interdire les propriétés inutilisées définies dans defineProps

[Mauvais](#vue-no-unused-properties-bad) · [Bon](#vue-no-unused-properties-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le composant déclare description comme prop, mais n’affiche que title.

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

<span id="vue-no-unused-properties-good"></span>

**Bon**

Les deux props déclarées sont référencées par le template.

```vue annotate="add:7"
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) · [Toutes les règles](all.md)

### `vue/no-unused-refs`

Signaler les refs de template (ref="x") jamais référencées dans &lt;script&gt;

[Mauvais](#vue-no-unused-refs-bad) · [Bon](#vue-no-unused-refs-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template déclare le nom de ref unused sans liaison de référence correspondante dans le script.

```vue annotate="remove:1,3"
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

<span id="vue-no-unused-refs-good"></span>

**Bon**

La ref de template inputEl possède une liaison ref du même nom dans script setup.

```vue annotate="add:1,3,4"
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) · [Toutes les règles](all.md)

### `vue/no-unused-setup-bindings`

Interdire les liaisons script setup qui ne sont jamais lues

[Mauvais](#vue-no-unused-setup-bindings-bad) · [Bon](#vue-no-unused-setup-bindings-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La liaison message de script setup n’est jamais lue par le template.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

<span id="vue-no-unused-setup-bindings-good"></span>

**Bon**

Le paragraphe interpole message et utilise ainsi la liaison déclarée.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [Toutes les règles](all.md)

### `vue/no-unused-vars`

Interdire les variables inutilisées définies dans les directives v-for et v-slot

[Mauvais](#vue-no-unused-vars-bad) · [Bon](#vue-no-unused-vars-good)

Gravité par défaut: `warning`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La boucle déclare un index inutilisé et le slot déclare foo sans y faire référence.

```vue annotate="remove:2,3,4"
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

<span id="vue-no-unused-vars-good"></span>

**Bon**

Les exemples utilisent index ou indiquent qu’il est volontairement inutilisé en le nommant _index, et le slot affiche data. Les clés fondées sur l’index illustrent uniquement un usage ici, sans recommander cet index pour une identité stable des éléments.

```vue annotate="add:2,3,4,5"
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) · [Toutes les règles](all.md)

### `vue/no-use-v-else-with-v-for`

Interdire `v-else-if` ou `v-else` sur le même élément que `v-for`

[Mauvais](#vue-no-use-v-else-with-v-for-bad) · [Bon](#vue-no-use-v-else-with-v-for-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La branche else et l’itération v-for sont attachées au même paragraphe.

```vue annotate="remove:3"
<template>
<p v-if="ready">Ready</p>
<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

<span id="vue-no-use-v-else-with-v-for-good"></span>

**Bon**

Un template distinct porte v-else, et son paragraphe enfant porte v-for.

```vue annotate="add:3"
<template>
<p v-if="ready">Ready</p>
<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) · [Toutes les règles](all.md)

### `vue/no-use-v-if-with-v-for`

Interdire `v-if` sur le même élément que `v-for`

[Mauvais](#vue-no-use-v-if-with-v-for-bad) · [Bon](#vue-no-use-v-if-with-v-for-good)

Gravité par défaut: `warning`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le même élément de liste combine v-if et v-for et teste la visibilité au moyen de la liaison de la boucle.

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**Bon**

Une collection calculée filtre les éléments visibles avant que le template n’itère dessus.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) · [Toutes les règles](all.md)

### `vue/no-useless-mustaches`

Interdire une interpolation à doubles accolades dont l’expression est une chaîne littérale constante

[Mauvais](#vue-no-useless-mustaches-bad) · [Bon](#vue-no-useless-mustaches-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’interpolation ne contient qu’une chaîne constante et ne nécessite pas d’évaluation d’expression.

```vue annotate="remove:2,3,4"
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

<span id="vue-no-useless-mustaches-good"></span>

**Bon**

Le texte littéral est écrit directement ; les expressions de variables, les chaînes de template avec interpolation et les espaces de séparation intentionnels restent des cas d’interpolation.

```vue annotate="add:2,3,4,5"
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) · [Toutes les règles](all.md)

### `vue/no-useless-template-attributes`

Interdire les attributs inutiles sur les éléments `<template>`

[Mauvais](#vue-no-useless-template-attributes-bad) · [Bon](#vue-no-useless-template-attributes-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template conditionnel possède une classe, mais cette enveloppe structurelle ne produit pas d’élément DOM qui puisse la recevoir.

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**Bon**

La classe est déplacée vers le paragraphe qui est effectivement affiché, tandis que v-if reste sur le template structurel.

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [Toutes les règles](all.md)

### `vue/no-useless-v-bind`

Interdire un v-bind dont la valeur est une simple chaîne littérale

[Mauvais](#vue-no-useless-v-bind-bad) · [Bon](#vue-no-useless-v-bind-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La liaison foo évalue une chaîne constante entre guillemets ou une chaîne de template sans interpolation.

```vue annotate="remove:2,3"
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

<span id="vue-no-useless-v-bind-good"></span>

**Bon**

La valeur constante devient un attribut statique ; les valeurs variables et interpolées conservent leur liaison.

```vue annotate="add:2,3,4"
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [Toutes les règles](all.md)

### `vue/no-v-for-template-key-on-child`

Interdire `key` sur l’enfant d’un `<template v-for>`

[Mauvais](#vue-no-v-for-template-key-on-child-bad) · [Bon](#vue-no-v-for-template-key-on-child-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le paragraphe enfant porte la clé, tandis que l’itération du template elle-même n’en possède pas.

```vue annotate="remove:2"
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

<span id="vue-no-v-for-template-key-on-child-good"></span>

**Bon**

La clé est déplacée vers template v-for et identifie le fragment répété dans son ensemble.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) · [Toutes les règles](all.md)

### `vue/no-v-html`

Déconseiller v-html pour prévenir les vulnérabilités XSS

[Mauvais](#vue-no-v-html-bad) · [Bon](#vue-no-v-html-good)

Gravité par défaut: `warning`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

v-html interprète le contenu comme du HTML plutôt que comme du texte ordinaire.

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**Bon**

L’interpolation à doubles accolades affiche le contenu comme du texte échappé au lieu d’injecter du HTML.

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [Toutes les règles](all.md)

### `vue/no-v-text`

Interdire la directive v-text ; préférer l’interpolation à doubles accolades

[Mauvais](#vue-no-v-text-bad) · [Bon](#vue-no-v-text-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le contenu du div est fourni par la directive v-text.

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

<span id="vue-no-v-text-good"></span>

**Bon**

L’interpolation à doubles accolades exprime la même liaison de texte directement dans le contenu de l’élément.

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) · [Toutes les règles](all.md)

### `vue/no-v-text-v-html-on-component`

Interdire v-text / v-html sur les éléments de composants

[Mauvais](#vue-no-v-text-v-html-on-component-bad) · [Bon](#vue-no-v-text-v-html-on-component-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La balise du composant reçoit v-html ou v-text, qui remplace le contenu d’un élément au lieu de fournir les slots du composant.

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**Bon**

Les cibles HTML natives peuvent recevoir les directives ; MyComponent reçoit son contenu via le slot par défaut.

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [Toutes les règles](all.md)

### `vue/permitted-contents`

Faire respecter les règles du modèle de contenu HTML

[Mauvais](#vue-permitted-contents-bad) · [Bon](#vue-permitted-contents-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les exemples placent du contenu de bloc dans p, omettent le corps du tableau, imbriquent des contrôles interactifs ou placent un div directement dans ul.

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

<span id="vue-permitted-contents-good"></span>

**Bon**

Les exemples utilisent du contenu en ligne dans le paragraphe, un tbody explicite et des enfants li. Le composant personnalisé MyItem n’est pas considéré comme un enfant natif connu de ul.

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [Toutes les règles](all.md)

### `vue/prefer-props-shorthand`

Recommander la syntaxe abrégée des props (Vue 3.4+)

[Mauvais](#vue-prefer-props-shorthand-bad) · [Bon](#vue-prefer-props-shorthand-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Chaque liaison répète le nom de la variable correspondante, y compris l’équivalent camelCase d’un argument séparé par des tirets.

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

<span id="vue-prefer-props-shorthand-good"></span>

**Bon**

La syntaxe abrégée des liaisons de même nom de Vue 3.4+ supprime les expressions répétées ; une variable source différente, telle que bar, reste explicite.

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) · [Toutes les règles](all.md)

### `vue/prefer-true-attribute-shorthand`

Préférer la syntaxe abrégée pour un attribut booléen lié à `true`

[Mauvais](#vue-prefer-true-attribute-shorthand-bad) · [Bon](#vue-prefer-true-attribute-shorthand-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un attribut booléen natif disabled est lié à la valeur constante true.

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

<span id="vue-prefer-true-attribute-shorthand-good"></span>

**Bon**

L’attribut natif utilise sa syntaxe booléenne abrégée. Les liaisons à false et les props de composants conservent leurs valeurs explicites.

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) · [Toutes les règles](all.md)

### `vue/prop-name-casing`

Imposer une casse aux noms des props déclarées

[Mauvais](#vue-prop-name-casing-bad) · [Bon](#vue-prop-name-casing-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Contrôle les noms des props déclarées, et non la casse des attributs transmis à un enfant.

**Configuration (Vite+)**

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

**Mauvais**

Le nom de prop déclaré user_name utilise des mots séparés par des traits de soulignement.

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**Bon**

La déclaration et sa référence dans le template utilisent le nom camelCase userName.

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [Toutes les règles](all.md)

### `vue/require-component-is`

Exiger `v-bind:is` sur les éléments `<component>`

[Mauvais](#vue-require-component-is-bad) · [Bon](#vue-require-component-is-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le `<component>` dynamique n’a pas de cible `is` ; Vue ne peut donc pas choisir le composant à afficher.

```vue annotate="remove:2"
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**Bon**

`:is="currentComponent"` fournit la sélection du composant ; la liaison peut changer à l’exécution.

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [Toutes les règles](all.md)

### `vue/require-component-registration`

Exiger un import ou un enregistrement explicite des composants

[Mauvais](#vue-require-component-registration-bad) · [Bon](#vue-require-component-registration-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

Listez les noms explicites des composants fournis par les plugins de l’application ou par previewSetup de Musea. Les graphies PascalCase et kebab-case sont acceptées ; les expressions régulières ne sont pas interprétées. Les options n’activent pas la règle. Les couches suivantes remplacent la liste ; une liste vide efface les noms hérités.

**Configuration (Vite+)**

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

**Mauvais**

`MissingWidget` n’est ni enregistré ni inclus dans la liste configurée des composants globaux autorisés.

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**Bon**

`MyButton` figure dans l’option `globals` de l’exemple. Cette option exempte un composant global connu ; elle ne l’enregistre ni ne l’importe.

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [Toutes les règles](all.md)

### `vue/require-scoped-style`

Exiger l’attribut scoped sur les balises style

[Mauvais](#vue-require-scoped-style-bad) · [Bon](#vue-require-scoped-style-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le style `.button` n’est pas scoped et peut affecter les éléments correspondants en dehors de ce composant.

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**Bon**

Ajouter `scoped` applique la portée du composant Vue aux mêmes sélecteur et déclarations.

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [Toutes les règles](all.md)

### `vue/require-toggle-inside-transition`

Exiger un mécanisme de bascule sur l’élément enveloppé par `<transition>`

[Mauvais](#vue-require-toggle-inside-transition-bad) · [Bon](#vue-require-toggle-inside-transition-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’enfant statique à l’intérieur de `<Transition>` n’a ni visibilité conditionnelle ni sélection dynamique pouvant déclencher une entrée ou une sortie.

```vue annotate="remove:3"
<template>
<transition>
<div>content</div>
</transition>
</template>
```

<span id="vue-require-toggle-inside-transition-good"></span>

**Bon**

`v-if="show"` modifie la présence de l’enfant et fournit à la transition une limite d’entrée et de sortie.

```vue annotate="add:3"
<template>
<transition>
<div v-if="show">content</div>
</transition>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) · [Toutes les règles](all.md)

### `vue/require-v-for-key`

Exiger `v-bind:key` avec les directives `v-for`

[Mauvais](#vue-require-v-for-key-bad) · [Bon](#vue-require-v-for-key-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Chaque `<li>` répété ne possède pas de clé identifiant l’élément correspondant lors des mises à jour de la liste.

```vue annotate="remove:2"
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**Bon**

`:key="item.id"` donne à chaque nœud répété l’identité de son élément plutôt que sa position actuelle.

```vue annotate="add:2"
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) · [Toutes les règles](all.md)

### `vue/scoped-event-names`

Recommander des noms d’événements préfixés par leur contexte au format context:event

[Mauvais](#vue-scoped-event-names-bad) · [Bon](#vue-scoped-event-names-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`playAudio`, `pauseAudio` et `reloadAudio` encodent leur contexte sous forme de suffixes camelCase au lieu de la convention d’événements séparés par deux-points définie par la règle.

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

**Bon**

`audio:play`, `audio:pause` et `audio:reload` partagent un contexte explicite `audio:`. Le composant émetteur doit utiliser les mêmes noms.

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) · [Toutes les règles](all.md)

### `vue/sfc-element-order`

Imposer un ordre cohérent des éléments de premier niveau d’un SFC

[Mauvais](#vue-sfc-element-order-bad) · [Bon](#vue-sfc-element-order-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

Le bloc style précède le bloc script, contrairement à l’ordre des blocs SFC configuré.

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

**Bon**

Les blocs suivent l’ordre script → template → style. Les projets peuvent choisir un autre ordre avec l’option typée de cette règle.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) · [Toutes les règles](all.md)

### `vue/single-style-block`

Recommander un seul bloc style

[Mauvais](#vue-single-style-block-bad) · [Bon](#vue-single-style-block-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le composant répartit ses styles scoped de panneau et de titre dans deux blocs style.

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

**Bon**

Les deux sélecteurs restent scoped dans un seul bloc style, respectant la convention du bloc unique sans supprimer aucun style.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) · [Toutes les règles](all.md)

### `vue/slot-name-casing`

Imposer kebab-case aux slots nommés utilisés avec v-slot

[Mauvais](#vue-slot-name-casing-bad) · [Bon](#vue-slot-name-casing-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le slot nommé `mySlot` utilise camelCase là où la règle exige un nom séparé par des tirets.

```vue annotate="remove:2"
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

<span id="vue-slot-name-casing-good"></span>

**Bon**

`#my-slot` utilise kebab-case. Renommez le point d’insertion du slot correspondant avec le même nom.

```vue annotate="add:2"
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [Toutes les règles](all.md)

### `vue/this-in-template`

Interdire `this.` dans les expressions des templates

[Mauvais](#vue-this-in-template-bad) · [Bon](#vue-this-in-template-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les expressions du template accèdent explicitement à `this.message`, `this.className` et `this.handleClick`, alors que Vue expose directement ces liaisons.

```vue annotate="remove:2,3,4"
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

<span id="vue-this-in-template-good"></span>

**Bon**

Utilisez directement `message`, `className` et `handleClick`. La chaîne littérale `'this.is.a.string'` reste inchangée, car elle ne constitue pas un accès à un membre.

```vue annotate="add:2,3,4,5"
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) · [Toutes les règles](all.md)

### `vue/use-unique-element-ids`

Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques

[Mauvais](#vue-use-unique-element-ids-bad) · [Bon](#vue-use-unique-element-ids-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’identifiant littéral `email` est réutilisé par chaque instance de ce composant, ce qui peut faire pointer son label vers la mauvaise instance lorsque plusieurs sont affichées.

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**Bon**

`useId()` produit l’`emailId` de l’instance ; liez la même valeur au `for` du label et à l’`id` de l’input.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [Toutes les règles](all.md)

### `vue/use-v-on-exact`

Imposer le modificateur `.exact` sur `v-on` en présence de gestionnaires utilisant des modificateurs

[Mauvais](#vue-use-v-on-exact-bad) · [Bon](#vue-use-v-on-exact-good)

Gravité par défaut: `warning`  
Préréglages: `essential`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le gestionnaire de clic ordinaire peut également s’exécuter lors d’un Ctrl-clic et chevaucher le gestionnaire `.ctrl` distinct.

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**Bon**

`.exact` limite le gestionnaire de clic ordinaire aux clics sans touche modificatrice ; le gestionnaire propre à Ctrl reste distinct.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) · [Toutes les règles](all.md)

### `vue/v-bind-style`

Imposer un style de directive `v-bind`

[Mauvais](#vue-v-bind-style-bad) · [Bon](#vue-v-bind-style-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-bind:class` utilise la forme longue alors que le style de liaison configuré exige la syntaxe abrégée avec deux-points.

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**Bon**

`:class` conserve la même expression avec la syntaxe abrégée exigée ; cette règle concerne la graphie, pas le type de la valeur.

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [Toutes les règles](all.md)

### `vue/v-on-event-hyphenation`

Imposer des tirets dans les noms d’événements personnalisés de v-on sur les composants

[Mauvais](#vue-v-on-event-hyphenation-bad) · [Bon](#vue-v-on-event-hyphenation-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

L’écouteur du composant personnalisé utilise `@myEvent` au lieu d’un nom d’événement séparé par des tirets.

```vue annotate="remove:2,3"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
</template>
```

<span id="vue-v-on-event-hyphenation-good"></span>

**Bon**

`@my-event` utilise la graphie exigée pour les événements personnalisés. Les écouteurs d’éléments natifs et les arguments d’événements dynamiques montrés ci-dessous sont hors du champ de ce contrôle.

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) · [Toutes les règles](all.md)

### `vue/v-on-handler-style`

Imposer une référence de méthode ou une fonction en ligne pour les gestionnaires v-on

[Mauvais](#vue-v-on-handler-style-bad) · [Bon](#vue-v-on-handler-style-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les gestionnaires placent des modifications et plusieurs instructions directement dans l’attribut d’événement.

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

<span id="vue-v-on-handler-style-good"></span>

**Bon**

Utilisez une référence de gestionnaire, ou une expression de fonction fléchée ou classique lorsqu’une logique en ligne est nécessaire. La limite de la fonction rend la forme du gestionnaire explicite.

```vue annotate="add:2,3,4,5"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) · [Toutes les règles](all.md)

### `vue/v-on-style`

Imposer un style de directive `v-on`

[Mauvais](#vue-v-on-style-bad) · [Bon](#vue-v-on-style-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-on:click` utilise la forme longue de l’écouteur d’événement alors que la règle exige la syntaxe abrégée.

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**Bon**

`@click` conserve le même gestionnaire tout en utilisant la syntaxe abrégée configurée.

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [Toutes les règles](all.md)

### `vue/v-slot-style`

Imposer un style de directive `v-slot`

[Mauvais](#vue-v-slot-style-bad) · [Bon](#vue-v-slot-style-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le composant utilise `#default` et le template utilise `v-slot:header`, contrairement aux styles définis par la règle pour chaque contexte.

```vue annotate="remove:2,4"
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

<span id="vue-v-slot-style-good"></span>

**Bon**

Utilisez `v-slot` pour le slot par défaut du composant et `#header` pour le slot nommé du template.

```vue annotate="add:2,4"
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [Toutes les règles](all.md)

### `vue/valid-attribute-name`

Exiger des noms d’attributs valides

[Mauvais](#vue-valid-attribute-name-bad) · [Bon](#vue-valid-attribute-name-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Diagnostic du mauvais exemple: `parser/template`

Un nom d’attribut mal formé est diagnostiqué par parser/template avant que cette règle défensive ne voie un attribut. L’exemple Mauvais signale donc parser/template et ne garantit pas un diagnostic distinct de vue/valid-attribute-name.

**Configuration (Vite+)**

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

**Mauvais**

Le guillemet dans `my"attr` rend le nom d’attribut mal formé. Cet exemple produit le diagnostic `parser/template` de l’analyseur, sans garantir un diagnostic distinct de cette règle.

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**Bon**

`my-attr` est un nom d’attribut bien formé ; l’analyseur du template peut donc lire l’attribut et sa valeur.

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [Toutes les règles](all.md)

### `vue/valid-template-root`

Exiger une racine `<template>` valide selon la sémantique des fragments de Vue 3

[Mauvais](#vue-valid-template-root-bad) · [Bon](#vue-valid-template-root-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un `<template>` imbriqué ordinaire occupe la racine du template sans directive lui donnant un rôle de rendu.

```vue annotate="remove:2"
<template>
<template>content</template>
</template>
```

<span id="vue-valid-template-root-good"></span>

**Bon**

Le `<div>` est un élément racine qui peut être affiché. Cet exemple n’impose pas de restriction universelle à une seule racine pour les fragments de Vue 3.

```vue annotate="add:2"
<template>
<div>content</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [Toutes les règles](all.md)

### `vue/valid-v-bind`

Exiger des directives `v-bind` valides

[Mauvais](#vue-valid-v-bind-bad) · [Bon](#vue-valid-v-bind-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le `v-bind` sans argument n’a pas d’expression objet, et la forme à argument vide n’a pas de nom d’attribut.

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

<span id="vue-valid-v-bind-good"></span>

**Bon**

Fournissez un attribut et une expression, liez un objet ou utilisez la syntaxe abrégée de même nom de Vue 3.4+, telle que `:loading`.

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) · [Toutes les règles](all.md)

### `vue/valid-v-cloak`

Exiger des directives `v-cloak` valides

[Mauvais](#vue-valid-v-cloak-bad) · [Bon](#vue-valid-v-cloak-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-cloak` reçoit une valeur, un argument ou un modificateur alors qu’il n’accepte aucun de ces éléments.

```vue annotate="remove:2,3,4"
<template>
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

<span id="vue-valid-v-cloak-good"></span>

**Bon**

Utilisez `v-cloak` seul ; le CSS peut masquer l’élément jusqu’à ce que Vue retire cet attribut après le montage.

```vue annotate="add:2"
<template>
<div v-cloak></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) · [Toutes les règles](all.md)

### `vue/valid-v-else`

Exiger des directives `v-else` valides

[Mauvais](#vue-valid-v-else-bad) · [Bon](#vue-valid-v-else-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les exemples donnent une expression à `v-else`, le combinent avec `v-if` ou omettent la branche conditionnelle qui doit le précéder immédiatement.

```vue annotate="remove:2,3"
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

<span id="vue-valid-v-else-good"></span>

**Bon**

Placez `v-else` seul immédiatement après la branche `v-if` correspondante.

```vue annotate="add:2"
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) · [Toutes les règles](all.md)

### `vue/valid-v-for`

Exiger des directives `v-for` valides

[Mauvais](#vue-valid-v-for-bad) · [Bon](#vue-valid-v-for-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les boucles omettent leur expression d’itération ou ajoutent un modificateur `.stop` non pris en charge.

```vue annotate="remove:2,3,4"
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

<span id="vue-valid-v-for-good"></span>

**Bon**

Utilisez `item in items` ou `(item, index) of items` avec une expression d’itération complète et les clés présentées.

```vue annotate="add:2,3"
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) · [Toutes les règles](all.md)

### `vue/valid-v-html`

Exiger des directives `v-html` valides

[Mauvais](#vue-valid-v-html-bad) · [Bon](#vue-valid-v-html-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-html` n’a pas d’expression ou utilise un argument ou modificateur que cette directive ne prend pas en charge.

```vue annotate="remove:2,3,4"
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

<span id="vue-valid-v-html-good"></span>

**Bon**

`v-html="html"` fournit une expression valide. La validité de la syntaxe n’assainit pas le HTML et ne rend pas sûr un contenu non fiable.

```vue annotate="add:2"
<template>
<div v-html="html"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) · [Toutes les règles](all.md)

### `vue/valid-v-if`

Exiger des directives `v-if` valides

[Mauvais](#vue-valid-v-if-bad) · [Bon](#vue-valid-v-if-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les conditions omettent une expression ou combinent `v-if` avec une directive else sur le même nœud.

```vue annotate="remove:2,3,4"
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

<span id="vue-valid-v-if-good"></span>

**Bon**

Chaque `v-if` possède une condition non vide, telle que `ready` ou `count > 0`, sans directive else incompatible.

```vue annotate="add:2,3"
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) · [Toutes les règles](all.md)

### `vue/valid-v-memo`

Exiger des directives `v-memo` valides

[Mauvais](#vue-valid-v-memo-bad) · [Bon](#vue-valid-v-memo-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un `v-memo` seul ne donne à Vue aucune expression de dépendances pour décider quand réutiliser le sous-arbre.

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

<span id="vue-valid-v-memo-good"></span>

**Bon**

`v-memo="[valueA, valueB]"` fournit le tableau de dépendances utilisé pour la mémoïsation.

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [Toutes les règles](all.md)

### `vue/valid-v-model`

Exiger des directives `v-model` valides

[Mauvais](#vue-valid-v-model-bad) · [Bon](#vue-valid-v-model-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un `<div>` natif ne peut pas utiliser `v-model` comme contrôle de formulaire, et une directive sans valeur sur input n’a pas d’expression cible modifiable.

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

<span id="vue-valid-v-model-good"></span>

**Bon**

Liez l’input, le select, le textarea ou le composant personnalisé aux variables modifiables présentées.

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) · [Toutes les règles](all.md)

### `vue/valid-v-on`

Exiger des directives `v-on` valides

[Mauvais](#vue-valid-v-on-bad) · [Bon](#vue-valid-v-on-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les formes d’écouteurs omettent un argument d’événement ou l’expression de gestionnaire ou d’objet nécessaire.

```vue annotate="remove:2,3,4"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

<span id="vue-valid-v-on-good"></span>

**Bon**

Utilisez un événement avec son gestionnaire, ou transmettez un objet d’écouteurs à `v-on` sans argument.

```vue annotate="add:2,3"
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [Toutes les règles](all.md)

### `vue/valid-v-once`

Exiger des directives `v-once` valides

[Mauvais](#vue-valid-v-once-bad) · [Bon](#vue-valid-v-once-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-once` possède une valeur, un argument ou un modificateur, alors que cette directive marque un rendu unique sans recevoir de valeur.

```vue annotate="remove:2,3,4"
<template>
<div v-once="foo"></div>
<div v-once:arg></div>
<div v-once.mod></div>
</template>
```

<span id="vue-valid-v-once-good"></span>

**Bon**

`v-once` seul marque le sous-arbre pour un rendu unique sans syntaxe non prise en charge.

```vue annotate="add:2"
<template>
<div v-once></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) · [Toutes les règles](all.md)

### `vue/valid-v-show`

Exiger des directives `v-show` valides

[Mauvais](#vue-valid-v-show-bad) · [Bon](#vue-valid-v-show-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-show` n’a pas d’expression de visibilité ou est placé sur un `<template>` qui ne produit aucun élément DOM dont l’affichage puisse être modifié.

```vue annotate="remove:2,3"
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

<span id="vue-valid-v-show-good"></span>

**Bon**

Appliquez l’expression de visibilité à un élément affiché, tel que `<div>`.

```vue annotate="add:2,3"
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [Toutes les règles](all.md)

### `vue/valid-v-slot`

Exiger des directives `v-slot` valides

[Mauvais](#vue-valid-v-slot-bad) · [Bon](#vue-valid-v-slot-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

La directive de slot est placée sur un `<div>` natif ou entre en conflit avec d’autres déclarations de slots par défaut ou nommés.

```vue annotate="remove:2,3,4"
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

<span id="vue-valid-v-slot-good"></span>

**Bon**

Déclarez le slot par défaut d’un composant sur ce composant, ou son slot nommé sur un enfant `<template #header>`.

```vue annotate="add:2,3,4,5"
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) · [Toutes les règles](all.md)

### `vue/valid-v-text`

Exiger des directives `v-text` valides

[Mauvais](#vue-valid-v-text-bad) · [Bon](#vue-valid-v-text-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-text` n’a pas d’expression de texte ou utilise un argument ou modificateur non pris en charge.

```vue annotate="remove:2,3,4"
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

<span id="vue-valid-v-text-good"></span>

**Bon**

`v-text="msg"` est syntaxiquement valide. La règle de style distincte `vue/no-v-text` peut toujours préférer l’interpolation.

```vue annotate="add:2"
<template>
<div v-text="msg"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [Toutes les règles](all.md)

### `vue/warn-custom-block`

Signaler les blocs personnalisés dans les fichiers SFC

[Mauvais](#vue-warn-custom-block-bad) · [Bon](#vue-warn-custom-block-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le SFC contient un bloc personnalisé `<i18n>`, qui nécessite une intégration externe au-delà du traitement ordinaire de template, script et style.

```vue annotate="remove:1,2,3,4"
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

<span id="vue-warn-custom-block-good"></span>

**Bon**

L’exemple utilise les blocs standard template et script-setup. Cet avertissement facultatif de portabilité ne signifie pas que tous les blocs personnalisés sont invalides en Vue.

```vue annotate="add:4,5,6,7"
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [Toutes les règles](all.md)

### `vue/warn-custom-directive`

Signaler les directives personnalisées nécessitant un enregistrement

[Mauvais](#vue-warn-custom-directive-bad) · [Bon](#vue-warn-custom-directive-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-focus`, `v-mask` et `v-click-outside` nécessitent des implémentations de directives propres au projet, signalées par cette convention facultative.

```vue annotate="remove:2,3,4"
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**Bon**

L’exemple utilise les directives intégrées `v-if`, `v-model` et `v-on`. Une directive personnalisée correctement enregistrée peut rester valide en Vue lorsque cette politique est désactivée.

```vue annotate="add:2,3,4"
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [Toutes les règles](all.md)
