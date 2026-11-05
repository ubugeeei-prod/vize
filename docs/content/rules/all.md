---
title: "All lint rules"
description: "Every rule Vize's linter ships, grouped by category. Open a rule to see what it reports, a failing and a passing example, and how to turn it on."
---

# All lint rules

Every rule Vize's linter ships, grouped by category. Open a rule to see what it reports, a failing and a passing example, and how to turn it on.

The Preset column names the smallest preset that turns a rule on. Rule pages list every preset. See [presets](../rules/index.md#presets) for what each one contains.

## Vue rules (120)

| Rule | What it checks | Preset | Auto-fix |
| --- | --- | --- | --- |
| <span id="vue-a11y-img-alt"></span>[`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | Require alt attribute on images for accessibility. | — |  |
| <span id="vue-attribute-hyphenation"></span>[`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | Enforce attribute naming style on custom components. | `happy-path` | ✓ |
| <span id="vue-attribute-order"></span>[`vue/attribute-order`](./reference/vue-attribute-order.md) | Enforce a consistent order of attributes. | `happy-path` |  |
| <span id="vue-comment-directive"></span>[`vue/comment-directive`](./reference/vue-comment-directive.md) | Support comment directives in `<template>`. | `essential` |  |
| <span id="vue-component-definition-name-casing"></span>[`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | Enforce PascalCase or kebab-case for component definition names. | `happy-path` |  |
| <span id="vue-component-name-in-template-casing"></span>[`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | Enforce specific casing for component names in templates. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-html-button-has-type"></span>[`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | Require an explicit valid type on button elements. | `nuxt`, `opinionated` |  |
| <span id="vue-html-end-tags"></span>[`vue/html-end-tags`](./reference/vue-html-end-tags.md) | Require end tags on HTML elements that may omit them. | `happy-path` | ✓ |
| <span id="vue-html-quotes"></span>[`vue/html-quotes`](./reference/vue-html-quotes.md) | Enforce quotes style of HTML attributes. | `happy-path` | ✓ |
| <span id="vue-html-self-closing"></span>[`vue/html-self-closing`](./reference/vue-html-self-closing.md) | Enforce self-closing style. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-jsx-uses-vars"></span>[`vue/jsx-uses-vars`](./reference/vue-jsx-uses-vars.md) | Treat variables used in JSX element names as used. | `essential` |  |
| <span id="vue-match-component-file-name"></span>[`vue/match-component-file-name`](./reference/vue-match-component-file-name.md) | Require a component name to match its file name. | — |  |
| <span id="vue-max-template-complexity"></span>[`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | Limit a component's own template complexity (cyclomatic and cognitive). | — |  |
| <span id="vue-multi-word-component-names"></span>[`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | Require component names to be multi-word. | `essential` |  |
| <span id="vue-mustache-interpolation-spacing"></span>[`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | Enforce consistent spacing inside mustache interpolations. | `happy-path` | ✓ |
| <span id="vue-no-array-index-key"></span>[`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | Disallow using the v-for index variable directly as the :key. | `nuxt`, `opinionated` |  |
| <span id="vue-no-bare-strings-in-template"></span>[`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | Disallow raw human-readable text in the template that should be internationalized. | — |  |
| <span id="vue-no-boolean-attr-value"></span>[`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | Disallow explicit values for boolean HTML attributes. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-child-content"></span>[`vue/no-child-content`](./reference/vue-no-child-content.md) | Disallow child content when using v-html or v-text. | `essential` |  |
| <span id="vue-no-custom-modifiers-on-v-model"></span>[`vue/no-custom-modifiers-on-v-model`](./reference/vue-no-custom-modifiers-on-v-model.md) | Disallow custom modifiers on `v-model` used on a component. | — |  |
| <span id="vue-no-deprecated-filter"></span>[`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | Disallow deprecated Vue 2 filter syntax using the pipe operator. | `essential` |  |
| <span id="vue-no-deprecated-functional-template"></span>[`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | Disallow the `functional` attribute on the SFC `<template>`. | `essential` |  |
| <span id="vue-no-deprecated-html-element-is"></span>[`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | Disallow the `is` attribute on native HTML elements. | `essential` |  |
| <span id="vue-no-deprecated-inline-template"></span>[`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | Disallow the deprecated `inline-template` attribute. | `essential` |  |
| <span id="vue-no-deprecated-router-link-tag-prop"></span>[`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | Disallow the `tag` prop on &lt;router-link&gt;. | `essential` |  |
| <span id="vue-no-deprecated-scope-attribute"></span>[`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | Disallow the deprecated `scope` attribute on &lt;template&gt;. | `essential` |  |
| <span id="vue-no-deprecated-slot-attribute"></span>[`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | Disallow the deprecated `slot` attribute. | `essential` |  |
| <span id="vue-no-deprecated-slot-scope-attribute"></span>[`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | Disallow the deprecated `slot-scope` attribute. | `essential` |  |
| <span id="vue-no-deprecated-v-bind-sync"></span>[`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | Disallow the deprecated `.sync` modifier on `v-bind`. | `essential` |  |
| <span id="vue-no-deprecated-v-is"></span>[`vue/no-deprecated-v-is`](./reference/vue-no-deprecated-v-is.md) | Disallow the deprecated `v-is` directive. | `essential` |  |
| <span id="vue-no-deprecated-v-on-native-modifier"></span>[`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | Disallow the deprecated `.native` modifier on `v-on`. | `essential` |  |
| <span id="vue-no-deprecated-v-on-number-modifiers"></span>[`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | Disallow deprecated numeric `keyCode` modifiers on `v-on`. | `essential` | ✓ |
| <span id="vue-no-dupe-v-else-if"></span>[`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | Disallow duplicate conditions in `v-if` / `v-else-if` chains. | `essential` |  |
| <span id="vue-no-duplicate-attributes"></span>[`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | Disallow duplicate attributes on the same element. | `essential` |  |
| <span id="vue-no-empty-component-block"></span>[`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | Disallow empty SFC blocks. | `nuxt`, `opinionated` |  |
| <span id="vue-no-inline-style"></span>[`vue/no-inline-style`](./reference/vue-no-inline-style.md) | Discourage use of inline style attributes. | `nuxt`, `opinionated` |  |
| <span id="vue-no-invalid-html-attribute"></span>[`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | Disallow invalid static values for HTML attributes. | `happy-path` |  |
| <span id="vue-no-lone-template"></span>[`vue/no-lone-template`](./reference/vue-no-lone-template.md) | Disallow unnecessary `<template>` elements. | `happy-path` |  |
| <span id="vue-no-multi-spaces"></span>[`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | Disallow multiple consecutive spaces. | `happy-path` | ✓ |
| <span id="vue-no-multiple-objects-in-class"></span>[`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | Disallow multiple object literals inside a :class array binding. | `nuxt`, `opinionated` |  |
| <span id="vue-no-multiple-template-root"></span>[`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | Disallow multiple root nodes in a template. | — |  |
| <span id="vue-no-mutating-props"></span>[`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | Disallow mutating component props. | `essential` |  |
| <span id="vue-no-negated-v-if-condition"></span>[`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | Disallow a negated v-if condition when the chain has a v-else. | `nuxt`, `opinionated` |  |
| <span id="vue-no-non-component-keep-alive-child"></span>[`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | Disallow plain element wrappers directly below `<KeepAlive>`. | `essential` |  |
| <span id="vue-no-preprocessor-lang"></span>[`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | Discourage CSS preprocessor usage in favor of modern CSS. | `nuxt`, `opinionated` |  |
| <span id="vue-no-ref-object-reactivity-loss"></span>[`vue/no-ref-object-reactivity-loss`](./reference/vue-no-ref-object-reactivity-loss.md) | Disallow copying a ref value in the same scope. | — |  |
| <span id="vue-no-reserved-component-names"></span>[`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | Disallow the use of reserved names as component names. | `essential` |  |
| <span id="vue-no-root-v-if"></span>[`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | Disallow v-if on the single root element of a template. | `nuxt`, `opinionated` |  |
| <span id="vue-no-script-non-standard-lang"></span>[`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | Discourage non-standard script lang values. | `nuxt`, `opinionated` |  |
| <span id="vue-no-setup-props-reactivity-loss"></span>[`vue/no-setup-props-reactivity-loss`](./reference/vue-no-setup-props-reactivity-loss.md) | Disallow setup prop reads that lose reactivity. | — |  |
| <span id="vue-no-src-attribute"></span>[`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | Discourage src attribute on SFC blocks. | `nuxt`, `opinionated` |  |
| <span id="vue-no-static-inline-styles"></span>[`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | Disallow static inline style attributes. | — |  |
| <span id="vue-no-template-key"></span>[`vue/no-template-key`](./reference/vue-no-template-key.md) | Disallow `key` attribute on `<template>`. | `essential` |  |
| <span id="vue-no-template-lang"></span>[`vue/no-template-lang`](./reference/vue-no-template-lang.md) | Discourage lang attribute on template block. | `nuxt`, `opinionated` |  |
| <span id="vue-no-template-shadow"></span>[`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | Disallow variable names that shadow variables in outer scope. | `happy-path` |  |
| <span id="vue-no-template-target-blank"></span>[`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | Disallow target="_blank" without rel="noopener noreferrer". | `happy-path` |  |
| <span id="vue-no-textarea-mustache"></span>[`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | Disallow mustache interpolation in `<textarea>`. | `essential` |  |
| <span id="vue-no-this-in-before-route-enter"></span>[`vue/no-this-in-before-route-enter`](./reference/vue-no-this-in-before-route-enter.md) | Disallow `this` in `beforeRouteEnter`. | — |  |
| <span id="vue-no-undefined-refs"></span>[`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | Disallow undefined variable references in templates. | — |  |
| <span id="vue-no-unsafe-url"></span>[`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | Warn about potentially unsafe URL bindings. | `essential` |  |
| <span id="vue-no-unsandboxed-iframe"></span>[`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | Require a sandbox attribute on iframe elements. | `happy-path` |  |
| <span id="vue-no-unused-components"></span>[`vue/no-unused-components`](./reference/vue-no-unused-components.md) | Disallow registering components that are not used inside templates. | `essential` |  |
| <span id="vue-no-unused-properties"></span>[`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | Disallow unused properties defined in defineProps. | `happy-path` |  |
| <span id="vue-no-unused-refs"></span>[`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | Report template refs (ref="x") never referenced in &lt;script&gt;. | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-setup-bindings"></span>[`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | Disallow unread script setup bindings. | — |  |
| <span id="vue-no-unused-vars"></span>[`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | Disallow unused variable definitions in v-for and v-slot directives. | `essential` |  |
| <span id="vue-no-use-v-else-with-v-for"></span>[`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | Disallow using `v-else-if` or `v-else` on the same element as `v-for`. | `happy-path` |  |
| <span id="vue-no-use-v-if-with-v-for"></span>[`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | Disallow using `v-if` on the same element as `v-for`. | `essential` |  |
| <span id="vue-no-useless-mustaches"></span>[`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | Disallow a mustache interpolation whose expression is a constant string literal. | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-template-attributes"></span>[`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | Disallow useless attributes on `<template>` elements. | `essential` |  |
| <span id="vue-no-useless-v-bind"></span>[`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | Disallow a v-bind whose value is a plain string literal. | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-for-template-key"></span>[`vue/no-v-for-template-key`](./reference/vue-no-v-for-template-key.md) | Disallow `key` on `<template v-for>`. | — |  |
| <span id="vue-no-v-for-template-key-on-child"></span>[`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | Disallow `key` on the child of a `<template v-for>`. | `essential` |  |
| <span id="vue-no-v-html"></span>[`vue/no-v-html`](./reference/vue-no-v-html.md) | Warn against v-html to prevent XSS vulnerabilities. | `essential` |  |
| <span id="vue-no-v-text"></span>[`vue/no-v-text`](./reference/vue-no-v-text.md) | Disallow the v-text directive; prefer mustache interpolation. | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-text-v-html-on-component"></span>[`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | Disallow v-text / v-html on component elements. | `essential` |  |
| <span id="vue-permitted-contents"></span>[`vue/permitted-contents`](./reference/vue-permitted-contents.md) | Enforce HTML content model rules. | `happy-path` |  |
| <span id="vue-prefer-prop-type-boolean-first"></span>[`vue/prefer-prop-type-boolean-first`](./reference/vue-prefer-prop-type-boolean-first.md) | Enforce `Boolean` comes first in component prop types. | — |  |
| <span id="vue-prefer-props-shorthand"></span>[`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | Recommend shorthand syntax for props (Vue 3.4+). | `nuxt`, `opinionated` | ✓ |
| <span id="vue-prefer-true-attribute-shorthand"></span>[`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | Prefer the shorthand for a boolean attribute bound to `true`. | `nuxt`, `opinionated` |  |
| <span id="vue-prop-name-casing"></span>[`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | Enforce a casing for declared prop names. | `happy-path` |  |
| <span id="vue-require-component-is"></span>[`vue/require-component-is`](./reference/vue-require-component-is.md) | Require `v-bind:is` on `<component>` elements. | `essential` |  |
| <span id="vue-require-component-registration"></span>[`vue/require-component-registration`](./reference/vue-require-component-registration.md) | Require explicit import or registration for components. | `opinionated` |  |
| <span id="vue-require-expose"></span>[`vue/require-expose`](./reference/vue-require-expose.md) | Require declare public properties using `expose`. | — |  |
| <span id="vue-require-scoped-style"></span>[`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | Require scoped attribute on style tags. | `happy-path` |  |
| <span id="vue-require-toggle-inside-transition"></span>[`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | Require a toggle on the element wrapped by `<transition>`. | `essential` |  |
| <span id="vue-require-v-for-key"></span>[`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | Require `v-bind:key` with `v-for` directives. | `essential` |  |
| <span id="vue-scoped-event-names"></span>[`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | Recommend scoped event names using context:event format. | `nuxt`, `opinionated` |  |
| <span id="vue-sfc-element-order"></span>[`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | Enforce consistent order of SFC top-level elements. | `happy-path` |  |
| <span id="vue-single-style-block"></span>[`vue/single-style-block`](./reference/vue-single-style-block.md) | Recommend having a single style block. | `happy-path` |  |
| <span id="vue-slot-name-casing"></span>[`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | Enforce kebab-case for named slots used via v-slot. | `nuxt`, `opinionated` |  |
| <span id="vue-this-in-template"></span>[`vue/this-in-template`](./reference/vue-this-in-template.md) | Disallow `this.` in template expressions. | `happy-path` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | Enforce unique element IDs using useId() instead of static literals. | `nuxt`, `opinionated` |  |
| <span id="vue-use-v-on-exact"></span>[`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | Enforce `.exact` modifier on `v-on` when there are modifier-based handlers. | `essential` |  |
| <span id="vue-v-bind-style"></span>[`vue/v-bind-style`](./reference/vue-v-bind-style.md) | Enforce `v-bind` directive style. | `happy-path` | ✓ |
| <span id="vue-v-on-event-hyphenation"></span>[`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | Enforce hyphenation of custom event names in v-on on components. | `happy-path` | ✓ |
| <span id="vue-v-on-handler-style"></span>[`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | Enforce writing v-on handlers as a method reference or an inline function. | `nuxt`, `opinionated` |  |
| <span id="vue-v-on-style"></span>[`vue/v-on-style`](./reference/vue-v-on-style.md) | Enforce `v-on` directive style. | `happy-path` | ✓ |
| <span id="vue-v-slot-style"></span>[`vue/v-slot-style`](./reference/vue-v-slot-style.md) | Enforce `v-slot` directive style. | `happy-path` | ✓ |
| <span id="vue-valid-attribute-name"></span>[`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | Require valid attribute names. | `essential` |  |
| <span id="vue-valid-model-definition"></span>[`vue/valid-model-definition`](./reference/vue-valid-model-definition.md) | Require valid keys in the model option. | — |  |
| <span id="vue-valid-template-root"></span>[`vue/valid-template-root`](./reference/vue-valid-template-root.md) | Enforce a valid `<template>` root for Vue 3 fragment semantics. | `essential` |  |
| <span id="vue-valid-v-bind"></span>[`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | Enforce valid `v-bind` directives. | `essential` |  |
| <span id="vue-valid-v-cloak"></span>[`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | Enforce valid `v-cloak` directives. | `essential` |  |
| <span id="vue-valid-v-else"></span>[`vue/valid-v-else`](./reference/vue-valid-v-else.md) | Enforce valid `v-else` directives. | `essential` | ✓ |
| <span id="vue-valid-v-else-if"></span>[`vue/valid-v-else-if`](./reference/vue-valid-v-else-if.md) | Enforce valid `v-else-if` directives. | `essential` |  |
| <span id="vue-valid-v-for"></span>[`vue/valid-v-for`](./reference/vue-valid-v-for.md) | Enforce valid `v-for` directives. | `essential` |  |
| <span id="vue-valid-v-html"></span>[`vue/valid-v-html`](./reference/vue-valid-v-html.md) | Enforce valid `v-html` directives. | `essential` |  |
| <span id="vue-valid-v-if"></span>[`vue/valid-v-if`](./reference/vue-valid-v-if.md) | Enforce valid `v-if` directives. | `essential` |  |
| <span id="vue-valid-v-is"></span>[`vue/valid-v-is`](./reference/vue-valid-v-is.md) | Enforce valid `v-is` directives. | `essential` |  |
| <span id="vue-valid-v-memo"></span>[`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | Enforce valid `v-memo` directives. | `essential` |  |
| <span id="vue-valid-v-model"></span>[`vue/valid-v-model`](./reference/vue-valid-v-model.md) | Enforce valid `v-model` directives. | `essential` |  |
| <span id="vue-valid-v-on"></span>[`vue/valid-v-on`](./reference/vue-valid-v-on.md) | Enforce valid `v-on` directives. | `essential` |  |
| <span id="vue-valid-v-once"></span>[`vue/valid-v-once`](./reference/vue-valid-v-once.md) | Enforce valid `v-once` directives. | `essential` |  |
| <span id="vue-valid-v-pre"></span>[`vue/valid-v-pre`](./reference/vue-valid-v-pre.md) | Enforce valid `v-pre` directives. | `essential` |  |
| <span id="vue-valid-v-show"></span>[`vue/valid-v-show`](./reference/vue-valid-v-show.md) | Enforce valid `v-show` directives. | `essential` |  |
| <span id="vue-valid-v-slot"></span>[`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | Enforce valid `v-slot` directives. | `essential` |  |
| <span id="vue-valid-v-text"></span>[`vue/valid-v-text`](./reference/vue-valid-v-text.md) | Enforce valid `v-text` directives. | `essential` |  |
| <span id="vue-warn-custom-block"></span>[`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | Warn about custom blocks in SFC files. | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-directive"></span>[`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | Warn about custom directives that need registration. | `nuxt`, `opinionated` |  |

## Type and script rules (82)

| Rule | What it checks | Preset | Auto-fix |
| --- | --- | --- | --- |
| <span id="script-component-options-name-casing"></span>[`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | Enforce PascalCase for the component `name` option. | `opinionated`, `nuxt` |  |
| <span id="script-custom-event-name-casing"></span>[`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | Enforce configured casing for emitted custom event names. | `opinionated`, `nuxt` |  |
| <span id="script-define-emits-declaration"></span>[`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | Enforce the type-based defineEmits&lt;{}&gt;() form over the runtime/array form. | `opinionated`, `nuxt` |  |
| <span id="script-define-macros-order"></span>[`script/define-macros-order`](./reference/script-define-macros-order.md) | Enforce a consistent order of the Vue compiler macros in &lt;script setup&gt;. | `opinionated`, `nuxt` |  |
| <span id="script-define-props-declaration"></span>[`script/define-props-declaration`](./reference/script-define-props-declaration.md) | Enforce type-based defineProps&lt;{ ... }&gt;() over the runtime/object form. | `opinionated`, `nuxt` |  |
| <span id="script-define-props-destructuring"></span>[`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | Enforce consistent style for defineProps destructuring in &lt;script setup&gt;. | — |  |
| <span id="script-no-arrow-functions-in-watch"></span>[`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | Disallow arrow functions as Options API watch handlers. | `essential` |  |
| <span id="script-no-async-in-computed"></span>[`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | Disallow async functions in computed properties. | `essential` |  |
| <span id="script-no-boolean-default"></span>[`script/no-boolean-default`](./reference/script-no-boolean-default.md) | Disallow a default on a Boolean prop. | `opinionated`, `nuxt` |  |
| <span id="script-no-computed-properties-in-data"></span>[`script/no-computed-properties-in-data`](./reference/script-no-computed-properties-in-data.md) | Disallow using computed properties inside the component data function. | `essential` |  |
| <span id="script-no-deep-destructure-in-props"></span>[`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | Disallow deeply nested destructuring in defineProps. | `opinionated`, `nuxt` |  |
| <span id="script-no-deprecated-data-object-declaration"></span>[`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | Disallow an object literal as the component data option (Vue 3 requires a function). | `essential` |  |
| <span id="script-no-deprecated-delete-set"></span>[`script/no-deprecated-delete-set`](./reference/script-no-deprecated-delete-set.md) | Disallow Vue 2 $set and $delete, which were removed in Vue 3. | `essential` |  |
| <span id="script-no-deprecated-destroyed-lifecycle"></span>[`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | Disallow deprecated destroyed and beforeDestroy lifecycle hooks. | `essential` | ✓ |
| <span id="script-no-deprecated-dollar-listeners-api"></span>[`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | Disallow the $listeners instance property removed in Vue 3 (merged into $attrs). | `essential` |  |
| <span id="script-no-deprecated-dollar-scopedslots-api"></span>[`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | Disallow the $scopedSlots instance property removed in Vue 3 (use $slots). | `essential` |  |
| <span id="script-no-deprecated-events-api"></span>[`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | Disallow the removed Vue 2 events API ($on / $off / $once). | `essential` |  |
| <span id="script-no-deprecated-model-definition"></span>[`script/no-deprecated-model-definition`](./reference/script-no-deprecated-model-definition.md) | Disallow the Vue 2 model option, which was removed in Vue 3. | `essential` |  |
| <span id="script-no-deprecated-props-default-this"></span>[`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | Disallow `this` inside a prop default/validator function (removed in Vue 3). | `essential` |  |
| <span id="script-no-deprecated-vue-config-keycodes"></span>[`script/no-deprecated-vue-config-keycodes`](./reference/script-no-deprecated-vue-config-keycodes.md) | Disallow Vue.config.keyCodes, which was removed in Vue 3. | `essential` |  |
| <span id="script-no-dupe-keys"></span>[`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | Disallow duplicate keys across Options API props/data/computed/methods/setup/inject. | `essential` |  |
| <span id="script-no-duplicate-attr-inheritance"></span>[`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | Flag a component that applies its fallthrough attributes twice. | `essential` |  |
| <span id="script-no-export-in-script-setup"></span>[`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | Disallow export statements inside &lt;script setup&gt;. | `essential` |  |
| <span id="script-no-expose-after-await"></span>[`script/no-expose-after-await`](./reference/script-no-expose-after-await.md) | Disallow expose() and defineExpose() after an await. | `essential` |  |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Disallow getCurrentInstance() in Vapor mode (returns null). | `opinionated` |  |
| <span id="script-no-import-compiler-macros"></span>[`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | Disallow importing Vue compiler macros that are auto-imported. | `essential` |  |
| <span id="script-no-internal-imports"></span>[`script/no-internal-imports`](./reference/script-no-internal-imports.md) | Disallow importing from Vue internal modules. | `essential` |  |
| <span id="script-no-lifecycle-after-await"></span>[`script/no-lifecycle-after-await`](./reference/script-no-lifecycle-after-await.md) | Disallow lifecycle hooks registered after an await in setup(). | `essential` |  |
| <span id="script-no-multiple-slot-args"></span>[`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | Disallow passing more than one argument to a scoped-slot function call. | `happy-path` |  |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Disallow nextTick() usage in Vapor-oriented components. | — |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Disallow Options API patterns in Vapor mode. | `opinionated` |  |
| <span id="script-no-potential-component-option-typo"></span>[`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | Flag likely typos in Options API component option names. | `essential` |  |
| <span id="script-no-reactive-destructure"></span>[`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | Disallow destructuring reactive objects which loses reactivity. | — |  |
| <span id="script-no-ref-as-operand"></span>[`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | Require ref-bound variables to be accessed via `.value` when used as an operand. | `essential` |  |
| <span id="script-no-required-prop-with-default"></span>[`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | Disallow a prop that is both required: true and has a default. | `essential` |  |
| <span id="script-no-reserved-identifiers"></span>[`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | Disallow using Vue compiler reserved identifiers. | `essential` |  |
| <span id="script-no-reserved-keys"></span>[`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | Disallow Vue-reserved names as Options API props/data/computed/methods/setup keys. | `essential` |  |
| <span id="script-no-reserved-props"></span>[`script/no-reserved-props`](./reference/script-no-reserved-props.md) | Disallow reserved names in a component's props declaration. | `essential` |  |
| <span id="script-no-restricted-globals"></span>[`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | Disallow references to runtime-environment globals that must go through a typed wrapper. | — |  |
| <span id="script-no-restricted-members"></span>[`script/no-restricted-members`](./reference/script-no-restricted-members.md) | Disallow project-configured object.property member accesses. | — |  |
| <span id="script-no-side-effects-in-computed-properties"></span>[`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | Disallow side effects in Options API computed getters. | `essential` |  |
| <span id="script-no-top-level-ref-in-script"></span>[`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | Disallow top-level ref/reactive to prevent Cross-Request State Pollution. | — |  |
| <span id="script-no-unstable-nested-components"></span>[`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | Disallow component definitions inside setup or render functions. | `happy-path` |  |
| <span id="script-no-unused-emit-declarations"></span>[`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | Flag declared events that are never emitted. | `opinionated`, `nuxt` |  |
| <span id="script-no-use-computed-property-like-method"></span>[`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | Disallow calling an Options API computed property like a method. | `essential` |  |
| <span id="script-no-watch-after-await"></span>[`script/no-watch-after-await`](./reference/script-no-watch-after-await.md) | Disallow watchers registered after an await in setup() without keeping their stop handle. | `essential` |  |
| <span id="script-no-with-defaults"></span>[`script/no-with-defaults`](./reference/script-no-with-defaults.md) | Discourage withDefaults in favor of destructuring defaults (Vue 3.5+). | `opinionated` |  |
| <span id="script-one-component-per-file"></span>[`script/one-component-per-file`](./reference/script-one-component-per-file.md) | Disallow more than one component definition in a file. | `opinionated`, `nuxt` |  |
| <span id="script-order-in-components"></span>[`script/order-in-components`](./reference/script-order-in-components.md) | Enforce the default order of properties in component options. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-computed"></span>[`script/prefer-computed`](./reference/script-prefer-computed.md) | Prefer computed() for derived reactive state. | — |  |
| <span id="script-prefer-define-emits-binding"></span>[`script/prefer-define-emits-binding`](./reference/script-prefer-define-emits-binding.md) | Prefer the captured defineEmits function over template $emit. | — | ✓ |
| <span id="script-prefer-define-options"></span>[`script/prefer-define-options`](./reference/script-prefer-define-options.md) | Prefer defineOptions() over a plain &lt;script&gt; that only sets name/inheritAttrs. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-import-from-vue"></span>[`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | Prefer importing from 'vue' instead of internal packages. | `essential` | ✓ |
| <span id="script-prefer-ref-over-reactive"></span>[`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | Recommend using ref() over reactive() for state management. | — |  |
| <span id="script-prefer-use-attrs"></span>[`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | Recommend using useAttrs() over context.attrs. | — |  |
| <span id="script-prefer-use-id"></span>[`script/prefer-use-id`](./reference/script-prefer-use-id.md) | Recommend using useId() for generating unique IDs (Vue 3.5+). | — |  |
| <span id="script-prefer-use-slots"></span>[`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | Recommend using useSlots() over context.slots. | — |  |
| <span id="script-prefer-use-template-ref"></span>[`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | Recommend useTemplateRef over ref for template references (Vue 3.5+). | `opinionated`, `nuxt` |  |
| <span id="script-require-default-prop"></span>[`script/require-default-prop`](./reference/script-require-default-prop.md) | Require a default value for every optional, non-Boolean prop. | `opinionated`, `nuxt` |  |
| <span id="script-require-explicit-emits"></span>[`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | Require emitted events to be declared in defineEmits or the emits option. | `happy-path` |  |
| <span id="script-require-explicit-slots"></span>[`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | Require slots consumed via useSlots() to be explicitly typed with defineSlots&lt;...&gt;(). | `opinionated`, `nuxt` |  |
| <span id="script-require-function-return-type"></span>[`script/require-function-return-type`](./reference/script-require-function-return-type.md) | Require return type annotations on functions. | — |  |
| <span id="script-require-prop-type-constructor"></span>[`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | Require prop `type` values to be constructors rather than string literals. | `essential` |  |
| <span id="script-require-prop-types"></span>[`script/require-prop-types`](./reference/script-require-prop-types.md) | Require every prop to declare a type. | `opinionated` |  |
| <span id="script-require-render-return"></span>[`script/require-render-return`](./reference/script-require-render-return.md) | Require the component render function to return a value. | `essential` |  |
| <span id="script-require-slots-as-functions"></span>[`script/require-slots-as-functions`](./reference/script-require-slots-as-functions.md) | Require $slots properties to be called as functions. | `essential` |  |
| <span id="script-require-symbol-provide"></span>[`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | Recommend using Symbol as injection key for provide/inject. | — |  |
| <span id="script-require-typed-object-prop"></span>[`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | Require an explicit type on a prop whose runtime type is `Object` or `Array`. | `opinionated`, `nuxt` |  |
| <span id="script-require-typed-ref"></span>[`script/require-typed-ref`](./reference/script-require-typed-ref.md) | Require an explicit type argument on a ref() initialized with no value, null, or undefined. | — |  |
| <span id="script-require-valid-default-prop"></span>[`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | Require a prop's default value to be valid for its declared type. | `essential` |  |
| <span id="script-return-in-computed-property"></span>[`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | Require a return value in every computed getter. | `essential` |  |
| <span id="script-return-in-emits-validator"></span>[`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | Require a return value in every Vue emits validator. | `essential` |  |
| <span id="script-valid-define-emits"></span>[`script/valid-define-emits`](./reference/script-valid-define-emits.md) | Enforce valid defineEmits() usage (no type+runtime args, no local references, single call). | `essential` |  |
| <span id="script-valid-define-options"></span>[`script/valid-define-options`](./reference/script-valid-define-options.md) | Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots). | `essential` |  |
| <span id="script-valid-define-props"></span>[`script/valid-define-props`](./reference/script-valid-define-props.md) | Enforce valid defineProps() usage (single call, not both type and runtime args, no local references). | `essential` |  |
| <span id="script-valid-next-tick"></span>[`script/valid-next-tick`](./reference/script-valid-next-tick.md) | Require the result of a nextTick() call to be awaited, chained, or given a callback. | `essential` |  |
| <span id="type-no-floating-promises"></span>[`type/no-floating-promises`](./reference/type-no-floating-promises.md) | Disallow floating (unhandled) Promises. | `nuxt`, `opinionated` |  |
| <span id="type-no-reactivity-loss"></span>[`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | Disallow plain snapshots of reactive values across assignments and calls. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-template-binding"></span>[`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | Disallow template bindings that resolve to unsafe types. | `nuxt`, `opinionated` |  |
| <span id="type-require-typed-emits"></span>[`type/require-typed-emits`](./reference/type-require-typed-emits.md) | Require type definition for defineEmits. | `happy-path` |  |
| <span id="type-require-typed-props"></span>[`type/require-typed-props`](./reference/type-require-typed-props.md) | Require type definition for defineProps. | `happy-path` |  |
| <span id="type-strict-boolean-expressions"></span>[`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | Require safe boolean expressions in script and template conditions. | — |  |

## HTML rules (9)

| Rule | What it checks | Preset | Auto-fix |
| --- | --- | --- | --- |
| <span id="html-deprecated-attr"></span>[`html/deprecated-attr`](./reference/html-deprecated-attr.md) | Disallow deprecated HTML attributes. | `happy-path` |  |
| <span id="html-deprecated-element"></span>[`html/deprecated-element`](./reference/html-deprecated-element.md) | Disallow deprecated HTML elements. | `happy-path` |  |
| <span id="html-id-duplication"></span>[`html/id-duplication`](./reference/html-id-duplication.md) | Disallow duplicate element IDs. | `essential` |  |
| <span id="html-no-consecutive-br"></span>[`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | Disallow consecutive &lt;br&gt; elements. | `happy-path` |  |
| <span id="html-no-dupe-style-properties"></span>[`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | Disallow duplicate properties in inline style attributes. | `nuxt`, `opinionated` |  |
| <span id="html-no-duplicate-class"></span>[`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | Disallow duplicate class names in a static class attribute. | `happy-path` |  |
| <span id="html-no-duplicate-dt"></span>[`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | Disallow duplicate &lt;dt&gt; names in &lt;dl&gt;. | `happy-path` |  |
| <span id="html-no-empty-palpable-content"></span>[`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | Disallow empty elements that expect visible content. | `happy-path` |  |
| <span id="html-require-datetime"></span>[`html/require-datetime`](./reference/html-require-datetime.md) | Require datetime attribute on &lt;time&gt; element. | `happy-path` |  |

## Accessibility rules (30)

| Rule | What it checks | Preset | Auto-fix |
| --- | --- | --- | --- |
| <span id="a11y-alt-text"></span>[`a11y/alt-text`](./reference/a11y-alt-text.md) | Require alternative text for media elements. | `happy-path` |  |
| <span id="a11y-anchor-has-content"></span>[`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | Require anchor elements to have accessible content. | `happy-path` |  |
| <span id="a11y-anchor-is-valid"></span>[`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | Enforce valid href on anchor elements. | `happy-path` |  |
| <span id="a11y-aria-props"></span>[`a11y/aria-props`](./reference/a11y-aria-props.md) | Disallow invalid ARIA attributes. | `happy-path` |  |
| <span id="a11y-aria-role"></span>[`a11y/aria-role`](./reference/a11y-aria-role.md) | Elements with ARIA roles must use a valid, non-abstract ARIA role. | `happy-path` |  |
| <span id="a11y-aria-unsupported-elements"></span>[`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | Disallow ARIA attributes on elements that do not support them. | `happy-path` |  |
| <span id="a11y-click-events-have-key-events"></span>[`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | Require keyboard event handlers with click events. | `happy-path` |  |
| <span id="a11y-form-control-has-label"></span>[`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | Require form controls to have associated labels. | `happy-path` |  |
| <span id="a11y-heading-has-content"></span>[`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | Require heading elements to have accessible content. | `happy-path` |  |
| <span id="a11y-heading-levels"></span>[`a11y/heading-levels`](./reference/a11y-heading-levels.md) | Disallow skipping heading levels. | `nuxt`, `opinionated` |  |
| <span id="a11y-iframe-has-title"></span>[`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | Require iframe elements to have a title attribute. | `happy-path` |  |
| <span id="a11y-img-alt"></span>[`a11y/img-alt`](./reference/a11y-img-alt.md) | Require alt attribute on images for accessibility. | — |  |
| <span id="a11y-interactive-supports-focus"></span>[`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | Require interactive role elements to be focusable. | `happy-path` |  |
| <span id="a11y-label-has-for"></span>[`a11y/label-has-for`](./reference/a11y-label-has-for.md) | Require labels to have associated form controls. | `happy-path` |  |
| <span id="a11y-landmark-roles"></span>[`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | Validate landmark role placement and uniqueness. | `nuxt`, `opinionated` |  |
| <span id="a11y-media-has-caption"></span>[`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | Require media elements to have captions. | `happy-path` |  |
| <span id="a11y-mouse-events-have-key-events"></span>[`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | Require focus/blur events with mouse events. | `happy-path` |  |
| <span id="a11y-no-access-key"></span>[`a11y/no-access-key`](./reference/a11y-no-access-key.md) | Disallow the use of the accesskey attribute. | `happy-path` |  |
| <span id="a11y-no-aria-hidden-on-focusable"></span>[`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | Disallow aria-hidden="true" on focusable elements. | `happy-path` |  |
| <span id="a11y-no-autofocus"></span>[`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | Disallow the use of the autofocus attribute. | `happy-path` |  |
| <span id="a11y-no-distracting-elements"></span>[`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt;. | `happy-path` |  |
| <span id="a11y-no-i-for-icon"></span>[`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | Disallow using &lt;i&gt; element for icons. | `happy-path` |  |
| <span id="a11y-no-redundant-roles"></span>[`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | Disallow redundant ARIA roles. | `happy-path` | ✓ |
| <span id="a11y-no-refer-to-non-existent-id"></span>[`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | Disallow references to non-existent IDs. | `happy-path` |  |
| <span id="a11y-no-role-presentation-on-focusable"></span>[`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | Disallow role="presentation" or role="none" on focusable elements. | `happy-path` |  |
| <span id="a11y-no-static-element-interactions"></span>[`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | Disallow event handlers on static elements. | `happy-path` |  |
| <span id="a11y-placeholder-label-option"></span>[`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | Require a placeholder label option in a required select. | `nuxt`, `opinionated` |  |
| <span id="a11y-role-has-required-aria-props"></span>[`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | Require ARIA roles to have required properties. | `happy-path` |  |
| <span id="a11y-tabindex-no-positive"></span>[`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | Disallow positive tabindex values. | `happy-path` |  |
| <span id="a11y-use-list"></span>[`a11y/use-list`](./reference/a11y-use-list.md) | Suggest using list elements for bullet-like text. | `nuxt`, `opinionated` |  |

## SSR rules (2)

| Rule | What it checks | Preset | Auto-fix |
| --- | --- | --- | --- |
| <span id="ssr-no-browser-globals-in-ssr"></span>[`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | Disallow browser-only globals in SSR context. | `happy-path` |  |
| <span id="ssr-no-hydration-mismatch"></span>[`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | Disallow non-deterministic values that cause hydration mismatch. | `happy-path` |  |

## Vapor rules (4)

| Rule | What it checks | Preset | Auto-fix |
| --- | --- | --- | --- |
| <span id="vapor-no-inline-template"></span>[`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | Disallow deprecated inline-template attribute. | `nuxt`, `opinionated` |  |
| <span id="vapor-no-vue-lifecycle-events"></span>[`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | Disallow @vue:xxx per-element lifecycle events (not supported in Vapor). | `happy-path` |  |
| <span id="vapor-prefer-static-class"></span>[`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | Prefer static class over dynamic class binding for string literals. | `nuxt`, `opinionated` | ✓ |
| <span id="vapor-require-vapor-attribute"></span>[`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | Suggest adding vapor attribute to script setup. | `nuxt`, `opinionated` |  |

## Ecosystem and Nuxt rules (13)

| Rule | What it checks | Preset | Auto-fix |
| --- | --- | --- | --- |
| <span id="ecosystem-nuxt-prefer-nuxt-link"></span>[`ecosystem/nuxt-prefer-nuxt-link`](./reference/ecosystem-nuxt-prefer-nuxt-link.md) | Prefer NuxtLink for internal application links. | `nuxt` |  |
| <span id="ecosystem-pinia-prefer-store-to-refs"></span>[`ecosystem/pinia-prefer-store-to-refs`](./reference/ecosystem-pinia-prefer-store-to-refs.md) | Prefer storeToRefs() when destructuring Pinia stores. | `ecosystem` |  |
| <span id="ecosystem-router-link-require-to"></span>[`ecosystem/router-link-require-to`](./reference/ecosystem-router-link-require-to.md) | Require a `to` target on RouterLink and NuxtLink components. | `ecosystem` |  |
| <span id="ecosystem-void-link-require-href"></span>[`ecosystem/void-link-require-href`](./reference/ecosystem-void-link-require-href.md) | Require `href` on Void Vue Link components. | `ecosystem` |  |
| <span id="ecosystem-void-link-valid-method"></span>[`ecosystem/void-link-valid-method`](./reference/ecosystem-void-link-valid-method.md) | Validate static Void Vue Link method props. | `ecosystem` |  |
| <span id="ecosystem-vue-i18n-no-missing-key"></span>[`ecosystem/vue-i18n-no-missing-key`](./reference/ecosystem-vue-i18n-no-missing-key.md) | Report static vue-i18n keys that are absent from local SFC messages. | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-link"></span>[`ecosystem/vue-router-prefer-named-link`](./reference/ecosystem-vue-router-prefer-named-link.md) | Prefer named route objects over static path strings in RouterLink. | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-push"></span>[`ecosystem/vue-router-prefer-named-push`](./reference/ecosystem-vue-router-prefer-named-push.md) | Prefer named route objects for Vue Router programmatic navigation. | `ecosystem` |  |
| <span id="ecosystem-vue-test-utils-no-html-snapshot"></span>[`ecosystem/vue-test-utils-no-html-snapshot`](./reference/ecosystem-vue-test-utils-no-html-snapshot.md) | Avoid snapshotting wrapper.html() in Vue Test Utils tests. | `ecosystem` |  |
| <span id="nuxt-no-nuxt-config-test-key"></span>[`nuxt/no-nuxt-config-test-key`](./reference/nuxt-no-nuxt-config-test-key.md) | Disallow setting `test` key in Nuxt config. | `nuxt` |  |
| <span id="nuxt-no-page-meta-runtime-values"></span>[`nuxt/no-page-meta-runtime-values`](./reference/nuxt-no-page-meta-runtime-values.md) | Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup. | `nuxt` |  |
| <span id="nuxt-nuxt-config-keys-order"></span>[`nuxt/nuxt-config-keys-order`](./reference/nuxt-nuxt-config-keys-order.md) | Prefer recommended order of Nuxt config properties. | `nuxt` | ✓ |
| <span id="nuxt-prefer-import-meta"></span>[`nuxt/prefer-import-meta`](./reference/nuxt-prefer-import-meta.md) | Prefer using `import.meta.*` over `process.*`. | `nuxt` | ✓ |

## Musea and CSS rules (16)

| Rule | What it checks | Preset | Auto-fix |
| --- | --- | --- | --- |
| <span id="css-no-display-none"></span>[`css/no-display-none`](./reference/css-no-display-none.md) | Suggest using v-show instead of display: none. | `opinionated`, `nuxt` |  |
| <span id="css-no-hardcoded-values"></span>[`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | Suggest using CSS variables instead of hardcoded values. | `opinionated`, `nuxt` |  |
| <span id="css-no-id-selectors"></span>[`css/no-id-selectors`](./reference/css-no-id-selectors.md) | Discourage use of ID selectors in CSS. | `opinionated`, `nuxt` |  |
| <span id="css-no-important"></span>[`css/no-important`](./reference/css-no-important.md) | Discourage use of !important in CSS. | `opinionated`, `nuxt` |  |
| <span id="css-no-utility-classes"></span>[`css/no-utility-classes`](./reference/css-no-utility-classes.md) | Warn against implementing utility classes in component styles. | `opinionated`, `nuxt` |  |
| <span id="css-no-v-bind-performance"></span>[`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | Warn about performance cost of CSS v-bind(). | `opinionated`, `nuxt` |  |
| <span id="css-prefer-logical-properties"></span>[`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | Recommend CSS logical properties for better i18n support. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-nested-selectors"></span>[`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | Recommend using CSS nesting for descendant selectors. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-slotted"></span>[`css/prefer-slotted`](./reference/css-prefer-slotted.md) | Recommend ::v-slotted() for styling slot content. | `opinionated`, `nuxt` |  |
| <span id="css-require-font-display"></span>[`css/require-font-display`](./reference/css-require-font-display.md) | Require font-display in @font-face rules. | `opinionated`, `nuxt` |  |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | Disallow empty &lt;variant&gt; blocks. | — |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | Prefer design token CSS variables over hardcoded primitive values. | — |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | Require component attribute in &lt;art&gt; block. | — |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | Require title attribute in &lt;art&gt; block. | — |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | Require unique variant names. | — |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | Require name attribute in &lt;variant&gt; blocks. | — |  |

## petite-vue rules (3)

| Rule | What it checks | Preset | Auto-fix |
| --- | --- | --- | --- |
| <span id="petite-vue-no-unsupported-directive"></span>[`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | Disallow directives that petite-vue does not support. | — |  |
| <span id="petite-vue-valid-v-effect"></span>[`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | Require v-effect to have a non-empty expression. | — |  |
| <span id="petite-vue-valid-v-scope"></span>[`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | Require v-scope to bind an object literal. | — |  |

## Cross-file rules (66)

Turn these checks on with `crossFile: true` in `vue.linter`, or run `vize lint --cross-file`. Diagnostics are shown as `vize:croquis/cf/<name>`; configure them in `vue.linter.rules` as `croquis/cf/<name>`, without `vize:`.

| Rule | What it checks | Status |
| --- | --- | --- |
| <span id="ecosystem-vue-router-extra-param"></span>[`ecosystem/vue-router-extra-param`](./project/ecosystem-vue-router-extra-param.md) | Reports navigation params that the target route does not declare. | Reported by `vp run lint` |
| <span id="ecosystem-vue-router-missing-param"></span>[`ecosystem/vue-router-missing-param`](./project/ecosystem-vue-router-missing-param.md) | Reports a named navigation that omits a required route param. | Reported by `vp run lint` |
| <span id="ecosystem-vue-router-param-type"></span>[`ecosystem/vue-router-param-type`](./project/ecosystem-vue-router-param-type.md) | Reports a route param value the declared segment does not accept. | Reported by `vp run lint` |
| <span id="ecosystem-vue-router-unknown-route"></span>[`ecosystem/vue-router-unknown-route`](./project/ecosystem-vue-router-unknown-route.md) | Reports a navigation name that the installed router does not declare. | Reported by `vp run lint` |
| <span id="html-cross-component-nesting"></span>[`html/cross-component-nesting`](./project/html-cross-component-nesting.md) | Check actual HTML nesting after imported components are composed. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-array-mutation"></span>[`vize:croquis/cf/array-mutation`](./project/vize-croquis-cf-array-mutation.md) | Use observable mutations for array index updates in Vue 2.7. | Planned; not reported yet |
| <span id="vize-croquis-cf-async-boundary"></span>[`vize:croquis/cf/async-boundary`](./project/vize-croquis-cf-async-boundary.md) | Reactive state crosses an async boundary and can be observed stale. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-async-no-suspense"></span>[`vize:croquis/cf/async-no-suspense`](./project/vize-croquis-cf-async-no-suspense.md) | An async component is rendered without a Suspense boundary. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-browser-api-ssr"></span>[`vize:croquis/cf/browser-api-ssr`](./project/vize-croquis-cf-browser-api-ssr.md) | A browser-only API is used where the component can render on the server. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-circular-dep"></span>[`vize:croquis/cf/circular-dep`](./project/vize-croquis-cf-circular-dep.md) | Components import each other in a cycle. | Planned; not reported yet |
| <span id="vize-croquis-cf-circular-reactive-dependency"></span>[`vize:croquis/cf/circular-reactive-dependency`](./project/vize-croquis-cf-circular-reactive-dependency.md) | Reactive computations depend on each other in a cycle. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-closure-captures-reactive"></span>[`vize:croquis/cf/closure-captures-reactive`](./project/vize-croquis-cf-closure-captures-reactive.md) | A closure captures a reactive value and will not see later updates. | Planned; not reported yet |
| <span id="vize-croquis-cf-composable-outside-setup"></span>[`vize:croquis/cf/composable-outside-setup`](./project/vize-croquis-cf-composable-outside-setup.md) | A composable is called outside `setup`. | Planned; not reported yet |
| <span id="vize-croquis-cf-computed-side-effects"></span>[`vize:croquis/cf/computed-side-effects`](./project/vize-croquis-cf-computed-side-effects.md) | A computed getter writes state or performs another side effect. | Planned; not reported yet |
| <span id="vize-croquis-cf-deep-import"></span>[`vize:croquis/cf/deep-import`](./project/vize-croquis-cf-deep-import.md) | An import chain is deeper than the project allows. | Planned; not reported yet |
| <span id="vize-croquis-cf-destructuring-breaks-reactivity"></span>[`vize:croquis/cf/destructuring-breaks-reactivity`](./project/vize-croquis-cf-destructuring-breaks-reactivity.md) | Destructuring a reactive object copies the fields and drops tracking. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-di-outside-setup"></span>[`vize:croquis/cf/di-outside-setup`](./project/vize-croquis-cf-di-outside-setup.md) | `provide` or `inject` is called outside `setup`. | Planned; not reported yet |
| <span id="vize-croquis-cf-dom-access-without-next-tick"></span>[`vize:croquis/cf/dom-access-without-next-tick`](./project/vize-croquis-cf-dom-access-without-next-tick.md) | The DOM is read before Vue has flushed the update. | Planned; not reported yet |
| <span id="vize-croquis-cf-duplicate-id"></span>[`vize:croquis/cf/duplicate-id`](./project/vize-croquis-cf-duplicate-id.md) | The same element id is used in more than one component. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-event-listener-leak"></span>[`vize:croquis/cf/event-listener-leak`](./project/vize-croquis-cf-event-listener-leak.md) | An event listener is registered and never removed. | Planned; not reported yet |
| <span id="vize-croquis-cf-event-modifier"></span>[`vize:croquis/cf/event-modifier`](./project/vize-croquis-cf-event-modifier.md) | An event listener uses a modifier the emit does not support. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-hydration-risk"></span>[`vize:croquis/cf/hydration-risk`](./project/vize-croquis-cf-hydration-risk.md) | The render can disagree between the server and the client. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-inherit-attrs-unused"></span>[`vize:croquis/cf/inherit-attrs-unused`](./project/vize-croquis-cf-inherit-attrs-unused.md) | `inheritAttrs: false` is set and the component never reads the attributes. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-inject-without-symbol"></span>[`vize:croquis/cf/inject-without-symbol`](./project/vize-croquis-cf-inject-without-symbol.md) | `inject` uses a plain key instead of an `InjectionKey` symbol. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-injected-async-mutation-race"></span>[`vize:croquis/cf/injected-async-mutation-race`](./project/vize-croquis-cf-injected-async-mutation-race.md) | An injected value is mutated from an async task that can race. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-lifecycle-outside-setup"></span>[`vize:croquis/cf/lifecycle-outside-setup`](./project/vize-croquis-cf-lifecycle-outside-setup.md) | A lifecycle hook is registered outside `setup`. | Planned; not reported yet |
| <span id="vize-croquis-cf-lifecycle-without-cleanup"></span>[`vize:croquis/cf/lifecycle-without-cleanup`](./project/vize-croquis-cf-lifecycle-without-cleanup.md) | A lifecycle hook starts work and never cleans it up. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-missing-required-prop"></span>[`vize:croquis/cf/missing-required-prop`](./project/vize-croquis-cf-missing-required-prop.md) | A required prop is not passed. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-missing-suspense"></span>[`vize:croquis/cf/missing-suspense`](./project/vize-croquis-cf-missing-suspense.md) | An async dependency is used outside a Suspense boundary. | Planned; not reported yet |
| <span id="vize-croquis-cf-module-scope-reactive"></span>[`vize:croquis/cf/module-scope-reactive`](./project/vize-croquis-cf-module-scope-reactive.md) | Reactive state is created at module scope and shared by every caller. | Planned; not reported yet |
| <span id="vize-croquis-cf-multi-root-attrs"></span>[`vize:croquis/cf/multi-root-attrs`](./project/vize-croquis-cf-multi-root-attrs.md) | A multi-root component receives attributes and has nowhere to put them. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-mutated-after-escape"></span>[`vize:croquis/cf/mutated-after-escape`](./project/vize-croquis-cf-mutated-after-escape.md) | A reactive object is mutated after it has escaped its owner. | Planned; not reported yet |
| <span id="vize-croquis-cf-non-reactive-provide"></span>[`vize:croquis/cf/non-reactive-provide`](./project/vize-croquis-cf-non-reactive-provide.md) | A provided value is not reactive, so descendants will not see updates. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-non-unique-id"></span>[`vize:croquis/cf/non-unique-id`](./project/vize-croquis-cf-non-unique-id.md) | An element id inside a loop is not unique per item. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-object-identity-comparison"></span>[`vize:croquis/cf/object-identity-comparison`](./project/vize-croquis-cf-object-identity-comparison.md) | Compare stable record IDs across raw objects and reactive proxies. | Planned; not reported yet |
| <span id="vize-croquis-cf-pinia-getter"></span>[`vize:croquis/cf/pinia-getter`](./project/vize-croquis-cf-pinia-getter.md) | Avoid setup-time snapshots of Pinia getter values. | Planned; not reported yet |
| <span id="vize-croquis-cf-prop-type-mismatch"></span>[`vize:croquis/cf/prop-type-mismatch`](./project/vize-croquis-cf-prop-type-mismatch.md) | A passed prop value does not match the declared type. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-provide-inject-type"></span>[`vize:croquis/cf/provide-inject-type`](./project/vize-croquis-cf-provide-inject-type.md) | A provided value and its inject do not have the same type. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-provide-without-symbol"></span>[`vize:croquis/cf/provide-without-symbol`](./project/vize-croquis-cf-provide-without-symbol.md) | `provide` uses a plain key instead of an `InjectionKey` symbol. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-reactive-export"></span>[`vize:croquis/cf/reactive-export`](./project/vize-croquis-cf-reactive-export.md) | Reactive state is exported from the module. | Planned; not reported yet |
| <span id="vize-croquis-cf-reactivity-outside-setup"></span>[`vize:croquis/cf/reactivity-outside-setup`](./project/vize-croquis-cf-reactivity-outside-setup.md) | A reactive API is called outside `setup`. | Planned; not reported yet |
| <span id="vize-croquis-cf-reassignment-breaks-reactivity"></span>[`vize:croquis/cf/reassignment-breaks-reactivity`](./project/vize-croquis-cf-reassignment-breaks-reactivity.md) | Reassigning a reactive binding replaces it with a plain value. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-reference-escapes-scope"></span>[`vize:croquis/cf/reference-escapes-scope`](./project/vize-croquis-cf-reference-escapes-scope.md) | A reactive reference escapes the scope that owns its lifetime. | Planned; not reported yet |
| <span id="vize-croquis-cf-setup-context-violation"></span>[`vize:croquis/cf/setup-context-violation`](./project/vize-croquis-cf-setup-context-violation.md) | Keep component-owned reactive state inside per-instance setup. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-shallow-deep-access"></span>[`vize:croquis/cf/shallow-deep-access`](./project/vize-croquis-cf-shallow-deep-access.md) | A deep property of a `shallowReactive` or `shallowRef` value is read as if it were tracked. | Planned; not reported yet |
| <span id="vize-croquis-cf-spread-breaks-reactivity"></span>[`vize:croquis/cf/spread-breaks-reactivity`](./project/vize-croquis-cf-spread-breaks-reactivity.md) | Spreading a reactive object copies its values and drops the tracking. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-suspense-no-fallback"></span>[`vize:croquis/cf/suspense-no-fallback`](./project/vize-croquis-cf-suspense-no-fallback.md) | `<Suspense>` has no fallback content. | Planned; not reported yet |
| <span id="vize-croquis-cf-template-ref-timing"></span>[`vize:croquis/cf/template-ref-timing`](./project/vize-croquis-cf-template-ref-timing.md) | A template ref is read before the component is mounted. | Planned; not reported yet |
| <span id="vize-croquis-cf-toraw-mutation"></span>[`vize:croquis/cf/toraw-mutation`](./project/vize-croquis-cf-toraw-mutation.md) | `toRaw` is used and the raw object is then mutated. | Planned; not reported yet |
| <span id="vize-croquis-cf-uncaught-error"></span>[`vize:croquis/cf/uncaught-error`](./project/vize-croquis-cf-uncaught-error.md) | A component can throw and no error boundary catches it. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-undeclared-emit"></span>[`vize:croquis/cf/undeclared-emit`](./project/vize-croquis-cf-undeclared-emit.md) | The component emits an event that is not declared. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-undeclared-prop"></span>[`vize:croquis/cf/undeclared-prop`](./project/vize-croquis-cf-undeclared-prop.md) | A parent passes a prop the child does not declare. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-undefined-slot"></span>[`vize:croquis/cf/undefined-slot`](./project/vize-croquis-cf-undefined-slot.md) | A parent fills a slot the child does not expose. | Planned; not reported yet |
| <span id="vize-croquis-cf-unhandled-event"></span>[`vize:croquis/cf/unhandled-event`](./project/vize-croquis-cf-unhandled-event.md) | A child emits an event that no parent handles. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-unmatched-inject"></span>[`vize:croquis/cf/unmatched-inject`](./project/vize-croquis-cf-unmatched-inject.md) | `inject` names a key that no ancestor provides. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-unmatched-listener"></span>[`vize:croquis/cf/unmatched-listener`](./project/vize-croquis-cf-unmatched-listener.md) | A parent listens for an event the child does not emit. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-unregistered-component"></span>[`vize:croquis/cf/unregistered-component`](./project/vize-croquis-cf-unregistered-component.md) | A template uses a component that is not registered or imported. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-unresolved-import"></span>[`vize:croquis/cf/unresolved-import`](./project/vize-croquis-cf-unresolved-import.md) | An import does not resolve to a module. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-unused-attrs"></span>[`vize:croquis/cf/unused-attrs`](./project/vize-croquis-cf-unused-attrs.md) | Fallthrough attributes are passed to a multi-root component that does not use them. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-unused-emit"></span>[`vize:croquis/cf/unused-emit`](./project/vize-croquis-cf-unused-emit.md) | A declared emit is never used. | Experimental; not reported by the CLI yet |
| <span id="vize-croquis-cf-unused-provide"></span>[`vize:croquis/cf/unused-provide`](./project/vize-croquis-cf-unused-provide.md) | A provided key is never injected. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-value-extraction-breaks-reactivity"></span>[`vize:croquis/cf/value-extraction-breaks-reactivity`](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md) | Reading a reactive value out into a local drops later updates. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-watch-can-be-computed"></span>[`vize:croquis/cf/watch-can-be-computed`](./project/vize-croquis-cf-watch-can-be-computed.md) | A watcher only copies a value into state and can be a computed. | Planned; not reported yet |
| <span id="vize-croquis-cf-watcheffect-async"></span>[`vize:croquis/cf/watcheffect-async`](./project/vize-croquis-cf-watcheffect-async.md) | Guard async watchEffect work against invalidation and stale responses. | Reported by `vp run lint` |
| <span id="vize-croquis-cf-watcher-outside-setup"></span>[`vize:croquis/cf/watcher-outside-setup`](./project/vize-croquis-cf-watcher-outside-setup.md) | `watch` or `watchEffect` is called outside `setup`. | Planned; not reported yet |
| <span id="vue-cross-file-attrs-fallthrough"></span>[`vue/cross-file-attrs-fallthrough`](./project/vue-cross-file-attrs-fallthrough.md) | A parent passes attributes to a resolved child whose root cannot inherit them and does not explicitly use $attrs. | Reported by `vp run lint` |

[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)
