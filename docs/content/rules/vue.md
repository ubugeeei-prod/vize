---
title: "Vue rules"
description: "Correctness, conventions and Vue 3 migration checks for SFC templates, directives, components and blocks."
---

# Vue rules

Correctness, conventions and Vue 3 migration checks for SFC templates, directives, components and blocks.

Each rule page shows a failing example, a passing example, and the configuration that turns the rule on.

## Essential (53)

Correctness rules in the `essential` preset.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="vue-comment-directive"></span>[`vue/comment-directive`](./reference/vue-comment-directive.md) | Support comment directives in `<template>`. |  |
| <span id="vue-jsx-uses-vars"></span>[`vue/jsx-uses-vars`](./reference/vue-jsx-uses-vars.md) | Treat variables used in JSX element names as used. |  |
| <span id="vue-multi-word-component-names"></span>[`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | Require component names to be multi-word. |  |
| <span id="vue-no-child-content"></span>[`vue/no-child-content`](./reference/vue-no-child-content.md) | Disallow child content when using v-html or v-text. |  |
| <span id="vue-no-deprecated-filter"></span>[`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | Disallow deprecated Vue 2 filter syntax using the pipe operator. |  |
| <span id="vue-no-deprecated-functional-template"></span>[`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | Disallow the `functional` attribute on the SFC `<template>`. |  |
| <span id="vue-no-deprecated-html-element-is"></span>[`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | Disallow the `is` attribute on native HTML elements. |  |
| <span id="vue-no-deprecated-inline-template"></span>[`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | Disallow the deprecated `inline-template` attribute. |  |
| <span id="vue-no-deprecated-router-link-tag-prop"></span>[`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | Disallow the `tag` prop on &lt;router-link&gt;. |  |
| <span id="vue-no-deprecated-scope-attribute"></span>[`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | Disallow the deprecated `scope` attribute on &lt;template&gt;. |  |
| <span id="vue-no-deprecated-slot-attribute"></span>[`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | Disallow the deprecated `slot` attribute. |  |
| <span id="vue-no-deprecated-slot-scope-attribute"></span>[`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | Disallow the deprecated `slot-scope` attribute. |  |
| <span id="vue-no-deprecated-v-bind-sync"></span>[`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | Disallow the deprecated `.sync` modifier on `v-bind`. |  |
| <span id="vue-no-deprecated-v-is"></span>[`vue/no-deprecated-v-is`](./reference/vue-no-deprecated-v-is.md) | Disallow the deprecated `v-is` directive. |  |
| <span id="vue-no-deprecated-v-on-native-modifier"></span>[`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | Disallow the deprecated `.native` modifier on `v-on`. |  |
| <span id="vue-no-deprecated-v-on-number-modifiers"></span>[`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | Disallow deprecated numeric `keyCode` modifiers on `v-on`. | ✓ |
| <span id="vue-no-dupe-v-else-if"></span>[`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | Disallow duplicate conditions in `v-if` / `v-else-if` chains. |  |
| <span id="vue-no-duplicate-attributes"></span>[`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | Disallow duplicate attributes on the same element. |  |
| <span id="vue-no-mutating-props"></span>[`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | Disallow mutating component props. |  |
| <span id="vue-no-non-component-keep-alive-child"></span>[`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | Disallow plain element wrappers directly below `<KeepAlive>`. |  |
| <span id="vue-no-reserved-component-names"></span>[`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | Disallow the use of reserved names as component names. |  |
| <span id="vue-no-template-key"></span>[`vue/no-template-key`](./reference/vue-no-template-key.md) | Disallow `key` attribute on `<template>`. |  |
| <span id="vue-no-textarea-mustache"></span>[`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | Disallow mustache interpolation in `<textarea>`. |  |
| <span id="vue-no-unsafe-url"></span>[`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | Warn about potentially unsafe URL bindings. |  |
| <span id="vue-no-unused-components"></span>[`vue/no-unused-components`](./reference/vue-no-unused-components.md) | Disallow registering components that are not used inside templates. |  |
| <span id="vue-no-unused-vars"></span>[`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | Disallow unused variable definitions in v-for and v-slot directives. |  |
| <span id="vue-no-use-v-if-with-v-for"></span>[`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | Disallow using `v-if` on the same element as `v-for`. |  |
| <span id="vue-no-useless-template-attributes"></span>[`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | Disallow useless attributes on `<template>` elements. |  |
| <span id="vue-no-v-for-template-key-on-child"></span>[`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | Disallow `key` on the child of a `<template v-for>`. |  |
| <span id="vue-no-v-html"></span>[`vue/no-v-html`](./reference/vue-no-v-html.md) | Warn against v-html to prevent XSS vulnerabilities. |  |
| <span id="vue-no-v-text-v-html-on-component"></span>[`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | Disallow v-text / v-html on component elements. |  |
| <span id="vue-require-component-is"></span>[`vue/require-component-is`](./reference/vue-require-component-is.md) | Require `v-bind:is` on `<component>` elements. |  |
| <span id="vue-require-toggle-inside-transition"></span>[`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | Require a toggle on the element wrapped by `<transition>`. |  |
| <span id="vue-require-v-for-key"></span>[`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | Require `v-bind:key` with `v-for` directives. |  |
| <span id="vue-use-v-on-exact"></span>[`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | Enforce `.exact` modifier on `v-on` when there are modifier-based handlers. |  |
| <span id="vue-valid-attribute-name"></span>[`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | Require valid attribute names. |  |
| <span id="vue-valid-template-root"></span>[`vue/valid-template-root`](./reference/vue-valid-template-root.md) | Enforce valid root content in an SFC `<template>` block. |  |
| <span id="vue-valid-v-bind"></span>[`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | Enforce valid `v-bind` directives. |  |
| <span id="vue-valid-v-cloak"></span>[`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | Enforce valid `v-cloak` directives. |  |
| <span id="vue-valid-v-else"></span>[`vue/valid-v-else`](./reference/vue-valid-v-else.md) | Enforce valid `v-else` directives. | ✓ |
| <span id="vue-valid-v-else-if"></span>[`vue/valid-v-else-if`](./reference/vue-valid-v-else-if.md) | Enforce valid `v-else-if` directives. |  |
| <span id="vue-valid-v-for"></span>[`vue/valid-v-for`](./reference/vue-valid-v-for.md) | Enforce valid `v-for` directives. |  |
| <span id="vue-valid-v-html"></span>[`vue/valid-v-html`](./reference/vue-valid-v-html.md) | Enforce valid `v-html` directives. |  |
| <span id="vue-valid-v-if"></span>[`vue/valid-v-if`](./reference/vue-valid-v-if.md) | Enforce valid `v-if` directives. |  |
| <span id="vue-valid-v-is"></span>[`vue/valid-v-is`](./reference/vue-valid-v-is.md) | Enforce valid `v-is` directives. |  |
| <span id="vue-valid-v-memo"></span>[`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | Enforce valid `v-memo` directives. |  |
| <span id="vue-valid-v-model"></span>[`vue/valid-v-model`](./reference/vue-valid-v-model.md) | Enforce valid `v-model` directives. |  |
| <span id="vue-valid-v-on"></span>[`vue/valid-v-on`](./reference/vue-valid-v-on.md) | Enforce valid `v-on` directives. |  |
| <span id="vue-valid-v-once"></span>[`vue/valid-v-once`](./reference/vue-valid-v-once.md) | Enforce valid `v-once` directives. |  |
| <span id="vue-valid-v-pre"></span>[`vue/valid-v-pre`](./reference/vue-valid-v-pre.md) | Enforce valid `v-pre` directives. |  |
| <span id="vue-valid-v-show"></span>[`vue/valid-v-show`](./reference/vue-valid-v-show.md) | Enforce valid `v-show` directives. |  |
| <span id="vue-valid-v-slot"></span>[`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | Enforce valid `v-slot` directives. |  |
| <span id="vue-valid-v-text"></span>[`vue/valid-v-text`](./reference/vue-valid-v-text.md) | Enforce valid `v-text` directives. |  |

## Default (27)

Added by `happy-path`, the default preset.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="vue-attribute-hyphenation"></span>[`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | Enforce attribute naming style on custom components. | ✓ |
| <span id="vue-attribute-order"></span>[`vue/attribute-order`](./reference/vue-attribute-order.md) | Enforce a consistent order of attributes. |  |
| <span id="vue-component-definition-name-casing"></span>[`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | Enforce PascalCase or kebab-case for component definition names. |  |
| <span id="vue-html-end-tags"></span>[`vue/html-end-tags`](./reference/vue-html-end-tags.md) | Require end tags on HTML elements that may omit them. | ✓ |
| <span id="vue-html-quotes"></span>[`vue/html-quotes`](./reference/vue-html-quotes.md) | Enforce quotes style of HTML attributes. | ✓ |
| <span id="vue-mustache-interpolation-spacing"></span>[`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | Enforce consistent spacing inside mustache interpolations. | ✓ |
| <span id="vue-no-invalid-html-attribute"></span>[`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | Disallow invalid static values for HTML attributes. |  |
| <span id="vue-no-lone-template"></span>[`vue/no-lone-template`](./reference/vue-no-lone-template.md) | Disallow unnecessary `<template>` elements. |  |
| <span id="vue-no-multi-spaces"></span>[`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | Disallow multiple consecutive spaces. | ✓ |
| <span id="vue-no-template-shadow"></span>[`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | Disallow variable names that shadow variables in outer scope. |  |
| <span id="vue-no-template-target-blank"></span>[`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | Disallow target="_blank" without rel="noopener noreferrer". |  |
| <span id="vue-no-this-in-before-route-enter"></span>[`vue/no-this-in-before-route-enter`](./reference/vue-no-this-in-before-route-enter.md) | Disallow `this` in `beforeRouteEnter`. |  |
| <span id="vue-no-undefined-refs"></span>[`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | Disallow undefined variable references in templates. |  |
| <span id="vue-no-unsandboxed-iframe"></span>[`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | Require a sandbox attribute on iframe elements. |  |
| <span id="vue-no-unused-properties"></span>[`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | Disallow unused properties defined in defineProps. |  |
| <span id="vue-no-unused-setup-bindings"></span>[`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | Disallow unread script setup bindings. |  |
| <span id="vue-no-use-v-else-with-v-for"></span>[`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | Disallow using `v-else-if` or `v-else` on the same element as `v-for`. |  |
| <span id="vue-permitted-contents"></span>[`vue/permitted-contents`](./reference/vue-permitted-contents.md) | Enforce HTML content model rules. |  |
| <span id="vue-prop-name-casing"></span>[`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | Enforce a casing for declared prop names. |  |
| <span id="vue-require-scoped-style"></span>[`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | Require scoped attribute on style tags. |  |
| <span id="vue-sfc-element-order"></span>[`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | Enforce consistent order of SFC top-level elements. |  |
| <span id="vue-single-style-block"></span>[`vue/single-style-block`](./reference/vue-single-style-block.md) | Recommend having a single style block. |  |
| <span id="vue-this-in-template"></span>[`vue/this-in-template`](./reference/vue-this-in-template.md) | Disallow `this.` in template expressions. |  |
| <span id="vue-v-bind-style"></span>[`vue/v-bind-style`](./reference/vue-v-bind-style.md) | Enforce `v-bind` directive style. | ✓ |
| <span id="vue-v-on-event-hyphenation"></span>[`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | Enforce hyphenation of custom event names in v-on on components. | ✓ |
| <span id="vue-v-on-style"></span>[`vue/v-on-style`](./reference/vue-v-on-style.md) | Enforce `v-on` directive style. | ✓ |
| <span id="vue-v-slot-style"></span>[`vue/v-slot-style`](./reference/vue-v-slot-style.md) | Enforce `v-slot` directive style. | ✓ |

## Stricter presets (27)

Only in `opinionated`, `nuxt` or `ecosystem`. The Presets column lists which.

| Rule | What it checks | Presets | Auto-fix |
| --- | --- | --- | --- |
| <span id="vue-component-name-in-template-casing"></span>[`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | Enforce specific casing for component names in templates. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-html-button-has-type"></span>[`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | Require an explicit valid type on button elements. | `nuxt`, `opinionated` |  |
| <span id="vue-html-self-closing"></span>[`vue/html-self-closing`](./reference/vue-html-self-closing.md) | Enforce self-closing style. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-array-index-key"></span>[`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | Disallow using the v-for index variable directly as the :key. | `nuxt`, `opinionated` |  |
| <span id="vue-no-boolean-attr-value"></span>[`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | Disallow explicit values for boolean HTML attributes. | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-empty-component-block"></span>[`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | Disallow empty SFC blocks. | `nuxt`, `opinionated` |  |
| <span id="vue-no-inline-style"></span>[`vue/no-inline-style`](./reference/vue-no-inline-style.md) | Discourage use of inline style attributes. | `nuxt`, `opinionated` |  |
| <span id="vue-no-multiple-objects-in-class"></span>[`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | Disallow multiple object literals inside a :class array binding. | `nuxt`, `opinionated` |  |
| <span id="vue-no-negated-v-if-condition"></span>[`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | Disallow a negated v-if condition when the chain has a v-else. | `nuxt`, `opinionated` |  |
| <span id="vue-no-preprocessor-lang"></span>[`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | Discourage CSS preprocessor usage in favor of modern CSS. | `nuxt`, `opinionated` |  |
| <span id="vue-no-root-v-if"></span>[`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | Disallow v-if on the single root element of a template. | `nuxt`, `opinionated` |  |
| <span id="vue-no-script-non-standard-lang"></span>[`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | Discourage non-standard script lang values. | `nuxt`, `opinionated` |  |
| <span id="vue-no-src-attribute"></span>[`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | Discourage src attribute on SFC blocks. | `nuxt`, `opinionated` |  |
| <span id="vue-no-template-lang"></span>[`vue/no-template-lang`](./reference/vue-no-template-lang.md) | Discourage lang attribute on template block. | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-refs"></span>[`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | Report template refs (ref="x") never referenced in &lt;script&gt;. | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-mustaches"></span>[`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | Disallow a mustache interpolation whose expression is a constant string literal. | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-v-bind"></span>[`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | Disallow a v-bind whose value is a plain string literal. | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-text"></span>[`vue/no-v-text`](./reference/vue-no-v-text.md) | Disallow the v-text directive; prefer mustache interpolation. | `nuxt`, `opinionated` |  |
| <span id="vue-prefer-props-shorthand"></span>[`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | Recommend shorthand syntax for props (Vue 3.4+). | `nuxt`, `opinionated` | ✓ |
| <span id="vue-prefer-true-attribute-shorthand"></span>[`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | Prefer the shorthand for a boolean attribute bound to `true`. | `nuxt`, `opinionated` |  |
| <span id="vue-require-component-registration"></span>[`vue/require-component-registration`](./reference/vue-require-component-registration.md) | Require explicit import or registration for components. | `opinionated` |  |
| <span id="vue-scoped-event-names"></span>[`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | Recommend scoped event names using context:event format. | `nuxt`, `opinionated` |  |
| <span id="vue-slot-name-casing"></span>[`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | Enforce configured casing for rendered slot names. | `nuxt`, `opinionated` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | Enforce unique element IDs using useId() instead of static literals. | `nuxt`, `opinionated` |  |
| <span id="vue-v-on-handler-style"></span>[`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | Enforce writing v-on handlers as method references. | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-block"></span>[`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | Warn about custom blocks in SFC files. | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-directive"></span>[`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | Warn about custom directives that need registration. | `nuxt`, `opinionated` |  |

## Opt-in (41)

Not in any preset. Turn them on by name in the `linter.rules` passed to `withVue`.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="vue-a11y-img-alt"></span>[`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | Require alt attribute on images for accessibility. |  |
| <span id="vue-camelcase"></span>[`vue/camelcase`](./reference/vue-camelcase.md) | Require camelCase identifiers in template expressions. |  |
| <span id="vue-dot-notation"></span>[`vue/dot-notation`](./reference/vue-dot-notation.md) | Prefer dot notation in template expressions. |  |
| <span id="vue-enforce-style-attribute"></span>[`vue/enforce-style-attribute`](./reference/vue-enforce-style-attribute.md) | Enforce allowed style-block attributes. |  |
| <span id="vue-eqeqeq"></span>[`vue/eqeqeq`](./reference/vue-eqeqeq.md) | Require strict equality in template expressions. |  |
| <span id="vue-match-component-file-name"></span>[`vue/match-component-file-name`](./reference/vue-match-component-file-name.md) | Require a component name to match its file name. |  |
| <span id="vue-max-template-complexity"></span>[`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | Limit a component's own template complexity (cyclomatic and cognitive). |  |
| <span id="vue-next-tick-style"></span>[`vue/next-tick-style`](./reference/vue-next-tick-style.md) | Enforce Promise-style nextTick usage. |  |
| <span id="vue-no-bare-strings-in-template"></span>[`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | Disallow raw human-readable text in the template that should be internationalized. |  |
| <span id="vue-no-console"></span>[`vue/no-console`](./reference/vue-no-console.md) | Disallow console access in template expressions. |  |
| <span id="vue-no-constant-condition"></span>[`vue/no-constant-condition`](./reference/vue-no-constant-condition.md) | Disallow constant conditions in template expressions. |  |
| <span id="vue-no-content-outside-block"></span>[`vue/no-content-outside-block`](./reference/vue-no-content-outside-block.md) | Disallow content outside SFC top-level blocks. |  |
| <span id="vue-no-custom-modifiers-on-v-model"></span>[`vue/no-custom-modifiers-on-v-model`](./reference/vue-no-custom-modifiers-on-v-model.md) | Disallow custom modifiers on `v-model` used on a component. |  |
| <span id="vue-no-define-model-default"></span>[`vue/no-define-model-default`](./reference/vue-no-define-model-default.md) | Disallow default values in defineModel. |  |
| <span id="vue-no-empty-pattern"></span>[`vue/no-empty-pattern`](./reference/vue-no-empty-pattern.md) | Disallow empty destructuring patterns in template expressions. |  |
| <span id="vue-no-implicit-coercion"></span>[`vue/no-implicit-coercion`](./reference/vue-no-implicit-coercion.md) | Disallow implicit coercion in template expressions. |  |
| <span id="vue-no-irregular-whitespace"></span>[`vue/no-irregular-whitespace`](./reference/vue-no-irregular-whitespace.md) | Disallow irregular whitespace in script blocks and templates. |  |
| <span id="vue-no-literals-in-template"></span>[`vue/no-literals-in-template`](./reference/vue-no-literals-in-template.md) | Disallow object, array and function literals in v-bind. |  |
| <span id="vue-no-loss-of-precision"></span>[`vue/no-loss-of-precision`](./reference/vue-no-loss-of-precision.md) | Disallow numeric literals that lose precision. |  |
| <span id="vue-no-multiple-template-root"></span>[`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | Disallow multiple root nodes in a template. |  |
| <span id="vue-no-negated-condition"></span>[`vue/no-negated-condition`](./reference/vue-no-negated-condition.md) | Disallow negated ternary conditions in template expressions. |  |
| <span id="vue-no-props-shadow"></span>[`vue/no-props-shadow`](./reference/vue-no-props-shadow.md) | Disallow setup declarations that shadow props. |  |
| <span id="vue-no-ref-object-reactivity-loss"></span>[`vue/no-ref-object-reactivity-loss`](./reference/vue-no-ref-object-reactivity-loss.md) | Disallow copying a ref value in the same scope. |  |
| <span id="vue-no-restricted-syntax"></span>[`vue/no-restricted-syntax`](./reference/vue-no-restricted-syntax.md) | Disallow configured syntax in the template. |  |
| <span id="vue-no-restricted-v-bind"></span>[`vue/no-restricted-v-bind`](./reference/vue-no-restricted-v-bind.md) | Disallow restricted v-bind arguments. |  |
| <span id="vue-no-setup-props-reactivity-loss"></span>[`vue/no-setup-props-reactivity-loss`](./reference/vue-no-setup-props-reactivity-loss.md) | Disallow setup prop reads that lose reactivity. |  |
| <span id="vue-no-shadow-native-events"></span>[`vue/no-shadow-native-events`](./reference/vue-no-shadow-native-events.md) | Disallow component events that shadow native events. |  |
| <span id="vue-no-sparse-arrays"></span>[`vue/no-sparse-arrays`](./reference/vue-no-sparse-arrays.md) | Disallow sparse arrays in template expressions. |  |
| <span id="vue-no-static-inline-styles"></span>[`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | Disallow static inline style attributes. |  |
| <span id="vue-no-useless-concat"></span>[`vue/no-useless-concat`](./reference/vue-no-useless-concat.md) | Disallow useless string concatenation in template expressions. |  |
| <span id="vue-no-v-for-template-key"></span>[`vue/no-v-for-template-key`](./reference/vue-no-v-for-template-key.md) | Disallow `key` on `<template v-for>`. |  |
| <span id="vue-object-shorthand"></span>[`vue/object-shorthand`](./reference/vue-object-shorthand.md) | Require object shorthand in template expressions. |  |
| <span id="vue-prefer-prop-type-boolean-first"></span>[`vue/prefer-prop-type-boolean-first`](./reference/vue-prefer-prop-type-boolean-first.md) | Enforce `Boolean` comes first in component prop types. |  |
| <span id="vue-prefer-template"></span>[`vue/prefer-template`](./reference/vue-prefer-template.md) | Prefer template literals in template expressions. |  |
| <span id="vue-prefer-v-model"></span>[`vue/prefer-v-model`](./reference/vue-prefer-v-model.md) | Prefer v-model over matching bind and update listeners. |  |
| <span id="vue-quote-props"></span>[`vue/quote-props`](./reference/vue-quote-props.md) | Disallow unnecessary quoted object properties in template expressions. |  |
| <span id="vue-require-expose"></span>[`vue/require-expose`](./reference/vue-require-expose.md) | Require declare public properties using `expose`. |  |
| <span id="vue-require-macro-variable-name"></span>[`vue/require-macro-variable-name`](./reference/vue-require-macro-variable-name.md) | Require conventional macro variable names. |  |
| <span id="vue-sort-keys"></span>[`vue/sort-keys`](./reference/vue-sort-keys.md) | Require sorted object keys, compatible with component option order. |  |
| <span id="vue-v-if-else-key"></span>[`vue/v-if-else-key`](./reference/vue-v-if-else-key.md) | Require keys on repeated conditional components. |  |
| <span id="vue-valid-model-definition"></span>[`vue/valid-model-definition`](./reference/vue-valid-model-definition.md) | Require valid keys in the model option. |  |

[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)
