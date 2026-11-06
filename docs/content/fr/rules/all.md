---
title: "Toutes les règles de lint"
description: "Toutes les règles du linter de Vize, regroupées par catégorie. Ouvrez une règle pour voir ce qu’elle signale, un mauvais et un bon exemple, et comment l’activer."
---

# Toutes les règles de lint

Toutes les règles du linter de Vize, regroupées par catégorie. Ouvrez une règle pour voir ce qu’elle signale, un mauvais et un bon exemple, et comment l’activer.

La colonne Préréglage indique le plus petit préréglage qui active la règle ; la page de chaque règle les liste tous. Voir [préréglages](../rules/index.md#préréglages) pour le contenu de chacun.

## Règles Vue (120)

| Règle | Ce qu’elle vérifie | Préréglage | Correction automatique |
| --- | --- | --- | --- |
| <span id="vue-a11y-img-alt"></span>[`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | Exiger un attribut alt sur les images pour les rendre accessibles. | — |  |
| <span id="vue-attribute-hyphenation"></span>[`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | Imposer un style de nommage des attributs sur les composants personnalisés. | `happy-path` | ✓ |
| <span id="vue-attribute-order"></span>[`vue/attribute-order`](./reference/vue-attribute-order.md) | Imposer un ordre cohérent des attributs. | `happy-path` |  |
| <span id="vue-comment-directive"></span>[`vue/comment-directive`](./reference/vue-comment-directive.md) | Prend en charge les commentaires de contrôle des diagnostics dans `<template>`. | `essential` |  |
| <span id="vue-component-definition-name-casing"></span>[`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | Imposer PascalCase ou kebab-case aux noms de définition des composants. | `happy-path` |  |
| <span id="vue-component-name-in-template-casing"></span>[`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | Imposer une casse précise aux noms de composants dans les templates. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-html-button-has-type"></span>[`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | Exiger un type explicite et valide sur les éléments button. | `nuxt`, `opinionated` |  |
| <span id="vue-html-end-tags"></span>[`vue/html-end-tags`](./reference/vue-html-end-tags.md) | Exige une balise de fermeture explicite pour les éléments HTML qui permettent de l'omettre. | `happy-path` | ✓ |
| <span id="vue-html-quotes"></span>[`vue/html-quotes`](./reference/vue-html-quotes.md) | Imposer un style de guillemets pour les attributs HTML. | `happy-path` | ✓ |
| <span id="vue-html-self-closing"></span>[`vue/html-self-closing`](./reference/vue-html-self-closing.md) | Imposer un style de fermeture automatique des balises. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-jsx-uses-vars"></span>[`vue/jsx-uses-vars`](./reference/vue-jsx-uses-vars.md) | Considère comme utilisées les variables référencées dans les noms d'éléments JSX. | `essential` |  |
| <span id="vue-match-component-file-name"></span>[`vue/match-component-file-name`](./reference/vue-match-component-file-name.md) | Exige que le nom déclaré du composant corresponde au nom du fichier. | — |  |
| <span id="vue-max-template-complexity"></span>[`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | Limiter la complexité propre du template d’un composant, cyclomatique et cognitive. | — |  |
| <span id="vue-multi-word-component-names"></span>[`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | Exiger des noms de composants composés de plusieurs mots. | `essential` |  |
| <span id="vue-mustache-interpolation-spacing"></span>[`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | Imposer un espacement cohérent dans les interpolations à doubles accolades. | `happy-path` | ✓ |
| <span id="vue-no-array-index-key"></span>[`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | Interdire l’utilisation directe de la variable d’index de v-for comme :key. | `nuxt`, `opinionated` |  |
| <span id="vue-no-bare-strings-in-template"></span>[`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé. | — |  |
| <span id="vue-no-boolean-attr-value"></span>[`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | Interdire les valeurs explicites des attributs HTML booléens. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-child-content"></span>[`vue/no-child-content`](./reference/vue-no-child-content.md) | Interdire le contenu enfant lors de l’utilisation de v-html ou v-text. | `essential` |  |
| <span id="vue-no-custom-modifiers-on-v-model"></span>[`vue/no-custom-modifiers-on-v-model`](./reference/vue-no-custom-modifiers-on-v-model.md) | Interdit les modificateurs personnalisés sur le `v-model` d'un composant pour assurer la compatibilité avec Vue 2. | — |  |
| <span id="vue-no-deprecated-filter"></span>[`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | Interdire la syntaxe obsolète des filtres de Vue 2 utilisant l’opérateur pipe. | `essential` |  |
| <span id="vue-no-deprecated-functional-template"></span>[`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | Interdire l’attribut `functional` sur le `<template>` d’un SFC. | `essential` |  |
| <span id="vue-no-deprecated-html-element-is"></span>[`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | Interdire l’attribut `is` sur les éléments HTML natifs. | `essential` |  |
| <span id="vue-no-deprecated-inline-template"></span>[`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | Interdire l’attribut obsolète `inline-template`. | `essential` |  |
| <span id="vue-no-deprecated-router-link-tag-prop"></span>[`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | Interdire la prop `tag` sur &lt;router-link&gt;. | `essential` |  |
| <span id="vue-no-deprecated-scope-attribute"></span>[`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | Interdire l’attribut obsolète `scope` sur &lt;template&gt;. | `essential` |  |
| <span id="vue-no-deprecated-slot-attribute"></span>[`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | Interdire l’attribut obsolète `slot`. | `essential` |  |
| <span id="vue-no-deprecated-slot-scope-attribute"></span>[`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | Interdire l’attribut obsolète `slot-scope`. | `essential` |  |
| <span id="vue-no-deprecated-v-bind-sync"></span>[`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | Interdire le modificateur obsolète `.sync` sur `v-bind`. | `essential` |  |
| <span id="vue-no-deprecated-v-is"></span>[`vue/no-deprecated-v-is`](./reference/vue-no-deprecated-v-is.md) | Interdire la directive obsolète `v-is`. | `essential` |  |
| <span id="vue-no-deprecated-v-on-native-modifier"></span>[`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | Interdire le modificateur obsolète `.native` sur `v-on`. | `essential` |  |
| <span id="vue-no-deprecated-v-on-number-modifiers"></span>[`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | Interdire les modificateurs numériques obsolètes `keyCode` sur `v-on`. | `essential` | ✓ |
| <span id="vue-no-dupe-v-else-if"></span>[`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | Interdire les conditions répétées dans les chaînes `v-if` / `v-else-if`. | `essential` |  |
| <span id="vue-no-duplicate-attributes"></span>[`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | Interdire les attributs en double sur un même élément. | `essential` |  |
| <span id="vue-no-empty-component-block"></span>[`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | Interdire les blocs SFC vides. | `nuxt`, `opinionated` |  |
| <span id="vue-no-inline-style"></span>[`vue/no-inline-style`](./reference/vue-no-inline-style.md) | Déconseiller l’utilisation d’attributs de style en ligne. | `nuxt`, `opinionated` |  |
| <span id="vue-no-invalid-html-attribute"></span>[`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | Interdire les valeurs statiques invalides des attributs HTML. | `happy-path` |  |
| <span id="vue-no-lone-template"></span>[`vue/no-lone-template`](./reference/vue-no-lone-template.md) | Interdire les éléments `<template>` inutiles. | `happy-path` |  |
| <span id="vue-no-multi-spaces"></span>[`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | Interdire plusieurs espaces consécutifs. | `happy-path` | ✓ |
| <span id="vue-no-multiple-objects-in-class"></span>[`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | Interdire plusieurs objets littéraux dans une liaison :class sous forme de tableau. | `nuxt`, `opinionated` |  |
| <span id="vue-no-multiple-template-root"></span>[`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | Interdire plusieurs nœuds racines dans un template. | — |  |
| <span id="vue-no-mutating-props"></span>[`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | Interdire la modification des props d’un composant. | `essential` |  |
| <span id="vue-no-negated-v-if-condition"></span>[`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | Interdire une condition v-if négative lorsque la chaîne comporte un v-else. | `nuxt`, `opinionated` |  |
| <span id="vue-no-non-component-keep-alive-child"></span>[`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | Interdire les enveloppes d’éléments ordinaires directement sous `<KeepAlive>`. | `essential` |  |
| <span id="vue-no-preprocessor-lang"></span>[`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | Déconseiller les préprocesseurs CSS au profit du CSS moderne. | `nuxt`, `opinionated` |  |
| <span id="vue-no-ref-object-reactivity-loss"></span>[`vue/no-ref-object-reactivity-loss`](./reference/vue-no-ref-object-reactivity-loss.md) | Interdit de copier la valeur d'une ref dans la même portée en perdant ses mises à jour réactives. | — |  |
| <span id="vue-no-reserved-component-names"></span>[`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | Interdire l’utilisation de noms réservés comme noms de composants. | `essential` |  |
| <span id="vue-no-root-v-if"></span>[`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | Interdire v-if sur l’unique élément racine d’un template. | `nuxt`, `opinionated` |  |
| <span id="vue-no-script-non-standard-lang"></span>[`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | Déconseiller les valeurs lang non standard dans les scripts. | `nuxt`, `opinionated` |  |
| <span id="vue-no-setup-props-reactivity-loss"></span>[`vue/no-setup-props-reactivity-loss`](./reference/vue-no-setup-props-reactivity-loss.md) | Interdit les copies des valeurs des props de setup qui perdent leurs mises à jour réactives. | — |  |
| <span id="vue-no-src-attribute"></span>[`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | Déconseiller l’attribut src sur les blocs SFC. | `nuxt`, `opinionated` |  |
| <span id="vue-no-static-inline-styles"></span>[`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | Interdire les attributs de style en ligne statiques. | — |  |
| <span id="vue-no-template-key"></span>[`vue/no-template-key`](./reference/vue-no-template-key.md) | Interdire l’attribut `key` sur `<template>`. | `essential` |  |
| <span id="vue-no-template-lang"></span>[`vue/no-template-lang`](./reference/vue-no-template-lang.md) | Déconseiller l’attribut lang sur le bloc template. | `nuxt`, `opinionated` |  |
| <span id="vue-no-template-shadow"></span>[`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | Interdire les noms de variables qui masquent des variables d’une portée externe. | `happy-path` |  |
| <span id="vue-no-template-target-blank"></span>[`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | Interdire target="_blank" sans rel="noopener noreferrer". | `happy-path` |  |
| <span id="vue-no-textarea-mustache"></span>[`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | Interdire l’interpolation à doubles accolades dans `<textarea>`. | `essential` |  |
| <span id="vue-no-this-in-before-route-enter"></span>[`vue/no-this-in-before-route-enter`](./reference/vue-no-this-in-before-route-enter.md) | Interdit d'accéder par `this` à une instance encore inexistante dans `beforeRouteEnter`. | `happy-path` |  |
| <span id="vue-no-undefined-refs"></span>[`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | Interdire les références à des variables non définies dans les templates. | `happy-path` |  |
| <span id="vue-no-unsafe-url"></span>[`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | Signaler les liaisons d’URL potentiellement dangereuses. | `essential` |  |
| <span id="vue-no-unsandboxed-iframe"></span>[`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | Exiger un attribut sandbox sur les éléments iframe. | `happy-path` |  |
| <span id="vue-no-unused-components"></span>[`vue/no-unused-components`](./reference/vue-no-unused-components.md) | Interdire l’enregistrement de composants inutilisés dans les templates. | `essential` |  |
| <span id="vue-no-unused-properties"></span>[`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | Interdire les propriétés inutilisées définies dans defineProps. | `happy-path` |  |
| <span id="vue-no-unused-refs"></span>[`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | Signaler les refs de template (ref="x") jamais référencées dans &lt;script&gt;. | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-setup-bindings"></span>[`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | Interdire les liaisons script setup qui ne sont jamais lues. | `happy-path` |  |
| <span id="vue-no-unused-vars"></span>[`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | Interdire les variables inutilisées définies dans les directives v-for et v-slot. | `essential` |  |
| <span id="vue-no-use-v-else-with-v-for"></span>[`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | Interdire `v-else-if` ou `v-else` sur le même élément que `v-for`. | `happy-path` |  |
| <span id="vue-no-use-v-if-with-v-for"></span>[`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | Interdire `v-if` sur le même élément que `v-for`. | `essential` |  |
| <span id="vue-no-useless-mustaches"></span>[`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | Interdire une interpolation à doubles accolades dont l’expression est une chaîne littérale constante. | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-template-attributes"></span>[`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | Interdire les attributs inutiles sur les éléments `<template>`. | `essential` |  |
| <span id="vue-no-useless-v-bind"></span>[`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | Interdire un v-bind dont la valeur est une simple chaîne littérale. | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-for-template-key"></span>[`vue/no-v-for-template-key`](./reference/vue-no-v-for-template-key.md) | Exige de placer la key de `template v-for` sur un enfant réel pour assurer la compatibilité avec Vue 2. | — |  |
| <span id="vue-no-v-for-template-key-on-child"></span>[`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | Interdire `key` sur l’enfant d’un `<template v-for>`. | `essential` |  |
| <span id="vue-no-v-html"></span>[`vue/no-v-html`](./reference/vue-no-v-html.md) | Déconseiller v-html pour prévenir les vulnérabilités XSS. | `essential` |  |
| <span id="vue-no-v-text"></span>[`vue/no-v-text`](./reference/vue-no-v-text.md) | Interdire la directive v-text ; préférer l’interpolation à doubles accolades. | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-text-v-html-on-component"></span>[`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | Interdire v-text / v-html sur les éléments de composants. | `essential` |  |
| <span id="vue-permitted-contents"></span>[`vue/permitted-contents`](./reference/vue-permitted-contents.md) | Faire respecter les règles du modèle de contenu HTML. | `happy-path` |  |
| <span id="vue-prefer-prop-type-boolean-first"></span>[`vue/prefer-prop-type-boolean-first`](./reference/vue-prefer-prop-type-boolean-first.md) | Exige que `Boolean` soit le premier constructeur dans la liste des types d'une prop. | — |  |
| <span id="vue-prefer-props-shorthand"></span>[`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | Recommander la syntaxe abrégée des props (Vue 3.4+). | `nuxt`, `opinionated` | ✓ |
| <span id="vue-prefer-true-attribute-shorthand"></span>[`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | Préférer la syntaxe abrégée pour un attribut booléen lié à `true`. | `nuxt`, `opinionated` |  |
| <span id="vue-prop-name-casing"></span>[`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | Imposer une casse aux noms des props déclarées. | `happy-path` |  |
| <span id="vue-require-component-is"></span>[`vue/require-component-is`](./reference/vue-require-component-is.md) | Exiger `v-bind:is` sur les éléments `<component>`. | `essential` |  |
| <span id="vue-require-component-registration"></span>[`vue/require-component-registration`](./reference/vue-require-component-registration.md) | Exiger un import ou un enregistrement explicite des composants. | `opinionated` |  |
| <span id="vue-require-expose"></span>[`vue/require-expose`](./reference/vue-require-expose.md) | Exige que les composants de l'Options API déclarent explicitement leurs propriétés publiques avec `expose`. | — |  |
| <span id="vue-require-scoped-style"></span>[`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | Exiger l’attribut scoped sur les balises style. | `happy-path` |  |
| <span id="vue-require-toggle-inside-transition"></span>[`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | Exiger un mécanisme de bascule sur l’élément enveloppé par `<transition>`. | `essential` |  |
| <span id="vue-require-v-for-key"></span>[`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | Exiger `v-bind:key` avec les directives `v-for`. | `essential` |  |
| <span id="vue-scoped-event-names"></span>[`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | Recommander des noms d’événements préfixés par leur contexte au format context:event. | `nuxt`, `opinionated` |  |
| <span id="vue-sfc-element-order"></span>[`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | Imposer un ordre cohérent des éléments de premier niveau d’un SFC. | `happy-path` |  |
| <span id="vue-single-style-block"></span>[`vue/single-style-block`](./reference/vue-single-style-block.md) | Recommander un seul bloc style. | `happy-path` |  |
| <span id="vue-slot-name-casing"></span>[`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | Imposer kebab-case aux slots nommés utilisés avec v-slot. | `nuxt`, `opinionated` |  |
| <span id="vue-this-in-template"></span>[`vue/this-in-template`](./reference/vue-this-in-template.md) | Interdire `this.` dans les expressions des templates. | `happy-path` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques. | `nuxt`, `opinionated` |  |
| <span id="vue-use-v-on-exact"></span>[`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | Imposer le modificateur `.exact` sur `v-on` en présence de gestionnaires utilisant des modificateurs. | `essential` |  |
| <span id="vue-v-bind-style"></span>[`vue/v-bind-style`](./reference/vue-v-bind-style.md) | Imposer un style de directive `v-bind`. | `happy-path` | ✓ |
| <span id="vue-v-on-event-hyphenation"></span>[`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | Imposer des tirets dans les noms d’événements personnalisés de v-on sur les composants. | `happy-path` | ✓ |
| <span id="vue-v-on-handler-style"></span>[`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | Imposer une référence de méthode ou une fonction en ligne pour les gestionnaires v-on. | `nuxt`, `opinionated` |  |
| <span id="vue-v-on-style"></span>[`vue/v-on-style`](./reference/vue-v-on-style.md) | Imposer un style de directive `v-on`. | `happy-path` | ✓ |
| <span id="vue-v-slot-style"></span>[`vue/v-slot-style`](./reference/vue-v-slot-style.md) | Imposer un style de directive `v-slot`. | `happy-path` | ✓ |
| <span id="vue-valid-attribute-name"></span>[`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | Exiger des noms d’attributs valides. | `essential` |  |
| <span id="vue-valid-model-definition"></span>[`vue/valid-model-definition`](./reference/vue-valid-model-definition.md) | Exige des clés valides dans l'option `model` de Vue 2. | — |  |
| <span id="vue-valid-template-root"></span>[`vue/valid-template-root`](./reference/vue-valid-template-root.md) | Exiger une racine `<template>` valide selon la sémantique des fragments de Vue 3. | `essential` |  |
| <span id="vue-valid-v-bind"></span>[`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | Exiger des directives `v-bind` valides. | `essential` |  |
| <span id="vue-valid-v-cloak"></span>[`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | Exiger des directives `v-cloak` valides. | `essential` |  |
| <span id="vue-valid-v-else"></span>[`vue/valid-v-else`](./reference/vue-valid-v-else.md) | Exiger des directives `v-else` valides. | `essential` | ✓ |
| <span id="vue-valid-v-else-if"></span>[`vue/valid-v-else-if`](./reference/vue-valid-v-else-if.md) | Exiger des directives `v-else-if` valides. | `essential` |  |
| <span id="vue-valid-v-for"></span>[`vue/valid-v-for`](./reference/vue-valid-v-for.md) | Exiger des directives `v-for` valides. | `essential` |  |
| <span id="vue-valid-v-html"></span>[`vue/valid-v-html`](./reference/vue-valid-v-html.md) | Exiger des directives `v-html` valides. | `essential` |  |
| <span id="vue-valid-v-if"></span>[`vue/valid-v-if`](./reference/vue-valid-v-if.md) | Exiger des directives `v-if` valides. | `essential` |  |
| <span id="vue-valid-v-is"></span>[`vue/valid-v-is`](./reference/vue-valid-v-is.md) | Exiger des directives `v-is` valides. | `essential` |  |
| <span id="vue-valid-v-memo"></span>[`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | Exiger des directives `v-memo` valides. | `essential` |  |
| <span id="vue-valid-v-model"></span>[`vue/valid-v-model`](./reference/vue-valid-v-model.md) | Exiger des directives `v-model` valides. | `essential` |  |
| <span id="vue-valid-v-on"></span>[`vue/valid-v-on`](./reference/vue-valid-v-on.md) | Exiger des directives `v-on` valides. | `essential` |  |
| <span id="vue-valid-v-once"></span>[`vue/valid-v-once`](./reference/vue-valid-v-once.md) | Exiger des directives `v-once` valides. | `essential` |  |
| <span id="vue-valid-v-pre"></span>[`vue/valid-v-pre`](./reference/vue-valid-v-pre.md) | Exiger des directives `v-pre` valides. | `essential` |  |
| <span id="vue-valid-v-show"></span>[`vue/valid-v-show`](./reference/vue-valid-v-show.md) | Exiger des directives `v-show` valides. | `essential` |  |
| <span id="vue-valid-v-slot"></span>[`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | Exiger des directives `v-slot` valides. | `essential` |  |
| <span id="vue-valid-v-text"></span>[`vue/valid-v-text`](./reference/vue-valid-v-text.md) | Exiger des directives `v-text` valides. | `essential` |  |
| <span id="vue-warn-custom-block"></span>[`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | Signaler les blocs personnalisés dans les fichiers SFC. | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-directive"></span>[`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | Signaler les directives personnalisées nécessitant un enregistrement. | `nuxt`, `opinionated` |  |

## Règles de type et de script (83)

| Règle | Ce qu’elle vérifie | Préréglage | Correction automatique |
| --- | --- | --- | --- |
| <span id="script-component-options-name-casing"></span>[`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | Imposer PascalCase à l’option `name` du composant. | `opinionated`, `nuxt` |  |
| <span id="script-custom-event-name-casing"></span>[`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | Imposer la casse configurée aux noms des événements personnalisés émis. | `opinionated`, `nuxt` |  |
| <span id="script-define-emits-declaration"></span>[`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | Imposer la forme typée defineEmits&lt;{}&gt;() à la place de la forme à l’exécution sous forme de tableau. | `opinionated`, `nuxt` |  |
| <span id="script-define-macros-order"></span>[`script/define-macros-order`](./reference/script-define-macros-order.md) | Imposer un ordre cohérent aux macros du compilateur Vue dans &lt;script setup&gt;. | `opinionated`, `nuxt` |  |
| <span id="script-define-props-declaration"></span>[`script/define-props-declaration`](./reference/script-define-props-declaration.md) | Imposer la forme typée defineProps&lt;{ ... }&gt;() à la place de la forme à l’exécution sous forme d’objet. | `opinionated`, `nuxt` |  |
| <span id="script-define-props-destructuring"></span>[`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | Imposer un style cohérent de déstructuration de defineProps dans &lt;script setup&gt;. | — |  |
| <span id="script-no-arrow-functions-in-watch"></span>[`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | Interdire les fonctions fléchées comme gestionnaires watch de l’Options API. | `essential` |  |
| <span id="script-no-async-in-computed"></span>[`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | Interdire les fonctions asynchrones dans les propriétés calculées. | `essential` |  |
| <span id="script-no-boolean-default"></span>[`script/no-boolean-default`](./reference/script-no-boolean-default.md) | Interdire une valeur par défaut sur une prop Boolean. | `opinionated`, `nuxt` |  |
| <span id="script-no-computed-properties-in-data"></span>[`script/no-computed-properties-in-data`](./reference/script-no-computed-properties-in-data.md) | Interdire l’usage d’une propriété calculée dans la fonction data du composant. | `essential` |  |
| <span id="script-no-deep-destructure-in-props"></span>[`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | Interdire la déstructuration profondément imbriquée dans defineProps. | `opinionated`, `nuxt` |  |
| <span id="script-no-deprecated-data-object-declaration"></span>[`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | Interdire un objet littéral comme option data du composant (Vue 3 exige une fonction). | `essential` |  |
| <span id="script-no-deprecated-delete-set"></span>[`script/no-deprecated-delete-set`](./reference/script-no-deprecated-delete-set.md) | Interdire $set et $delete de Vue 2, retirés dans Vue 3. | `essential` |  |
| <span id="script-no-deprecated-destroyed-lifecycle"></span>[`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | Interdire les hooks de cycle de vie dépréciés destroyed et beforeDestroy. | `essential` | ✓ |
| <span id="script-no-deprecated-dollar-listeners-api"></span>[`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | Interdire la propriété d’instance $listeners supprimée dans Vue 3 (fusionnée dans $attrs). | `essential` |  |
| <span id="script-no-deprecated-dollar-scopedslots-api"></span>[`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | Interdire la propriété d’instance $scopedSlots supprimée dans Vue 3 (utiliser $slots). | `essential` |  |
| <span id="script-no-deprecated-events-api"></span>[`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | Interdire l’API d’événements de Vue 2 supprimée ($on / $off / $once). | `essential` |  |
| <span id="script-no-deprecated-model-definition"></span>[`script/no-deprecated-model-definition`](./reference/script-no-deprecated-model-definition.md) | Interdire l’option model de Vue 2, retirée dans Vue 3. | `essential` |  |
| <span id="script-no-deprecated-props-default-this"></span>[`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | Interdire `this` dans une fonction de valeur par défaut ou de validation de prop (supprimé dans Vue 3). | `essential` |  |
| <span id="script-no-deprecated-vue-config-keycodes"></span>[`script/no-deprecated-vue-config-keycodes`](./reference/script-no-deprecated-vue-config-keycodes.md) | Interdire Vue.config.keyCodes, retiré dans Vue 3. | `essential` |  |
| <span id="script-no-dupe-keys"></span>[`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | Interdire les clés dupliquées entre props/data/computed/methods/setup/inject de l’Options API. | `essential` |  |
| <span id="script-no-duplicate-attr-inheritance"></span>[`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | Signaler un composant qui applique deux fois ses attributs transmis automatiquement. | `essential` |  |
| <span id="script-no-export-in-script-setup"></span>[`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | Interdire les instructions export dans &lt;script setup&gt;. | `essential` |  |
| <span id="script-no-expose-after-await"></span>[`script/no-expose-after-await`](./reference/script-no-expose-after-await.md) | Interdire expose() et defineExpose() après un await. | `essential` |  |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Interdire getCurrentInstance() en mode Vapor (renvoie null). | `opinionated` |  |
| <span id="script-no-import-compiler-macros"></span>[`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | Interdire l’import des macros du compilateur Vue importées automatiquement. | `essential` |  |
| <span id="script-no-internal-imports"></span>[`script/no-internal-imports`](./reference/script-no-internal-imports.md) | Interdire les imports depuis les modules internes de Vue. | `essential` |  |
| <span id="script-no-lifecycle-after-await"></span>[`script/no-lifecycle-after-await`](./reference/script-no-lifecycle-after-await.md) | Interdire l’enregistrement de hooks de cycle de vie après un await dans setup(). | `essential` |  |
| <span id="script-no-multiple-slot-args"></span>[`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | Interdire de passer plusieurs arguments à un appel de fonction de slot à portée. | `happy-path` |  |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Interdire l’utilisation de nextTick() dans les composants destinés à Vapor. | — |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Interdire les formes de l’Options API en mode Vapor. | `opinionated` |  |
| <span id="script-no-potential-component-option-typo"></span>[`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | Signaler les fautes de frappe probables dans les noms d’options de composant de l’Options API. | `essential` |  |
| <span id="script-no-reactive-destructure"></span>[`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | Interdire la déstructuration d’objets réactifs qui fait perdre la réactivité. | — |  |
| <span id="script-no-ref-as-operand"></span>[`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | Exiger l’accès via `.value` aux variables liées à une ref lorsqu’elles servent d’opérande. | `essential` |  |
| <span id="script-no-required-prop-with-default"></span>[`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | Interdire une prop qui possède à la fois required: true et une valeur par défaut. | `essential` |  |
| <span id="script-no-reserved-identifiers"></span>[`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | Interdire les identifiants réservés du compilateur Vue. | `essential` |  |
| <span id="script-no-reserved-keys"></span>[`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | Interdire les noms réservés par Vue comme clés de props/data/computed/methods/setup/inject de l’Options API. | `essential` |  |
| <span id="script-no-reserved-props"></span>[`script/no-reserved-props`](./reference/script-no-reserved-props.md) | Interdire les noms réservés dans la déclaration des props d’un composant. | `essential` |  |
| <span id="script-no-restricted-globals"></span>[`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | Interdire les références aux variables globales de l’environnement d’exécution qui doivent passer par une couche d’encapsulation typée. | — |  |
| <span id="script-no-restricted-members"></span>[`script/no-restricted-members`](./reference/script-no-restricted-members.md) | Interdire les accès aux membres object.property configurés par le projet. | — |  |
| <span id="script-no-side-effects-in-computed-properties"></span>[`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | Interdire les effets de bord dans les getters calculés de l’Options API. | `essential` |  |
| <span id="script-no-top-level-ref-in-script"></span>[`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | Interdire ref/reactive au niveau supérieur pour éviter la contamination de l’état entre requêtes. | — |  |
| <span id="script-no-unstable-nested-components"></span>[`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | Interdire les définitions de composants dans les fonctions setup ou de rendu. | `happy-path` |  |
| <span id="script-no-unused-emit-declarations"></span>[`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | Signaler les événements déclarés qui ne sont jamais émis. | `opinionated`, `nuxt` |  |
| <span id="script-no-use-computed-property-like-method"></span>[`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | Interdire d’appeler une propriété calculée de l’Options API comme une méthode. | `essential` |  |
| <span id="script-no-watch-after-await"></span>[`script/no-watch-after-await`](./reference/script-no-watch-after-await.md) | Interdire les observateurs créés après un await dans setup() sans conserver leur fonction d’arrêt. | `essential` |  |
| <span id="script-no-with-defaults"></span>[`script/no-with-defaults`](./reference/script-no-with-defaults.md) | Déconseiller withDefaults au profit des valeurs par défaut dans la déstructuration (Vue 3.5+). | `opinionated` |  |
| <span id="script-one-component-per-file"></span>[`script/one-component-per-file`](./reference/script-one-component-per-file.md) | Interdire plus d’une définition de composant dans un fichier. | `opinionated`, `nuxt` |  |
| <span id="script-order-in-components"></span>[`script/order-in-components`](./reference/script-order-in-components.md) | Imposer l’ordre par défaut des propriétés des options de composant. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-computed"></span>[`script/prefer-computed`](./reference/script-prefer-computed.md) | Préférer computed() pour l’état réactif dérivé. | — |  |
| <span id="script-prefer-define-emits-binding"></span>[`script/prefer-define-emits-binding`](./reference/script-prefer-define-emits-binding.md) | Préférer la fonction capturée de defineEmits au $emit du template. | — | ✓ |
| <span id="script-prefer-define-options"></span>[`script/prefer-define-options`](./reference/script-prefer-define-options.md) | Préférer defineOptions() à un &lt;script&gt; ordinaire qui ne définit que name/inheritAttrs. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-import-from-vue"></span>[`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | Préférer les imports depuis 'vue' plutôt que depuis les packages internes. | `essential` | ✓ |
| <span id="script-prefer-ref-over-reactive"></span>[`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | Recommander ref() plutôt que reactive() pour gérer l’état. | — |  |
| <span id="script-prefer-use-attrs"></span>[`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | Recommander useAttrs() plutôt que context.attrs. | — |  |
| <span id="script-prefer-use-id"></span>[`script/prefer-use-id`](./reference/script-prefer-use-id.md) | Recommander useId() pour générer des identifiants uniques (Vue 3.5+). | — |  |
| <span id="script-prefer-use-slots"></span>[`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | Recommander useSlots() plutôt que context.slots. | — |  |
| <span id="script-prefer-use-template-ref"></span>[`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | Recommander useTemplateRef plutôt que ref pour les références de template (Vue 3.5+). | `opinionated`, `nuxt` |  |
| <span id="script-require-default-prop"></span>[`script/require-default-prop`](./reference/script-require-default-prop.md) | Exiger une valeur par défaut pour chaque prop facultative non booléenne. | `opinionated`, `nuxt` |  |
| <span id="script-require-define-component"></span>[`script/require-define-component`](./reference/script-require-define-component.md) | Exiger defineComponent() pour les exports par défaut des SFC TypeScript. | `happy-path` |  |
| <span id="script-require-explicit-emits"></span>[`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | Exiger la déclaration des événements émis dans defineEmits ou l’option emits. | `happy-path` |  |
| <span id="script-require-explicit-slots"></span>[`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | Exiger que les slots utilisés via useSlots() soient explicitement typés avec defineSlots&lt;...&gt;(). | `opinionated`, `nuxt` |  |
| <span id="script-require-function-return-type"></span>[`script/require-function-return-type`](./reference/script-require-function-return-type.md) | Exiger des annotations de type de retour sur les fonctions. | — |  |
| <span id="script-require-prop-type-constructor"></span>[`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | Exiger que les valeurs `type` des props soient des constructeurs plutôt que des chaînes littérales. | `essential` |  |
| <span id="script-require-prop-types"></span>[`script/require-prop-types`](./reference/script-require-prop-types.md) | Exiger que chaque prop déclare un type. | `opinionated` |  |
| <span id="script-require-render-return"></span>[`script/require-render-return`](./reference/script-require-render-return.md) | Exiger que la fonction render du composant renvoie une valeur. | `essential` |  |
| <span id="script-require-slots-as-functions"></span>[`script/require-slots-as-functions`](./reference/script-require-slots-as-functions.md) | Exiger que les propriétés de $slots soient appelées comme des fonctions. | `essential` |  |
| <span id="script-require-symbol-provide"></span>[`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | Recommander Symbol comme clé d’injection pour provide/inject. | — |  |
| <span id="script-require-typed-object-prop"></span>[`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | Exiger un type explicite sur une prop dont le type à l’exécution est `Object` ou `Array`. | `opinionated`, `nuxt` |  |
| <span id="script-require-typed-ref"></span>[`script/require-typed-ref`](./reference/script-require-typed-ref.md) | Exiger un argument de type explicite sur un ref() initialisé sans valeur, avec null ou avec undefined. | — |  |
| <span id="script-require-valid-default-prop"></span>[`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | Exiger que la valeur par défaut d’une prop soit valide pour son type déclaré. | `essential` |  |
| <span id="script-return-in-computed-property"></span>[`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | Exiger une valeur de retour dans chaque getter calculé. | `essential` |  |
| <span id="script-return-in-emits-validator"></span>[`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | Exiger une valeur de retour dans chaque validateur emits de l’Options API. | `essential` |  |
| <span id="script-valid-define-emits"></span>[`script/valid-define-emits`](./reference/script-valid-define-emits.md) | Imposer une utilisation valide de defineEmits() (pas d’arguments de type et d’exécution combinés, pas de références locales, un seul appel). | `essential` |  |
| <span id="script-valid-define-options"></span>[`script/valid-define-options`](./reference/script-valid-define-options.md) | Imposer une utilisation valide de defineOptions() (un seul argument objet, sans props/emits/expose/slots). | `essential` |  |
| <span id="script-valid-define-props"></span>[`script/valid-define-props`](./reference/script-valid-define-props.md) | Imposer une utilisation valide de defineProps() (un seul appel, pas d’arguments de type et d’exécution combinés, pas de références locales). | `essential` |  |
| <span id="script-valid-next-tick"></span>[`script/valid-next-tick`](./reference/script-valid-next-tick.md) | Exiger que le résultat d’un appel nextTick() soit attendu, chaîné ou associé à un callback. | `essential` |  |
| <span id="type-no-floating-promises"></span>[`type/no-floating-promises`](./reference/type-no-floating-promises.md) | Interdire les Promises laissées sans traitement. | `nuxt`, `opinionated` |  |
| <span id="type-no-reactivity-loss"></span>[`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | Interdire les instantanés ordinaires de valeurs réactives lors des affectations et des appels. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-template-binding"></span>[`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | Interdire les liaisons de template dont le type résolu est non sûr. | `nuxt`, `opinionated` |  |
| <span id="type-require-typed-emits"></span>[`type/require-typed-emits`](./reference/type-require-typed-emits.md) | Exiger une définition de type pour defineEmits. | `happy-path` |  |
| <span id="type-require-typed-props"></span>[`type/require-typed-props`](./reference/type-require-typed-props.md) | Exiger une définition de type pour defineProps. | `happy-path` |  |
| <span id="type-strict-boolean-expressions"></span>[`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | Exiger des expressions booléennes sûres dans les conditions des scripts et des templates. | — |  |

## Règles HTML (9)

| Règle | Ce qu’elle vérifie | Préréglage | Correction automatique |
| --- | --- | --- | --- |
| <span id="html-deprecated-attr"></span>[`html/deprecated-attr`](./reference/html-deprecated-attr.md) | Interdire les attributs HTML obsolètes. | `happy-path` |  |
| <span id="html-deprecated-element"></span>[`html/deprecated-element`](./reference/html-deprecated-element.md) | Interdire les éléments HTML obsolètes. | `happy-path` |  |
| <span id="html-id-duplication"></span>[`html/id-duplication`](./reference/html-id-duplication.md) | Interdire les identifiants d’éléments en double. | `essential` |  |
| <span id="html-no-consecutive-br"></span>[`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | Interdire les éléments &lt;br&gt; consécutifs. | `happy-path` |  |
| <span id="html-no-dupe-style-properties"></span>[`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | Interdire les propriétés en double dans les attributs de style en ligne. | `nuxt`, `opinionated` |  |
| <span id="html-no-duplicate-class"></span>[`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | Interdire les noms de classes en double dans un attribut class statique. | `happy-path` |  |
| <span id="html-no-duplicate-dt"></span>[`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | Interdire les noms &lt;dt&gt; en double dans &lt;dl&gt;. | `happy-path` |  |
| <span id="html-no-empty-palpable-content"></span>[`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | Interdire les éléments vides qui attendent un contenu visible. | `happy-path` |  |
| <span id="html-require-datetime"></span>[`html/require-datetime`](./reference/html-require-datetime.md) | Exiger l’attribut datetime sur l’élément &lt;time&gt;. | `happy-path` |  |

## Règles d’accessibilité (30)

| Règle | Ce qu’elle vérifie | Préréglage | Correction automatique |
| --- | --- | --- | --- |
| <span id="a11y-alt-text"></span>[`a11y/alt-text`](./reference/a11y-alt-text.md) | Exiger un texte alternatif pour les éléments multimédias. | `happy-path` |  |
| <span id="a11y-anchor-has-content"></span>[`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | Exiger un contenu accessible pour les éléments de lien. | `happy-path` |  |
| <span id="a11y-anchor-is-valid"></span>[`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | Imposer un href valide sur les éléments de lien. | `happy-path` |  |
| <span id="a11y-aria-props"></span>[`a11y/aria-props`](./reference/a11y-aria-props.md) | Interdire les attributs ARIA invalides. | `happy-path` |  |
| <span id="a11y-aria-role"></span>[`a11y/aria-role`](./reference/a11y-aria-role.md) | Exiger un rôle ARIA valide et non abstrait pour les éléments possédant un rôle ARIA. | `happy-path` |  |
| <span id="a11y-aria-unsupported-elements"></span>[`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | Interdire les attributs ARIA sur les éléments qui ne les prennent pas en charge. | `happy-path` |  |
| <span id="a11y-click-events-have-key-events"></span>[`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | Exiger des gestionnaires de clavier avec les événements de clic. | `happy-path` |  |
| <span id="a11y-form-control-has-label"></span>[`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | Exiger des libellés associés aux contrôles de formulaire. | `happy-path` |  |
| <span id="a11y-heading-has-content"></span>[`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | Exiger un contenu accessible pour les éléments de titre. | `happy-path` |  |
| <span id="a11y-heading-levels"></span>[`a11y/heading-levels`](./reference/a11y-heading-levels.md) | Interdire de sauter des niveaux de titre. | `nuxt`, `opinionated` |  |
| <span id="a11y-iframe-has-title"></span>[`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | Exiger un attribut title sur les éléments iframe. | `happy-path` |  |
| <span id="a11y-img-alt"></span>[`a11y/img-alt`](./reference/a11y-img-alt.md) | Exiger un attribut alt sur les images pour les rendre accessibles. | — |  |
| <span id="a11y-interactive-supports-focus"></span>[`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | Exiger que les éléments possédant un rôle interactif puissent recevoir le focus. | `happy-path` |  |
| <span id="a11y-label-has-for"></span>[`a11y/label-has-for`](./reference/a11y-label-has-for.md) | Exiger des contrôles de formulaire associés aux libellés. | `happy-path` |  |
| <span id="a11y-landmark-roles"></span>[`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | Valider l’emplacement et l’unicité des rôles de zones de repère. | `nuxt`, `opinionated` |  |
| <span id="a11y-media-has-caption"></span>[`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | Exiger des sous-titres pour les éléments multimédias. | `happy-path` |  |
| <span id="a11y-mouse-events-have-key-events"></span>[`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | Exiger des événements de focus et de perte du focus avec les événements de souris. | `happy-path` |  |
| <span id="a11y-no-access-key"></span>[`a11y/no-access-key`](./reference/a11y-no-access-key.md) | Interdire l’utilisation de l’attribut accesskey. | `happy-path` |  |
| <span id="a11y-no-aria-hidden-on-focusable"></span>[`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | Interdire aria-hidden="true" sur les éléments pouvant recevoir le focus. | `happy-path` |  |
| <span id="a11y-no-autofocus"></span>[`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | Interdire l’utilisation de l’attribut autofocus. | `happy-path` |  |
| <span id="a11y-no-distracting-elements"></span>[`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | Interdire les éléments distrayants tels que &lt;marquee&gt; et &lt;blink&gt;. | `happy-path` |  |
| <span id="a11y-no-i-for-icon"></span>[`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | Interdire l’élément &lt;i&gt; pour les icônes. | `happy-path` |  |
| <span id="a11y-no-redundant-roles"></span>[`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | Interdire les rôles ARIA redondants. | `happy-path` | ✓ |
| <span id="a11y-no-refer-to-non-existent-id"></span>[`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | Interdire les références à des identifiants inexistants. | `happy-path` |  |
| <span id="a11y-no-role-presentation-on-focusable"></span>[`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | Interdire role="presentation" ou role="none" sur les éléments pouvant recevoir le focus. | `happy-path` |  |
| <span id="a11y-no-static-element-interactions"></span>[`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | Interdire les gestionnaires d’événements sur les éléments statiques. | `happy-path` |  |
| <span id="a11y-placeholder-label-option"></span>[`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | Exiger une option d’invite dans un select required. | `nuxt`, `opinionated` |  |
| <span id="a11y-role-has-required-aria-props"></span>[`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | Exiger les propriétés obligatoires des rôles ARIA. | `happy-path` |  |
| <span id="a11y-tabindex-no-positive"></span>[`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | Interdire les valeurs positives de tabindex. | `happy-path` |  |
| <span id="a11y-use-list"></span>[`a11y/use-list`](./reference/a11y-use-list.md) | Suggérer des éléments de liste pour les textes ressemblant à des listes à puces. | `nuxt`, `opinionated` |  |

## Règles SSR (2)

| Règle | Ce qu’elle vérifie | Préréglage | Correction automatique |
| --- | --- | --- | --- |
| <span id="ssr-no-browser-globals-in-ssr"></span>[`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | Interdire les variables globales propres au navigateur dans un contexte SSR. | `happy-path` |  |
| <span id="ssr-no-hydration-mismatch"></span>[`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | Interdire les valeurs non déterministes qui causent des divergences d’hydratation. | `happy-path` |  |

## Règles Vapor (4)

| Règle | Ce qu’elle vérifie | Préréglage | Correction automatique |
| --- | --- | --- | --- |
| <span id="vapor-no-inline-template"></span>[`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | Interdire l’attribut obsolète inline-template. | `nuxt`, `opinionated` |  |
| <span id="vapor-no-vue-lifecycle-events"></span>[`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor). | `happy-path` |  |
| <span id="vapor-prefer-static-class"></span>[`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales. | `nuxt`, `opinionated` | ✓ |
| <span id="vapor-require-vapor-attribute"></span>[`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | Suggérer l’ajout de l’attribut vapor à script setup. | `nuxt`, `opinionated` |  |

## Règles de l’écosystème et de Nuxt (13)

| Règle | Ce qu’elle vérifie | Préréglage | Correction automatique |
| --- | --- | --- | --- |
| <span id="ecosystem-nuxt-prefer-nuxt-link"></span>[`ecosystem/nuxt-prefer-nuxt-link`](./reference/ecosystem-nuxt-prefer-nuxt-link.md) | Préférer NuxtLink pour les liens internes à l’application. | `nuxt` |  |
| <span id="ecosystem-pinia-prefer-store-to-refs"></span>[`ecosystem/pinia-prefer-store-to-refs`](./reference/ecosystem-pinia-prefer-store-to-refs.md) | Préférer storeToRefs() lors de la déstructuration des stores Pinia. | `ecosystem` |  |
| <span id="ecosystem-router-link-require-to"></span>[`ecosystem/router-link-require-to`](./reference/ecosystem-router-link-require-to.md) | Exiger une destination `to` sur les composants RouterLink et NuxtLink. | `ecosystem` |  |
| <span id="ecosystem-void-link-require-href"></span>[`ecosystem/void-link-require-href`](./reference/ecosystem-void-link-require-href.md) | Exiger `href` sur les composants Link de Void Vue. | `ecosystem` |  |
| <span id="ecosystem-void-link-valid-method"></span>[`ecosystem/void-link-valid-method`](./reference/ecosystem-void-link-valid-method.md) | Valider les props method statiques des composants Link de Void Vue. | `ecosystem` |  |
| <span id="ecosystem-vue-i18n-no-missing-key"></span>[`ecosystem/vue-i18n-no-missing-key`](./reference/ecosystem-vue-i18n-no-missing-key.md) | Signaler les clés vue-i18n statiques absentes des messages locaux du SFC. | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-link"></span>[`ecosystem/vue-router-prefer-named-link`](./reference/ecosystem-vue-router-prefer-named-link.md) | Préférer les objets de routes nommées aux chaînes de chemin statiques dans RouterLink. | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-push"></span>[`ecosystem/vue-router-prefer-named-push`](./reference/ecosystem-vue-router-prefer-named-push.md) | Préférer les objets de routes nommées pour la navigation programmatique de Vue Router. | `ecosystem` |  |
| <span id="ecosystem-vue-test-utils-no-html-snapshot"></span>[`ecosystem/vue-test-utils-no-html-snapshot`](./reference/ecosystem-vue-test-utils-no-html-snapshot.md) | Éviter les instantanés de wrapper.html() dans les tests Vue Test Utils. | `ecosystem` |  |
| <span id="nuxt-no-nuxt-config-test-key"></span>[`nuxt/no-nuxt-config-test-key`](./reference/nuxt-no-nuxt-config-test-key.md) | Interdire la clé `test` dans la configuration Nuxt. | `nuxt` |  |
| <span id="nuxt-no-page-meta-runtime-values"></span>[`nuxt/no-page-meta-runtime-values`](./reference/nuxt-no-page-meta-runtime-values.md) | Interdire les valeurs du contexte d’exécution évaluées immédiatement dans `definePageMeta`, extrait dans un chunk séparé à la compilation et exécuté avant le setup du composant. | `nuxt` |  |
| <span id="nuxt-nuxt-config-keys-order"></span>[`nuxt/nuxt-config-keys-order`](./reference/nuxt-nuxt-config-keys-order.md) | Préférer l’ordre recommandé des propriétés de configuration Nuxt. | `nuxt` | ✓ |
| <span id="nuxt-prefer-import-meta"></span>[`nuxt/prefer-import-meta`](./reference/nuxt-prefer-import-meta.md) | Préférer `import.meta.*` à `process.*`. | `nuxt` | ✓ |

## Règles CSS (10)

| Règle | Ce qu’elle vérifie | Préréglage | Correction automatique |
| --- | --- | --- | --- |
| <span id="css-no-display-none"></span>[`css/no-display-none`](./reference/css-no-display-none.md) | Suggérer v-show plutôt que display: none. | `opinionated`, `nuxt` |  |
| <span id="css-no-hardcoded-values"></span>[`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | Suggérer des variables CSS plutôt que des valeurs codées en dur. | `opinionated`, `nuxt` |  |
| <span id="css-no-id-selectors"></span>[`css/no-id-selectors`](./reference/css-no-id-selectors.md) | Déconseiller les sélecteurs d’identifiant en CSS. | `opinionated`, `nuxt` |  |
| <span id="css-no-important"></span>[`css/no-important`](./reference/css-no-important.md) | Déconseiller !important en CSS. | `opinionated`, `nuxt` |  |
| <span id="css-no-utility-classes"></span>[`css/no-utility-classes`](./reference/css-no-utility-classes.md) | Déconseiller l’implémentation de classes utilitaires dans les styles des composants. | `opinionated`, `nuxt` |  |
| <span id="css-no-v-bind-performance"></span>[`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | Signaler le coût de performance de v-bind() en CSS. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-logical-properties"></span>[`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | Recommander les propriétés logiques CSS pour mieux prendre en charge l’internationalisation. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-nested-selectors"></span>[`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | Recommander l’imbrication CSS pour les sélecteurs de descendants. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-slotted"></span>[`css/prefer-slotted`](./reference/css-prefer-slotted.md) | Recommander ::v-slotted() pour styliser le contenu des slots. | `opinionated`, `nuxt` |  |
| <span id="css-require-font-display"></span>[`css/require-font-display`](./reference/css-require-font-display.md) | Exiger font-display dans les règles @font-face. | `opinionated`, `nuxt` |  |

## Règles Musea (6)

| Règle | Ce qu’elle vérifie | Préréglage | Correction automatique |
| --- | --- | --- | --- |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | Interdire les blocs &lt;variant&gt; vides. | — |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | Préférer les variables CSS de design tokens aux valeurs primitives codées en dur. | — |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | Exiger l’attribut component dans le bloc &lt;art&gt;. | — |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | Exiger l’attribut title dans le bloc &lt;art&gt;. | — |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | Exiger des noms de variants uniques. | — |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | Exiger un attribut name dans les blocs &lt;variant&gt;. | — |  |

## Règles petite-vue (3)

| Règle | Ce qu’elle vérifie | Préréglage | Correction automatique |
| --- | --- | --- | --- |
| <span id="petite-vue-no-unsupported-directive"></span>[`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | Interdire les directives non prises en charge par petite-vue. | — |  |
| <span id="petite-vue-valid-v-effect"></span>[`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | Exiger une expression non vide pour v-effect. | — |  |
| <span id="petite-vue-valid-v-scope"></span>[`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | Exiger que v-scope reçoive un objet littéral. | — |  |

## Règles inter-fichiers (66)

Activez ces vérifications avec `crossFile: true` dans `vue.linter`, ou lancez `vize lint --cross-file`. Les diagnostics s’affichent sous la forme `vize:croquis/cf/<name>` ; configurez-les dans `vue.linter.rules` sous la forme `croquis/cf/<name>`, sans `vize:`.

| Règle | Ce qu’elle vérifie | État |
| --- | --- | --- |
| <span id="ecosystem-vue-router-extra-param"></span>[`ecosystem/vue-router-extra-param`](./project/ecosystem-vue-router-extra-param.md) | Signale les paramètres de navigation que la route cible ne déclare pas. | Signalée par `vp run lint` |
| <span id="ecosystem-vue-router-missing-param"></span>[`ecosystem/vue-router-missing-param`](./project/ecosystem-vue-router-missing-param.md) | Signale une navigation nommée qui omet un paramètre de route obligatoire. | Signalée par `vp run lint` |
| <span id="ecosystem-vue-router-param-type"></span>[`ecosystem/vue-router-param-type`](./project/ecosystem-vue-router-param-type.md) | Signale une valeur de paramètre de route que le segment déclaré n’accepte pas. | Signalée par `vp run lint` |
| <span id="ecosystem-vue-router-unknown-route"></span>[`ecosystem/vue-router-unknown-route`](./project/ecosystem-vue-router-unknown-route.md) | Signale un nom de navigation que le routeur installé ne déclare pas. | Signalée par `vp run lint` |
| <span id="html-cross-component-nesting"></span>[`html/cross-component-nesting`](./project/html-cross-component-nesting.md) | Vérifier l’imbrication HTML réelle après la composition des composants importés. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-array-mutation"></span>[`vize:croquis/cf/array-mutation`](./project/vize-croquis-cf-array-mutation.md) | Utilisez des mutations observables pour mettre à jour les indices d’un tableau dans Vue 2.7. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-async-boundary"></span>[`vize:croquis/cf/async-boundary`](./project/vize-croquis-cf-async-boundary.md) | Un état réactif traverse une frontière asynchrone et peut être observé dans un état périmé. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-async-no-suspense"></span>[`vize:croquis/cf/async-no-suspense`](./project/vize-croquis-cf-async-no-suspense.md) | Un composant asynchrone est rendu sans frontière Suspense. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-browser-api-ssr"></span>[`vize:croquis/cf/browser-api-ssr`](./project/vize-croquis-cf-browser-api-ssr.md) | Une API réservée au navigateur est utilisée alors que le composant peut être rendu sur le serveur. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-circular-dep"></span>[`vize:croquis/cf/circular-dep`](./project/vize-croquis-cf-circular-dep.md) | Des composants s’importent mutuellement en formant un cycle. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-circular-reactive-dependency"></span>[`vize:croquis/cf/circular-reactive-dependency`](./project/vize-croquis-cf-circular-reactive-dependency.md) | Des calculs réactifs dépendent les uns des autres en formant un cycle. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-closure-captures-reactive"></span>[`vize:croquis/cf/closure-captures-reactive`](./project/vize-croquis-cf-closure-captures-reactive.md) | Une fermeture capture une valeur réactive et ne verra pas les mises à jour ultérieures. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-composable-outside-setup"></span>[`vize:croquis/cf/composable-outside-setup`](./project/vize-croquis-cf-composable-outside-setup.md) | Un composable est appelé en dehors de `setup`. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-computed-side-effects"></span>[`vize:croquis/cf/computed-side-effects`](./project/vize-croquis-cf-computed-side-effects.md) | L’accesseur d’une propriété calculée écrit dans l’état ou produit un autre effet de bord. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-deep-import"></span>[`vize:croquis/cf/deep-import`](./project/vize-croquis-cf-deep-import.md) | Une chaîne d’imports est plus profonde que ce que le projet autorise. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-destructuring-breaks-reactivity"></span>[`vize:croquis/cf/destructuring-breaks-reactivity`](./project/vize-croquis-cf-destructuring-breaks-reactivity.md) | La déstructuration d’un objet réactif copie ses champs et supprime leur suivi. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-di-outside-setup"></span>[`vize:croquis/cf/di-outside-setup`](./project/vize-croquis-cf-di-outside-setup.md) | `provide` ou `inject` est appelé en dehors de `setup`. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-dom-access-without-next-tick"></span>[`vize:croquis/cf/dom-access-without-next-tick`](./project/vize-croquis-cf-dom-access-without-next-tick.md) | Le DOM est lu avant que Vue ait appliqué la mise à jour. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-duplicate-id"></span>[`vize:croquis/cf/duplicate-id`](./project/vize-croquis-cf-duplicate-id.md) | Le même identifiant d’élément est utilisé dans plusieurs composants. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-event-listener-leak"></span>[`vize:croquis/cf/event-listener-leak`](./project/vize-croquis-cf-event-listener-leak.md) | Un écouteur d’événement est enregistré sans jamais être supprimé. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-event-modifier"></span>[`vize:croquis/cf/event-modifier`](./project/vize-croquis-cf-event-modifier.md) | Un écouteur d’événement utilise un modificateur que l’événement émis ne prend pas en charge. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-hydration-risk"></span>[`vize:croquis/cf/hydration-risk`](./project/vize-croquis-cf-hydration-risk.md) | Le rendu peut diverger entre le serveur et le client. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-inherit-attrs-unused"></span>[`vize:croquis/cf/inherit-attrs-unused`](./project/vize-croquis-cf-inherit-attrs-unused.md) | `inheritAttrs: false` est défini et le composant ne lit jamais les attributs. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-inject-without-symbol"></span>[`vize:croquis/cf/inject-without-symbol`](./project/vize-croquis-cf-inject-without-symbol.md) | `inject` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-injected-async-mutation-race"></span>[`vize:croquis/cf/injected-async-mutation-race`](./project/vize-croquis-cf-injected-async-mutation-race.md) | Une valeur injectée est modifiée par une tâche asynchrone susceptible de provoquer une condition de concurrence. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-lifecycle-outside-setup"></span>[`vize:croquis/cf/lifecycle-outside-setup`](./project/vize-croquis-cf-lifecycle-outside-setup.md) | Un hook de cycle de vie est enregistré en dehors de `setup`. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-lifecycle-without-cleanup"></span>[`vize:croquis/cf/lifecycle-without-cleanup`](./project/vize-croquis-cf-lifecycle-without-cleanup.md) | Un hook de cycle de vie lance un travail sans jamais le nettoyer. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-missing-required-prop"></span>[`vize:croquis/cf/missing-required-prop`](./project/vize-croquis-cf-missing-required-prop.md) | Une prop obligatoire n’est pas transmise. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-missing-suspense"></span>[`vize:croquis/cf/missing-suspense`](./project/vize-croquis-cf-missing-suspense.md) | Une dépendance asynchrone est utilisée en dehors d’une frontière Suspense. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-module-scope-reactive"></span>[`vize:croquis/cf/module-scope-reactive`](./project/vize-croquis-cf-module-scope-reactive.md) | Un état réactif est créé au niveau du module et partagé par tous les appelants. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-multi-root-attrs"></span>[`vize:croquis/cf/multi-root-attrs`](./project/vize-croquis-cf-multi-root-attrs.md) | Un composant à plusieurs racines reçoit des attributs sans avoir d’endroit où les placer. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-mutated-after-escape"></span>[`vize:croquis/cf/mutated-after-escape`](./project/vize-croquis-cf-mutated-after-escape.md) | Un objet réactif est modifié après avoir échappé à son propriétaire. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-non-reactive-provide"></span>[`vize:croquis/cf/non-reactive-provide`](./project/vize-croquis-cf-non-reactive-provide.md) | Une valeur fournie n’est pas réactive ; les descendants ne verront donc pas les mises à jour. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-non-unique-id"></span>[`vize:croquis/cf/non-unique-id`](./project/vize-croquis-cf-non-unique-id.md) | L’identifiant d’un élément dans une boucle n’est pas unique pour chaque entrée. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-object-identity-comparison"></span>[`vize:croquis/cf/object-identity-comparison`](./project/vize-croquis-cf-object-identity-comparison.md) | Comparez les identifiants stables des enregistrements représentés par des objets bruts et des proxies réactifs. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-pinia-getter"></span>[`vize:croquis/cf/pinia-getter`](./project/vize-croquis-cf-pinia-getter.md) | Évitez de copier les valeurs des getters Pinia sous forme d’instantanés pendant setup. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-prop-type-mismatch"></span>[`vize:croquis/cf/prop-type-mismatch`](./project/vize-croquis-cf-prop-type-mismatch.md) | La valeur d’une prop transmise ne correspond pas au type déclaré. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-provide-inject-type"></span>[`vize:croquis/cf/provide-inject-type`](./project/vize-croquis-cf-provide-inject-type.md) | Une valeur fournie et son injection n’ont pas le même type. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-provide-without-symbol"></span>[`vize:croquis/cf/provide-without-symbol`](./project/vize-croquis-cf-provide-without-symbol.md) | `provide` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-reactive-export"></span>[`vize:croquis/cf/reactive-export`](./project/vize-croquis-cf-reactive-export.md) | Un état réactif est exporté depuis le module. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-reactivity-outside-setup"></span>[`vize:croquis/cf/reactivity-outside-setup`](./project/vize-croquis-cf-reactivity-outside-setup.md) | Une API réactive est appelée en dehors de `setup`. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-reassignment-breaks-reactivity"></span>[`vize:croquis/cf/reassignment-breaks-reactivity`](./project/vize-croquis-cf-reassignment-breaks-reactivity.md) | Réaffecter une liaison réactive la remplace par une valeur ordinaire. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-reference-escapes-scope"></span>[`vize:croquis/cf/reference-escapes-scope`](./project/vize-croquis-cf-reference-escapes-scope.md) | Une référence réactive échappe à la portée qui gère sa durée de vie. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-setup-context-violation"></span>[`vize:croquis/cf/setup-context-violation`](./project/vize-croquis-cf-setup-context-violation.md) | Créez l’état réactif propre à chaque composant dans le setup de son instance. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-shallow-deep-access"></span>[`vize:croquis/cf/shallow-deep-access`](./project/vize-croquis-cf-shallow-deep-access.md) | Une propriété profonde d’une valeur `shallowReactive` ou `shallowRef` est lue comme si elle était suivie. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-spread-breaks-reactivity"></span>[`vize:croquis/cf/spread-breaks-reactivity`](./project/vize-croquis-cf-spread-breaks-reactivity.md) | Décomposer un objet réactif avec l’opérateur spread copie ses valeurs et supprime leur suivi. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-suspense-no-fallback"></span>[`vize:croquis/cf/suspense-no-fallback`](./project/vize-croquis-cf-suspense-no-fallback.md) | `<Suspense>` n’a aucun contenu de secours. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-template-ref-timing"></span>[`vize:croquis/cf/template-ref-timing`](./project/vize-croquis-cf-template-ref-timing.md) | Une référence de template est lue avant le montage du composant. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-toraw-mutation"></span>[`vize:croquis/cf/toraw-mutation`](./project/vize-croquis-cf-toraw-mutation.md) | `toRaw` est utilisé, puis l’objet brut est modifié. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-uncaught-error"></span>[`vize:croquis/cf/uncaught-error`](./project/vize-croquis-cf-uncaught-error.md) | Un composant peut lever une exception sans qu’aucune frontière d’erreur ne la capture. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-undeclared-emit"></span>[`vize:croquis/cf/undeclared-emit`](./project/vize-croquis-cf-undeclared-emit.md) | Le composant émet un événement qui n’est pas déclaré. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-undeclared-prop"></span>[`vize:croquis/cf/undeclared-prop`](./project/vize-croquis-cf-undeclared-prop.md) | Un parent transmet une prop que l’enfant ne déclare pas. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-undefined-slot"></span>[`vize:croquis/cf/undefined-slot`](./project/vize-croquis-cf-undefined-slot.md) | Un parent remplit un slot que l’enfant n’expose pas. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-unhandled-event"></span>[`vize:croquis/cf/unhandled-event`](./project/vize-croquis-cf-unhandled-event.md) | Un enfant émet un événement qu’aucun parent ne gère. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-unmatched-inject"></span>[`vize:croquis/cf/unmatched-inject`](./project/vize-croquis-cf-unmatched-inject.md) | `inject` nomme une clé qu’aucun ancêtre ne fournit. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-unmatched-listener"></span>[`vize:croquis/cf/unmatched-listener`](./project/vize-croquis-cf-unmatched-listener.md) | Un parent écoute un événement que l’enfant n’émet pas. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-unregistered-component"></span>[`vize:croquis/cf/unregistered-component`](./project/vize-croquis-cf-unregistered-component.md) | Un template utilise un composant qui n’est ni enregistré ni importé. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-unresolved-import"></span>[`vize:croquis/cf/unresolved-import`](./project/vize-croquis-cf-unresolved-import.md) | Un import ne se résout pas vers un module. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-unused-attrs"></span>[`vize:croquis/cf/unused-attrs`](./project/vize-croquis-cf-unused-attrs.md) | Des attributs à transmettre automatiquement sont passés à un composant à plusieurs racines qui ne les utilise pas. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-unused-emit"></span>[`vize:croquis/cf/unused-emit`](./project/vize-croquis-cf-unused-emit.md) | Un événement déclaré n’est jamais émis. | Expérimentale ; pas encore signalée par le CLI |
| <span id="vize-croquis-cf-unused-provide"></span>[`vize:croquis/cf/unused-provide`](./project/vize-croquis-cf-unused-provide.md) | Une clé fournie n’est jamais injectée. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-value-extraction-breaks-reactivity"></span>[`vize:croquis/cf/value-extraction-breaks-reactivity`](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md) | Extraire une valeur réactive dans une variable locale fait perdre les mises à jour ultérieures. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-watch-can-be-computed"></span>[`vize:croquis/cf/watch-can-be-computed`](./project/vize-croquis-cf-watch-can-be-computed.md) | Un observateur ne fait que copier une valeur dans l’état et peut être remplacé par une propriété calculée. | Prévue ; pas encore signalée |
| <span id="vize-croquis-cf-watcheffect-async"></span>[`vize:croquis/cf/watcheffect-async`](./project/vize-croquis-cf-watcheffect-async.md) | Protégez le travail asynchrone de watchEffect contre l’invalidation et les réponses périmées. | Signalée par `vp run lint` |
| <span id="vize-croquis-cf-watcher-outside-setup"></span>[`vize:croquis/cf/watcher-outside-setup`](./project/vize-croquis-cf-watcher-outside-setup.md) | `watch` ou `watchEffect` est appelé en dehors de `setup`. | Prévue ; pas encore signalée |
| <span id="vue-cross-file-attrs-fallthrough"></span>[`vue/cross-file-attrs-fallthrough`](./project/vue-cross-file-attrs-fallthrough.md) | Un parent transmet des attributs à un enfant résolu dont la racine ne peut pas en hériter et qui n’utilise pas explicitement $attrs. | Signalée par `vp run lint` |

[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](/rules/options.md) · [Migrer depuis ESLint](/rules/migration.md)
