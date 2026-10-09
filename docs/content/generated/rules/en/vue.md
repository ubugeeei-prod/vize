---
title: Vue rules
---

# Vue rules

Every Vue rule has its purpose, configuration, Bad and Good examples on this page. Highlighted lines show the change; copied code keeps the complete source.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [Bad](#vue-a11y-img-alt-bad) · [Good](#vue-a11y-img-alt-good) | Require alt attribute on images for accessibility |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [Bad](#vue-attribute-hyphenation-bad) · [Good](#vue-attribute-hyphenation-good) | Enforce attribute naming style on custom components |
| [`vue/attribute-order`](#vue-attribute-order) | [Bad](#vue-attribute-order-bad) · [Good](#vue-attribute-order-good) | Enforce a consistent order of attributes |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [Bad](#vue-component-definition-name-casing-bad) · [Good](#vue-component-definition-name-casing-good) | Enforce PascalCase or kebab-case for component definition names |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [Bad](#vue-component-name-in-template-casing-bad) · [Good](#vue-component-name-in-template-casing-good) | Enforce specific casing for component names in templates |
| [`vue/html-button-has-type`](#vue-html-button-has-type) | [Bad](#vue-html-button-has-type-bad) · [Good](#vue-html-button-has-type-good) | Require an explicit valid type on button elements |
| [`vue/html-quotes`](#vue-html-quotes) | [Bad](#vue-html-quotes-bad) · [Good](#vue-html-quotes-good) | Enforce quotes style of HTML attributes |
| [`vue/html-self-closing`](#vue-html-self-closing) | [Bad](#vue-html-self-closing-bad) · [Good](#vue-html-self-closing-good) | Enforce self-closing style |
| [`vue/max-template-complexity`](#vue-max-template-complexity) | [Bad](#vue-max-template-complexity-bad) · [Good](#vue-max-template-complexity-good) | Limit a component's own template complexity (cyclomatic and cognitive) |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [Bad](#vue-multi-word-component-names-bad) · [Good](#vue-multi-word-component-names-good) | Require component names to be multi-word |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [Bad](#vue-mustache-interpolation-spacing-bad) · [Good](#vue-mustache-interpolation-spacing-good) | Enforce consistent spacing inside mustache interpolations |
| [`vue/no-array-index-key`](#vue-no-array-index-key) | [Bad](#vue-no-array-index-key-bad) · [Good](#vue-no-array-index-key-good) | Disallow using the v-for index variable directly as the :key |
| [`vue/no-bare-strings-in-template`](#vue-no-bare-strings-in-template) | [Bad](#vue-no-bare-strings-in-template-bad) · [Good](#vue-no-bare-strings-in-template-good) | Disallow raw human-readable text in the template that should be internationalized |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [Bad](#vue-no-boolean-attr-value-bad) · [Good](#vue-no-boolean-attr-value-good) | Disallow explicit values for boolean HTML attributes |
| [`vue/no-child-content`](#vue-no-child-content) | [Bad](#vue-no-child-content-bad) · [Good](#vue-no-child-content-good) | Disallow child content when using v-html or v-text |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [Bad](#vue-no-deprecated-filter-bad) · [Good](#vue-no-deprecated-filter-good) | Disallow deprecated Vue 2 filter syntax using the pipe operator |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [Bad](#vue-no-deprecated-functional-template-bad) · [Good](#vue-no-deprecated-functional-template-good) | Disallow the `functional` attribute on the SFC `<template>` |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [Bad](#vue-no-deprecated-html-element-is-bad) · [Good](#vue-no-deprecated-html-element-is-good) | Disallow the `is` attribute on native HTML elements |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [Bad](#vue-no-deprecated-inline-template-bad) · [Good](#vue-no-deprecated-inline-template-good) | Disallow the deprecated `inline-template` attribute |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [Bad](#vue-no-deprecated-router-link-tag-prop-bad) · [Good](#vue-no-deprecated-router-link-tag-prop-good) | Disallow the `tag` prop on &lt;router-link&gt; |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [Bad](#vue-no-deprecated-scope-attribute-bad) · [Good](#vue-no-deprecated-scope-attribute-good) | Disallow the deprecated `scope` attribute on &lt;template&gt; |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [Bad](#vue-no-deprecated-slot-attribute-bad) · [Good](#vue-no-deprecated-slot-attribute-good) | Disallow the deprecated `slot` attribute |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [Bad](#vue-no-deprecated-slot-scope-attribute-bad) · [Good](#vue-no-deprecated-slot-scope-attribute-good) | Disallow the deprecated `slot-scope` attribute |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [Bad](#vue-no-deprecated-v-bind-sync-bad) · [Good](#vue-no-deprecated-v-bind-sync-good) | Disallow the deprecated `.sync` modifier on `v-bind` |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [Bad](#vue-no-deprecated-v-on-native-modifier-bad) · [Good](#vue-no-deprecated-v-on-native-modifier-good) | Disallow the deprecated `.native` modifier on `v-on` |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [Bad](#vue-no-deprecated-v-on-number-modifiers-bad) · [Good](#vue-no-deprecated-v-on-number-modifiers-good) | Disallow deprecated numeric `keyCode` modifiers on `v-on` |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [Bad](#vue-no-dupe-v-else-if-bad) · [Good](#vue-no-dupe-v-else-if-good) | Disallow duplicate conditions in `v-if` / `v-else-if` chains |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [Bad](#vue-no-duplicate-attributes-bad) · [Good](#vue-no-duplicate-attributes-good) | Disallow duplicate attributes on the same element |
| [`vue/no-empty-component-block`](#vue-no-empty-component-block) | [Bad](#vue-no-empty-component-block-bad) · [Good](#vue-no-empty-component-block-good) | Disallow empty SFC blocks |
| [`vue/no-inline-style`](#vue-no-inline-style) | [Bad](#vue-no-inline-style-bad) · [Good](#vue-no-inline-style-good) | Discourage use of inline style attributes |
| [`vue/no-invalid-html-attribute`](#vue-no-invalid-html-attribute) | [Bad](#vue-no-invalid-html-attribute-bad) · [Good](#vue-no-invalid-html-attribute-good) | Disallow invalid static values for HTML attributes |
| [`vue/no-lone-template`](#vue-no-lone-template) | [Bad](#vue-no-lone-template-bad) · [Good](#vue-no-lone-template-good) | Disallow unnecessary `<template>` elements |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [Bad](#vue-no-multi-spaces-bad) · [Good](#vue-no-multi-spaces-good) | Disallow multiple consecutive spaces |
| [`vue/no-multiple-objects-in-class`](#vue-no-multiple-objects-in-class) | [Bad](#vue-no-multiple-objects-in-class-bad) · [Good](#vue-no-multiple-objects-in-class-good) | Disallow multiple object literals inside a :class array binding |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [Bad](#vue-no-multiple-template-root-bad) · [Good](#vue-no-multiple-template-root-good) | Disallow multiple root nodes in a template |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [Bad](#vue-no-mutating-props-bad) · [Good](#vue-no-mutating-props-good) | Disallow mutating component props |
| [`vue/no-negated-v-if-condition`](#vue-no-negated-v-if-condition) | [Bad](#vue-no-negated-v-if-condition-bad) · [Good](#vue-no-negated-v-if-condition-good) | Disallow a negated v-if condition when the chain has a v-else |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [Bad](#vue-no-non-component-keep-alive-child-bad) · [Good](#vue-no-non-component-keep-alive-child-good) | Disallow plain element wrappers directly below `<KeepAlive>` |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [Bad](#vue-no-preprocessor-lang-bad) · [Good](#vue-no-preprocessor-lang-good) | Discourage CSS preprocessor usage in favor of modern CSS |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [Bad](#vue-no-reserved-component-names-bad) · [Good](#vue-no-reserved-component-names-good) | Disallow the use of reserved names as component names |
| [`vue/no-root-v-if`](#vue-no-root-v-if) | [Bad](#vue-no-root-v-if-bad) · [Good](#vue-no-root-v-if-good) | Disallow v-if on the single root element of a template |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [Bad](#vue-no-script-non-standard-lang-bad) · [Good](#vue-no-script-non-standard-lang-good) | Discourage non-standard script lang values |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [Bad](#vue-no-src-attribute-bad) · [Good](#vue-no-src-attribute-good) | Discourage src attribute on SFC blocks |
| [`vue/no-static-inline-styles`](#vue-no-static-inline-styles) | [Bad](#vue-no-static-inline-styles-bad) · [Good](#vue-no-static-inline-styles-good) | Disallow static inline style attributes |
| [`vue/no-template-key`](#vue-no-template-key) | [Bad](#vue-no-template-key-bad) · [Good](#vue-no-template-key-good) | Disallow `key` attribute on `<template>` |
| [`vue/no-template-lang`](#vue-no-template-lang) | [Bad](#vue-no-template-lang-bad) · [Good](#vue-no-template-lang-good) | Discourage lang attribute on template block |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [Bad](#vue-no-template-shadow-bad) · [Good](#vue-no-template-shadow-good) | Disallow variable names that shadow variables in outer scope |
| [`vue/no-template-target-blank`](#vue-no-template-target-blank) | [Bad](#vue-no-template-target-blank-bad) · [Good](#vue-no-template-target-blank-good) | Disallow target="_blank" without rel="noopener noreferrer" |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [Bad](#vue-no-textarea-mustache-bad) · [Good](#vue-no-textarea-mustache-good) | Disallow mustache interpolation in `<textarea>` |
| [`vue/no-undefined-refs`](#vue-no-undefined-refs) | [Bad](#vue-no-undefined-refs-bad) · [Good](#vue-no-undefined-refs-good) | Disallow undefined variable references in templates |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [Bad](#vue-no-unsafe-url-bad) · [Good](#vue-no-unsafe-url-good) | Warn about potentially unsafe URL bindings |
| [`vue/no-unsandboxed-iframe`](#vue-no-unsandboxed-iframe) | [Bad](#vue-no-unsandboxed-iframe-bad) · [Good](#vue-no-unsandboxed-iframe-good) | Require a sandbox attribute on iframe elements |
| [`vue/no-unused-components`](#vue-no-unused-components) | [Bad](#vue-no-unused-components-bad) · [Good](#vue-no-unused-components-good) | Disallow registering components that are not used inside templates |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [Bad](#vue-no-unused-properties-bad) · [Good](#vue-no-unused-properties-good) | Disallow unused properties defined in defineProps |
| [`vue/no-unused-refs`](#vue-no-unused-refs) | [Bad](#vue-no-unused-refs-bad) · [Good](#vue-no-unused-refs-good) | Report template refs (ref="x") never referenced in &lt;script&gt; |
| [`vue/no-unused-setup-bindings`](#vue-no-unused-setup-bindings) | [Bad](#vue-no-unused-setup-bindings-bad) · [Good](#vue-no-unused-setup-bindings-good) | Disallow unread script setup bindings |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [Bad](#vue-no-unused-vars-bad) · [Good](#vue-no-unused-vars-good) | Disallow unused variable definitions in v-for and v-slot directives |
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [Bad](#vue-no-use-v-else-with-v-for-bad) · [Good](#vue-no-use-v-else-with-v-for-good) | Disallow using `v-else-if` or `v-else` on the same element as `v-for` |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [Bad](#vue-no-use-v-if-with-v-for-bad) · [Good](#vue-no-use-v-if-with-v-for-good) | Disallow using `v-if` on the same element as `v-for` |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [Bad](#vue-no-useless-mustaches-bad) · [Good](#vue-no-useless-mustaches-good) | Disallow a mustache interpolation whose expression is a constant string literal |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [Bad](#vue-no-useless-template-attributes-bad) · [Good](#vue-no-useless-template-attributes-good) | Disallow useless attributes on `<template>` elements |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [Bad](#vue-no-useless-v-bind-bad) · [Good](#vue-no-useless-v-bind-good) | Disallow a v-bind whose value is a plain string literal |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [Bad](#vue-no-v-for-template-key-on-child-bad) · [Good](#vue-no-v-for-template-key-on-child-good) | Disallow `key` on the child of a `<template v-for>` |
| [`vue/no-v-html`](#vue-no-v-html) | [Bad](#vue-no-v-html-bad) · [Good](#vue-no-v-html-good) | Warn against v-html to prevent XSS vulnerabilities |
| [`vue/no-v-text`](#vue-no-v-text) | [Bad](#vue-no-v-text-bad) · [Good](#vue-no-v-text-good) | Disallow the v-text directive; prefer mustache interpolation |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [Bad](#vue-no-v-text-v-html-on-component-bad) · [Good](#vue-no-v-text-v-html-on-component-good) | Disallow v-text / v-html on component elements |
| [`vue/permitted-contents`](#vue-permitted-contents) | [Bad](#vue-permitted-contents-bad) · [Good](#vue-permitted-contents-good) | Enforce HTML content model rules |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [Bad](#vue-prefer-props-shorthand-bad) · [Good](#vue-prefer-props-shorthand-good) | Recommend shorthand syntax for props (Vue 3.4+) |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [Bad](#vue-prefer-true-attribute-shorthand-bad) · [Good](#vue-prefer-true-attribute-shorthand-good) | Prefer the shorthand for a boolean attribute bound to `true` |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [Bad](#vue-prop-name-casing-bad) · [Good](#vue-prop-name-casing-good) | Enforce a casing for declared prop names |
| [`vue/require-component-is`](#vue-require-component-is) | [Bad](#vue-require-component-is-bad) · [Good](#vue-require-component-is-good) | Require `v-bind:is` on `<component>` elements |
| [`vue/require-component-registration`](#vue-require-component-registration) | [Bad](#vue-require-component-registration-bad) · [Good](#vue-require-component-registration-good) | Require explicit import or registration for components |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [Bad](#vue-require-scoped-style-bad) · [Good](#vue-require-scoped-style-good) | Require scoped attribute on style tags |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [Bad](#vue-require-toggle-inside-transition-bad) · [Good](#vue-require-toggle-inside-transition-good) | Require a toggle on the element wrapped by `<transition>` |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [Bad](#vue-require-v-for-key-bad) · [Good](#vue-require-v-for-key-good) | Require `v-bind:key` with `v-for` directives |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [Bad](#vue-scoped-event-names-bad) · [Good](#vue-scoped-event-names-good) | Recommend scoped event names using context:event format |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [Bad](#vue-sfc-element-order-bad) · [Good](#vue-sfc-element-order-good) | Enforce consistent order of SFC top-level elements |
| [`vue/single-style-block`](#vue-single-style-block) | [Bad](#vue-single-style-block-bad) · [Good](#vue-single-style-block-good) | Recommend having a single style block |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [Bad](#vue-slot-name-casing-bad) · [Good](#vue-slot-name-casing-good) | Enforce kebab-case for named slots used via v-slot |
| [`vue/this-in-template`](#vue-this-in-template) | [Bad](#vue-this-in-template-bad) · [Good](#vue-this-in-template-good) | Disallow `this.` in template expressions |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Bad](#vue-use-unique-element-ids-bad) · [Good](#vue-use-unique-element-ids-good) | Enforce unique element IDs using useId() instead of static literals |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [Bad](#vue-use-v-on-exact-bad) · [Good](#vue-use-v-on-exact-good) | Enforce `.exact` modifier on `v-on` when there are modifier-based handlers |
| [`vue/v-bind-style`](#vue-v-bind-style) | [Bad](#vue-v-bind-style-bad) · [Good](#vue-v-bind-style-good) | Enforce `v-bind` directive style |
| [`vue/v-on-event-hyphenation`](#vue-v-on-event-hyphenation) | [Bad](#vue-v-on-event-hyphenation-bad) · [Good](#vue-v-on-event-hyphenation-good) | Enforce hyphenation of custom event names in v-on on components |
| [`vue/v-on-handler-style`](#vue-v-on-handler-style) | [Bad](#vue-v-on-handler-style-bad) · [Good](#vue-v-on-handler-style-good) | Enforce writing v-on handlers as a method reference or an inline function |
| [`vue/v-on-style`](#vue-v-on-style) | [Bad](#vue-v-on-style-bad) · [Good](#vue-v-on-style-good) | Enforce `v-on` directive style |
| [`vue/v-slot-style`](#vue-v-slot-style) | [Bad](#vue-v-slot-style-bad) · [Good](#vue-v-slot-style-good) | Enforce `v-slot` directive style |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [Bad](#vue-valid-attribute-name-bad) · [Good](#vue-valid-attribute-name-good) | Require valid attribute names |
| [`vue/valid-template-root`](#vue-valid-template-root) | [Bad](#vue-valid-template-root-bad) · [Good](#vue-valid-template-root-good) | Enforce a valid `<template>` root for Vue 3 fragment semantics |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [Bad](#vue-valid-v-bind-bad) · [Good](#vue-valid-v-bind-good) | Enforce valid `v-bind` directives |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [Bad](#vue-valid-v-cloak-bad) · [Good](#vue-valid-v-cloak-good) | Enforce valid `v-cloak` directives |
| [`vue/valid-v-else`](#vue-valid-v-else) | [Bad](#vue-valid-v-else-bad) · [Good](#vue-valid-v-else-good) | Enforce valid `v-else` directives |
| [`vue/valid-v-for`](#vue-valid-v-for) | [Bad](#vue-valid-v-for-bad) · [Good](#vue-valid-v-for-good) | Enforce valid `v-for` directives |
| [`vue/valid-v-html`](#vue-valid-v-html) | [Bad](#vue-valid-v-html-bad) · [Good](#vue-valid-v-html-good) | Enforce valid `v-html` directives |
| [`vue/valid-v-if`](#vue-valid-v-if) | [Bad](#vue-valid-v-if-bad) · [Good](#vue-valid-v-if-good) | Enforce valid `v-if` directives |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [Bad](#vue-valid-v-memo-bad) · [Good](#vue-valid-v-memo-good) | Enforce valid `v-memo` directives |
| [`vue/valid-v-model`](#vue-valid-v-model) | [Bad](#vue-valid-v-model-bad) · [Good](#vue-valid-v-model-good) | Enforce valid `v-model` directives |
| [`vue/valid-v-on`](#vue-valid-v-on) | [Bad](#vue-valid-v-on-bad) · [Good](#vue-valid-v-on-good) | Enforce valid `v-on` directives |
| [`vue/valid-v-once`](#vue-valid-v-once) | [Bad](#vue-valid-v-once-bad) · [Good](#vue-valid-v-once-good) | Enforce valid `v-once` directives |
| [`vue/valid-v-show`](#vue-valid-v-show) | [Bad](#vue-valid-v-show-bad) · [Good](#vue-valid-v-show-good) | Enforce valid `v-show` directives |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [Bad](#vue-valid-v-slot-bad) · [Good](#vue-valid-v-slot-good) | Enforce valid `v-slot` directives |
| [`vue/valid-v-text`](#vue-valid-v-text) | [Bad](#vue-valid-v-text-bad) · [Good](#vue-valid-v-text-good) | Enforce valid `v-text` directives |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [Bad](#vue-warn-custom-block-bad) · [Good](#vue-warn-custom-block-good) | Warn about custom blocks in SFC files |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [Bad](#vue-warn-custom-directive-bad) · [Good](#vue-warn-custom-directive-good) | Warn about custom directives that need registration |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `vue/a11y-img-alt`

Require alt attribute on images for accessibility

[Bad](#vue-a11y-img-alt-bad) · [Good](#vue-a11y-img-alt-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

Neither the static image nor the dynamically sourced image supplies an alt attribute.

```vue annotate="remove:2,3"
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

<span id="vue-a11y-img-alt-good"></span>

**Good**

Informative images get descriptive alt text, decoration gets an empty alt, and the dynamic image binds its description.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) · [All rules](all.md)

### `vue/attribute-hyphenation`

Enforce attribute naming style on custom components

[Bad](#vue-attribute-hyphenation-bad) · [Good](#vue-attribute-hyphenation-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

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

**Bad**

The component attribute uses the camelCase spelling firstName.

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**Good**

The first-name spelling follows the configured hyphenated component-attribute convention.

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [All rules](all.md)

### `vue/attribute-order`

Enforce a consistent order of attributes

[Bad](#vue-attribute-order-bad) · [Good](#vue-attribute-order-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The event handler appears before the structural v-if directive and ordinary id attribute.

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**Good**

v-if comes first, followed by id and then the event handler, following the rule ordering.

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [All rules](all.md)

### `vue/component-definition-name-casing`

Enforce PascalCase or kebab-case for component definition names

[Bad](#vue-component-definition-name-casing-bad) · [Good](#vue-component-definition-name-casing-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

The component filename is checked. PascalCase and kebab-case are accepted; mixed casing is reported.

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

**Bad**

The filename myComponent.vue mixes a lowercase initial with an internal uppercase letter instead of using PascalCase or kebab-case.

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

<span id="vue-component-definition-name-casing-good"></span>

**Good**

Renaming the file to MyComponent.vue uses PascalCase; its template content is unchanged.

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [All rules](all.md)

### `vue/component-name-in-template-casing`

Enforce specific casing for component names in templates

[Bad](#vue-component-name-in-template-casing-bad) · [Good](#vue-component-name-in-template-casing-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

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

**Bad**

The component is written in kebab-case and camelCase under the PascalCase convention.

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

**Good**

MyComponent uses PascalCase; native slot syntax remains lowercase.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) · [All rules](all.md)

### `vue/html-button-has-type`

Require an explicit valid type on button elements

[Bad](#vue-html-button-has-type-bad) · [Good](#vue-html-button-has-type-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

One button omits type and another supplies the unsupported foo type.

```vue annotate="remove:2,3"
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

<span id="vue-html-button-has-type-good"></span>

**Good**

Buttons specify button, submit, or reset; a bound type is treated as dynamic.

```vue annotate="add:2,3,4,5"
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) · [All rules](all.md)

### `vue/html-quotes`

Enforce quotes style of HTML attributes

[Bad](#vue-html-quotes-bad) · [Good](#vue-html-quotes-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The attributes use single quotes or no quotes instead of the double-quote convention.

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**Good**

Both ordinary attributes and directive expressions use double quotes.

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [All rules](all.md)

### `vue/html-self-closing`

Enforce self-closing style

[Bad](#vue-html-self-closing-bad) · [Good](#vue-html-self-closing-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

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

**Bad**

The empty component uses a closing pair, while void img and br elements omit the configured self-closing spelling.

```vue annotate="remove:2,3,4"
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**Good**

The component and void elements use self-closing syntax; a div with content retains its closing tag.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [All rules](all.md)

### `vue/max-template-complexity`

Limit a component's own template complexity (cyclomatic and cognitive)

[Bad](#vue-max-template-complexity-bad) · [Good](#vue-max-template-complexity-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Bad has cyclomatic complexity 13 and cognitive complexity 25 (limits: 11 and 16). Each component is measured separately; only inline HTML templates are supported.

See [complexity scoring and component boundaries](../guide/cross-file-complexity.md) for the contributions behind the example's two scores.

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

**Bad**

The parent-authored branches, loop, slot content, and expression decisions produce scores of 13 and 25, above the default limits 11 and 16.

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

**Good**

The parent template delegates rendering to RowList and keeps one v-if; its own scores are 2 and 1.

```vue annotate="add:2"
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) · [All rules](all.md)

### `vue/multi-word-component-names`

Require component names to be multi-word

[Bad](#vue-multi-word-component-names-bad) · [Good](#vue-multi-word-component-names-good)

Default severity: `error`  
Presets: `essential`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

The filename is the finding. Rename the same component; changing a child tag does not fix it.

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

**Bad**

Item.vue gives the component a single-word name.

`Item.vue`

```vue
<template><p>Item</p></template>
```

<span id="vue-multi-word-component-names-good"></span>

**Good**

TodoItem.vue gives the same template a multi-word component name.

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [All rules](all.md)

### `vue/mustache-interpolation-spacing`

Enforce consistent spacing inside mustache interpolations

[Bad](#vue-mustache-interpolation-spacing-bad) · [Good](#vue-mustache-interpolation-spacing-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The text interpolation is missing a space at one or both delimiter boundaries.

```vue annotate="remove:2,3,4"
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**Good**

Spaces separate the expression from both opening and closing mustache delimiters.

```vue annotate="add:2,3,4"
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [All rules](all.md)

### `vue/no-array-index-key`

Disallow using the v-for index variable directly as the :key

[Bad](#vue-no-array-index-key-bad) · [Good](#vue-no-array-index-key-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The list key is its current index, so item identity changes when the list is reordered.

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

<span id="vue-no-array-index-key-good"></span>

**Good**

The key comes from item.id, preserving the identity of each item across position changes.

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) · [All rules](all.md)

### `vue/no-bare-strings-in-template`

Disallow raw human-readable text in the template that should be internationalized

[Bad](#vue-no-bare-strings-in-template-bad) · [Good](#vue-no-bare-strings-in-template-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

Visible text and naming attributes embed untranslated strings directly in the template.

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

<span id="vue-no-bare-strings-in-template-good"></span>

**Good**

Translatable content calls $t; the punctuation and numeric-only examples are allowed exceptions.

```vue annotate="add:2,3,4,5,6"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<div>123</div>
<button :title="$t('close')">{{ $t('x') }}</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47) · [All rules](all.md)

### `vue/no-boolean-attr-value`

Disallow explicit values for boolean HTML attributes

[Bad](#vue-no-boolean-attr-value-bad) · [Good](#vue-no-boolean-attr-value-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The boolean disabled and checked attributes redundantly contain string values.

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**Good**

The presence of each boolean attribute expresses the same enabled state without a value.

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [All rules](all.md)

### `vue/no-child-content`

Disallow child content when using v-html or v-text

[Bad](#vue-no-child-content-bad) · [Good](#vue-no-child-content-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

v-text replaces the paragraph content, so the authored fallback text cannot survive that directive.

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**Good**

Removing the child text leaves v-text as the single source of paragraph content.

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [All rules](all.md)

### `vue/no-deprecated-filter`

Disallow deprecated Vue 2 filter syntax using the pipe operator

[Bad](#vue-no-deprecated-filter-bad) · [Good](#vue-no-deprecated-filter-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The pipe uses the removed Vue filter syntax to apply capitalize.

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

<span id="vue-no-deprecated-filter-good"></span>

**Good**

Calling capitalize(message) applies the transformation as an ordinary expression.

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) · [All rules](all.md)

### `vue/no-deprecated-functional-template`

Disallow the `functional` attribute on the SFC `<template>`

[Bad](#vue-no-deprecated-functional-template-bad) · [Good](#vue-no-deprecated-functional-template-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The SFC template has the removed functional attribute and reads the old props context.

```vue annotate="remove:1,2"
<template functional>
<div>{{ props.msg }}</div>
</template>
```

<span id="vue-no-deprecated-functional-template-good"></span>

**Good**

The ordinary template omits functional and reads the component binding msg directly.

```vue annotate="add:1,2"
<template>
<div>{{ msg }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) · [All rules](all.md)

### `vue/no-deprecated-html-element-is`

Disallow the `is` attribute on native HTML elements

[Bad](#vue-no-deprecated-html-element-is-bad) · [Good](#vue-no-deprecated-html-element-is-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

A native div uses the old unprefixed is attribute to request a Vue component.

```vue annotate="remove:2"
<template>
<div is="MyComponent" />
</template>
```

<span id="vue-no-deprecated-html-element-is-good"></span>

**Good**

A dynamic component uses :is; the native-element spelling explicitly uses the vue: prefix.

```vue annotate="add:2,3"
<template>
<component :is="MyComponent" />
<div is="vue:MyComponent" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) · [All rules](all.md)

### `vue/no-deprecated-inline-template`

Disallow the deprecated `inline-template` attribute

[Bad](#vue-no-deprecated-inline-template-bad) · [Good](#vue-no-deprecated-inline-template-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

Card uses the deprecated inline-template attribute for its supplied content.

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

<span id="vue-no-deprecated-inline-template-good"></span>

**Good**

The same content is passed normally without the inline-template attribute.

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) · [All rules](all.md)

### `vue/no-deprecated-router-link-tag-prop`

Disallow the `tag` prop on &lt;router-link&gt;

[Bad](#vue-no-deprecated-router-link-tag-prop-bad) · [Good](#vue-no-deprecated-router-link-tag-prop-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

RouterLink uses the removed tag prop to request a button element.

```vue annotate="remove:2"
<template>
<router-link to="/home" tag="button">Home</router-link>
</template>
```

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

**Good**

The slot provides navigate to an explicitly authored button.

```vue annotate="add:2,3,4"
<template>
<router-link to="/home" v-slot="{ navigate }">
<button @click="navigate">Home</button>
</router-link>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [All rules](all.md)

### `vue/no-deprecated-scope-attribute`

Disallow the deprecated `scope` attribute on &lt;template&gt;

[Bad](#vue-no-deprecated-scope-attribute-bad) · [Good](#vue-no-deprecated-scope-attribute-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The slot template declares props through the deprecated scope attribute.

```vue annotate="remove:2"
<template>
<Card><template scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-scope-attribute-good"></span>

**Good**

The default-slot directive declares the same props binding through current slot syntax.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) · [All rules](all.md)

### `vue/no-deprecated-slot-attribute`

Disallow the deprecated `slot` attribute

[Bad](#vue-no-deprecated-slot-attribute-bad) · [Good](#vue-no-deprecated-slot-attribute-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The header slot is selected through the old slot attribute.

```vue annotate="remove:3,4"
<template>
<Foo>
<template slot="header"><h1>Title</h1></template>
<div :slot="name">Title</div>
</Foo>
</template>
```

<span id="vue-no-deprecated-slot-attribute-good"></span>

**Good**

v-slot:header explicitly selects the header slot with the current directive.

```vue annotate="add:3"
<template>
<Foo>
<template v-slot:header><h1>Title</h1></template>
</Foo>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) · [All rules](all.md)

### `vue/no-deprecated-slot-scope-attribute`

Disallow the deprecated `slot-scope` attribute

[Bad](#vue-no-deprecated-slot-scope-attribute-bad) · [Good](#vue-no-deprecated-slot-scope-attribute-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The template receives slot props through the deprecated slot-scope attribute.

```vue annotate="remove:2"
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

**Good**

The #default directive receives those props without slot-scope.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) · [All rules](all.md)

### `vue/no-deprecated-v-bind-sync`

Disallow the deprecated `.sync` modifier on `v-bind`

[Bad](#vue-no-deprecated-v-bind-sync-bad) · [Good](#vue-no-deprecated-v-bind-sync-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The bindings use the removed .sync modifier, including its combination with .camel.

```vue annotate="remove:2,3,4"
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

<span id="vue-no-deprecated-v-bind-sync-good"></span>

**Good**

Use an ordinary one-way title binding or v-model:title when an update channel is required.

```vue annotate="add:2,3"
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) · [All rules](all.md)

### `vue/no-deprecated-v-on-native-modifier`

Disallow the deprecated `.native` modifier on `v-on`

[Bad](#vue-no-deprecated-v-on-native-modifier-bad) · [Good](#vue-no-deprecated-v-on-native-modifier-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The component handlers use the removed .native event modifier.

```vue annotate="remove:2,3,4"
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

**Good**

The handlers omit .native and preserve other event modifiers such as .stop.

```vue annotate="add:2,3"
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) · [All rules](all.md)

### `vue/no-deprecated-v-on-number-modifiers`

Disallow deprecated numeric `keyCode` modifiers on `v-on`

[Bad](#vue-no-deprecated-v-on-number-modifiers-bad) · [Good](#vue-no-deprecated-v-on-number-modifiers-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The keyboard handlers identify keys by the removed numeric codes 13 and 27.

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

**Good**

The handlers use the named enter and esc key modifiers.

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [All rules](all.md)

### `vue/no-dupe-v-else-if`

Disallow duplicate conditions in `v-if` / `v-else-if` chains

[Bad](#vue-no-dupe-v-else-if-bad) · [Good](#vue-no-dupe-v-else-if-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The else-if repeats the ready condition already tested by the first branch, making that later branch unreachable.

```vue annotate="remove:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**Good**

The second branch tests loading, a distinct state that can reach the else-if.

```vue annotate="add:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [All rules](all.md)

### `vue/no-duplicate-attributes`

Disallow duplicate attributes on the same element

[Bad](#vue-no-duplicate-attributes-bad) · [Good](#vue-no-duplicate-attributes-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The same button declares class twice instead of one combined class value.

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**Good**

Both class tokens appear in a single class attribute.

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [All rules](all.md)

### `vue/no-empty-component-block`

Disallow empty SFC blocks

[Bad](#vue-no-empty-component-block-bad) · [Good](#vue-no-empty-component-block-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The template, script, and style blocks contain no meaningful content.

```vue annotate="remove:1,3,5"
<template></template>

<script></script>

<style>
</style>
```

<span id="vue-no-empty-component-block-good"></span>

**Good**

Each retained block contains actual markup, script declarations, or style declarations.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) · [All rules](all.md)

### `vue/no-inline-style`

Discourage use of inline style attributes

[Bad](#vue-no-inline-style-bad) · [Good](#vue-no-inline-style-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The static style attribute embeds the color declaration in the element.

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**Good**

Classes express the fixed color; the ratio-dependent width remains a dynamic style binding, outside the static-attribute check.

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [All rules](all.md)

### `vue/no-invalid-html-attribute`

Disallow invalid static values for HTML attributes

[Bad](#vue-no-invalid-html-attribute-bad) · [Good](#vue-no-invalid-html-attribute-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The anchor uses stylesheet as a rel value, although that value belongs to stylesheet link elements.

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

<span id="vue-no-invalid-html-attribute-good"></span>

**Good**

The anchor uses help, a rel value appropriate for a linked help resource.

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) · [All rules](all.md)

### `vue/no-lone-template`

Disallow unnecessary `<template>` elements

[Bad](#vue-no-lone-template-bad) · [Good](#vue-no-lone-template-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The inner template has no directive or slot role that gives it a structural purpose.

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**Good**

Removing the unnecessary wrapper leaves the paragraph directly inside div.

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [All rules](all.md)

### `vue/no-multi-spaces`

Disallow multiple consecutive spaces

[Bad](#vue-no-multi-spaces-bad) · [Good](#vue-no-multi-spaces-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

Two spaces separate attributes or the element name and the first attribute.

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**Good**

Single spaces separate the same attributes.

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [All rules](all.md)

### `vue/no-multiple-objects-in-class`

Disallow multiple object literals inside a :class array binding

[Bad](#vue-no-multiple-objects-in-class-bad) · [Good](#vue-no-multiple-objects-in-class-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

A class array contains two top-level object literals that can be merged.

```vue annotate="remove:2,3"
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

<span id="vue-no-multiple-objects-in-class-good"></span>

**Good**

One object contains the class conditions; arrays with one object and a string or with non-literal entries remain allowed.

```vue annotate="add:2,3,4"
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) · [All rules](all.md)

### `vue/no-multiple-template-root`

Disallow multiple root nodes in a template

[Bad](#vue-no-multiple-template-root-bad) · [Good](#vue-no-multiple-template-root-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Enable only for a single-root contract. Vue 3 normally supports fragments.

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

**Bad**

The opt-in single-root convention sees two sibling paragraphs at the template root.

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

<span id="vue-no-multiple-template-root-good"></span>

**Good**

A section wraps the paragraphs into one root; enable this convention only when a single-root contract is intended.

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) · [All rules](all.md)

### `vue/no-mutating-props`

Disallow mutating component props

[Bad](#vue-no-mutating-props-bad) · [Good](#vue-no-mutating-props-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

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

**Bad**

Incrementing props.count writes directly to a value supplied by the parent.

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**Good**

The component emits update:count with the next value, leaving the parent responsible for updating the prop.

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L42) · [All rules](all.md)

### `vue/no-negated-v-if-condition`

Disallow a negated v-if condition when the chain has a v-else

[Bad](#vue-no-negated-v-if-condition-bad) · [Good](#vue-no-negated-v-if-condition-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The paired v-if and v-else branches begin with a negated condition.

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

<span id="vue-no-negated-v-if-condition-good"></span>

**Good**

A positive ok condition comes first; when inverting a condition, place the original opposite branch first. A lone negated v-if and !== comparisons remain allowed.

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) · [All rules](all.md)

### `vue/no-non-component-keep-alive-child`

Disallow plain element wrappers directly below `<KeepAlive>`

[Bad](#vue-no-non-component-keep-alive-child-bad) · [Good](#vue-no-non-component-keep-alive-child-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

KeepAlive conditionally wraps a native div rather than directly caching UserCard.

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

**Good**

The first example makes UserCard the conditional child. The v-show wrapper illustrates a shape outside this conditional-child check, not a promise that the native wrapper is cached.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L14) · [All rules](all.md)

### `vue/no-preprocessor-lang`

Discourage CSS preprocessor usage in favor of modern CSS

[Bad](#vue-no-preprocessor-lang-bad) · [Good](#vue-no-preprocessor-lang-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Not implemented for SFC lint  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Current support: `no-sfc-finding`

This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.

**Configured ID (currently no SFC finding)**

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

**Bad**

The style block selects SCSS with lang. This describes the intended no-preprocessor convention; the current SFC path does not emit this rule.

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**Good**

The same CSS declarations omit the preprocessor lang. This is the convention repair, not an executable Bad/Good diagnostic difference today.

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [All rules](all.md)

### `vue/no-reserved-component-names`

Disallow the use of reserved names as component names

[Bad](#vue-no-reserved-component-names-bad) · [Good](#vue-no-reserved-component-names-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The component name button conflicts with a native HTML element name.

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**Good**

AppButton is an application component name and does not reuse the native button name.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) · [All rules](all.md)

### `vue/no-root-v-if`

Disallow v-if on the single root element of a template

[Bad](#vue-no-root-v-if-bad) · [Good](#vue-no-root-v-if-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The component root itself appears and disappears under v-if.

```vue annotate="remove:2"
<template>
<div v-if="show">content</div>
</template>
```

<span id="vue-no-root-v-if-good"></span>

**Good**

A stable outer div remains the root while the nested paragraph carries the visibility condition.

```vue annotate="add:2,3,4"
<template>
<div>
<p v-if="show">content</p>
</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [All rules](all.md)

### `vue/no-script-non-standard-lang`

Discourage non-standard script lang values

[Bad](#vue-no-script-non-standard-lang-bad) · [Good](#vue-no-script-non-standard-lang-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Not implemented for SFC lint  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Current support: `no-sfc-finding`

This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.

**Configured ID (currently no SFC finding)**

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

**Bad**

The script uses CoffeeScript syntax under lang=coffee. The current SFC path does not emit this catalog rule for that language.

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**Good**

The script uses an ordinary TypeScript declaration with lang=ts, illustrating the intended language convention.

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [All rules](all.md)

### `vue/no-src-attribute`

Discourage src attribute on SFC blocks

[Bad](#vue-no-src-attribute-bad) · [Good](#vue-no-src-attribute-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The SFC blocks delegate their template, script, and style content to src files.

```vue annotate="remove:1,2,3"
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**Good**

Each SFC block contains its own content without an external src attribute.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) · [All rules](all.md)

### `vue/no-static-inline-styles`

Disallow static inline style attributes

[Bad](#vue-no-static-inline-styles-bad) · [Good](#vue-no-static-inline-styles-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The paragraph carries the constant color declaration in its style attribute.

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

<span id="vue-no-static-inline-styles-good"></span>

**Good**

A notice class and scoped stylesheet hold the constant color outside the template attribute.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [All rules](all.md)

### `vue/no-template-key`

Disallow `key` attribute on `<template>`

[Bad](#vue-no-template-key-bad) · [Good](#vue-no-template-key-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

A non-loop template wrapper has a key even though it is not the keyed iteration boundary.

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**Good**

The key belongs to a template v-for iteration, where it identifies each repeated fragment.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [All rules](all.md)

### `vue/no-template-lang`

Discourage lang attribute on template block

[Bad](#vue-no-template-lang-bad) · [Good](#vue-no-template-lang-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Not implemented for SFC lint  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Current support: `no-sfc-finding`

This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.

**Configured ID (currently no SFC finding)**

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

**Bad**

The template selects Pug through lang. This is an intended HTML-only convention; the current SFC path does not diagnose this catalog ID.

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**Good**

An ordinary HTML template omits lang and uses the paragraph directly. This illustrates the convention without claiming a current SFC finding.

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [All rules](all.md)

### `vue/no-template-shadow`

Disallow variable names that shadow variables in outer scope

[Bad](#vue-no-template-shadow-bad) · [Good](#vue-no-template-shadow-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

The current check compares nested v-for bindings. It does not report a single v-for binding merely because it shares a script binding's name.

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

**Bad**

The inner v-for declares item again and hides the outer item binding inside the nested loop.

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**Good**

The inner loop declares child, leaving item available for the outer row and child for the nested row.

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [All rules](all.md)

### `vue/no-template-target-blank`

Disallow target="_blank" without rel="noopener noreferrer"

[Bad](#vue-no-template-target-blank-bad) · [Good](#vue-no-template-target-blank-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The external link opens a new browsing context without the expected rel protection.

```vue annotate="remove:2"
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

<span id="vue-no-template-target-blank-good"></span>

**Good**

The same link includes noopener noreferrer alongside target=_blank.

```vue annotate="add:2"
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [All rules](all.md)

### `vue/no-textarea-mustache`

Disallow mustache interpolation in `<textarea>`

[Bad](#vue-no-textarea-mustache-bad) · [Good](#vue-no-textarea-mustache-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The textarea places message in child interpolation instead of binding its value.

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**Good**

v-model binds the editable textarea value to message.

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [All rules](all.md)

### `vue/no-undefined-refs`

Disallow undefined variable references in templates

[Bad](#vue-no-undefined-refs-bad) · [Good](#vue-no-undefined-refs-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The template reads missing, although the script declares only message.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

<span id="vue-no-undefined-refs-good"></span>

**Good**

The interpolation reads the existing message binding.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) · [All rules](all.md)

### `vue/no-unsafe-url`

Warn about potentially unsafe URL bindings

[Bad](#vue-no-unsafe-url-bad) · [Good](#vue-no-unsafe-url-good)

Default severity: `warning`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The anchor destination begins with the executable javascript: scheme.

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**Good**

The anchor uses the ordinary local /next navigation destination.

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [All rules](all.md)

### `vue/no-unsandboxed-iframe`

Require a sandbox attribute on iframe elements

[Bad](#vue-no-unsandboxed-iframe-bad) · [Good](#vue-no-unsandboxed-iframe-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The embedded frame has no sandbox attribute limiting its capabilities.

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

<span id="vue-no-unsandboxed-iframe-good"></span>

**Good**

sandbox applies restrictions; allow-scripts explicitly opts into that one capability when needed.

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) · [All rules](all.md)

### `vue/no-unused-components`

Disallow registering components that are not used inside templates

[Bad](#vue-no-unused-components-bad) · [Good](#vue-no-unused-components-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

UserAvatar is imported as a component but the template never renders it.

```vue annotate="remove:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

<span id="vue-no-unused-components-good"></span>

**Good**

The template renders the imported UserAvatar and passes the user binding.

```vue annotate="add:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) · [All rules](all.md)

### `vue/no-unused-properties`

Disallow unused properties defined in defineProps

[Bad](#vue-no-unused-properties-bad) · [Good](#vue-no-unused-properties-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The component declares description as a prop but renders only title.

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

<span id="vue-no-unused-properties-good"></span>

**Good**

Both declared props are referenced by the template.

```vue annotate="add:7"
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) · [All rules](all.md)

### `vue/no-unused-refs`

Report template refs (ref="x") never referenced in &lt;script&gt;

[Bad](#vue-no-unused-refs-bad) · [Good](#vue-no-unused-refs-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The template declares the unused ref name with no corresponding script reference binding.

```vue annotate="remove:1,3"
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

<span id="vue-no-unused-refs-good"></span>

**Good**

The inputEl template ref has a same-named ref binding in script setup.

```vue annotate="add:1,3,4"
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) · [All rules](all.md)

### `vue/no-unused-setup-bindings`

Disallow unread script setup bindings

[Bad](#vue-no-unused-setup-bindings-bad) · [Good](#vue-no-unused-setup-bindings-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The script setup message binding is never read by the template.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

<span id="vue-no-unused-setup-bindings-good"></span>

**Good**

The paragraph interpolates message, using the declared binding.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [All rules](all.md)

### `vue/no-unused-vars`

Disallow unused variable definitions in v-for and v-slot directives

[Bad](#vue-no-unused-vars-bad) · [Good](#vue-no-unused-vars-good)

Default severity: `warning`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The loop declares an unused index and the slot declares foo without referencing it.

```vue annotate="remove:2,3,4"
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

<span id="vue-no-unused-vars-good"></span>

**Good**

The examples consume index or mark it intentionally unused as _index, and the slot renders data. Index keys are only a usage example here, not a recommendation for stable item identity.

```vue annotate="add:2,3,4,5"
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) · [All rules](all.md)

### `vue/no-use-v-else-with-v-for`

Disallow using `v-else-if` or `v-else` on the same element as `v-for`

[Bad](#vue-no-use-v-else-with-v-for-bad) · [Good](#vue-no-use-v-else-with-v-for-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The else branch and v-for iteration are attached to the same paragraph.

```vue annotate="remove:3"
<template>
<p v-if="ready">Ready</p>
<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

<span id="vue-no-use-v-else-with-v-for-good"></span>

**Good**

A separate template owns v-else, and its child paragraph owns v-for.

```vue annotate="add:3"
<template>
<p v-if="ready">Ready</p>
<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) · [All rules](all.md)

### `vue/no-use-v-if-with-v-for`

Disallow using `v-if` on the same element as `v-for`

[Bad](#vue-no-use-v-if-with-v-for-bad) · [Good](#vue-no-use-v-if-with-v-for-good)

Default severity: `warning`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The same list element combines v-if and v-for and tests visibility through the loop binding.

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**Good**

A computed collection filters the visible items before the template iterates over them.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) · [All rules](all.md)

### `vue/no-useless-mustaches`

Disallow a mustache interpolation whose expression is a constant string literal

[Bad](#vue-no-useless-mustaches-bad) · [Good](#vue-no-useless-mustaches-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The interpolation contains only a constant string and does not need expression evaluation.

```vue annotate="remove:2,3,4"
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

<span id="vue-no-useless-mustaches-good"></span>

**Good**

Literal text is written directly; variable expressions, interpolated template strings, and intentional separator whitespace remain interpolation cases.

```vue annotate="add:2,3,4,5"
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) · [All rules](all.md)

### `vue/no-useless-template-attributes`

Disallow useless attributes on `<template>` elements

[Bad](#vue-no-useless-template-attributes-bad) · [Good](#vue-no-useless-template-attributes-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The conditional template has a class, but this structural wrapper does not render a DOM element to receive it.

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**Good**

The class moves to the paragraph that actually renders while v-if stays on the structural template.

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [All rules](all.md)

### `vue/no-useless-v-bind`

Disallow a v-bind whose value is a plain string literal

[Bad](#vue-no-useless-v-bind-bad) · [Good](#vue-no-useless-v-bind-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The foo binding evaluates a constant quoted string or a template string without interpolation.

```vue annotate="remove:2,3"
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

<span id="vue-no-useless-v-bind-good"></span>

**Good**

The constant value becomes a static attribute; variable and interpolated values retain their binding.

```vue annotate="add:2,3,4"
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [All rules](all.md)

### `vue/no-v-for-template-key-on-child`

Disallow `key` on the child of a `<template v-for>`

[Bad](#vue-no-v-for-template-key-on-child-bad) · [Good](#vue-no-v-for-template-key-on-child-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The child paragraph has the key while the template iteration itself has no key.

```vue annotate="remove:2"
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

<span id="vue-no-v-for-template-key-on-child-good"></span>

**Good**

The key moves to template v-for, identifying the complete repeated fragment.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) · [All rules](all.md)

### `vue/no-v-html`

Warn against v-html to prevent XSS vulnerabilities

[Bad](#vue-no-v-html-bad) · [Good](#vue-no-v-html-good)

Default severity: `warning`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

v-html interprets content as HTML rather than ordinary text.

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**Good**

Mustache interpolation displays content as escaped text instead of injecting HTML.

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [All rules](all.md)

### `vue/no-v-text`

Disallow the v-text directive; prefer mustache interpolation

[Bad](#vue-no-v-text-bad) · [Good](#vue-no-v-text-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The div's content is supplied through the v-text directive.

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

<span id="vue-no-v-text-good"></span>

**Good**

Mustache interpolation expresses the same text binding directly in the element content.

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) · [All rules](all.md)

### `vue/no-v-text-v-html-on-component`

Disallow v-text / v-html on component elements

[Bad](#vue-no-v-text-v-html-on-component-bad) · [Good](#vue-no-v-text-v-html-on-component-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The component tag receives v-html or v-text, which replaces element content rather than supplying component slots.

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**Good**

Native HTML targets can receive the directives; MyComponent receives its content through the default slot.

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [All rules](all.md)

### `vue/permitted-contents`

Enforce HTML content model rules

[Bad](#vue-permitted-contents-bad) · [Good](#vue-permitted-contents-good)

Default severity: `error`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The examples put block content in p, omit the table body, nest interactive controls, or put a div directly inside ul.

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

<span id="vue-permitted-contents-good"></span>

**Good**

The examples use inline paragraph content, an explicit tbody, and li children. The custom MyItem is not treated as a known native ul child.

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [All rules](all.md)

### `vue/prefer-props-shorthand`

Recommend shorthand syntax for props (Vue 3.4+)

[Bad](#vue-prefer-props-shorthand-bad) · [Good](#vue-prefer-props-shorthand-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

Each binding repeats the corresponding variable name, including the camelCase equivalent of a hyphenated argument.

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

<span id="vue-prefer-props-shorthand-good"></span>

**Good**

Vue 3.4+ same-name binding shorthand removes the repeated expressions; a different source variable such as bar remains explicit.

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) · [All rules](all.md)

### `vue/prefer-true-attribute-shorthand`

Prefer the shorthand for a boolean attribute bound to `true`

[Bad](#vue-prefer-true-attribute-shorthand-bad) · [Good](#vue-prefer-true-attribute-shorthand-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

A native boolean disabled attribute binds the constant true value.

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

<span id="vue-prefer-true-attribute-shorthand-good"></span>

**Good**

The native attribute uses its boolean shorthand. False bindings and component props retain their explicit values.

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) · [All rules](all.md)

### `vue/prop-name-casing`

Enforce a casing for declared prop names

[Bad](#vue-prop-name-casing-bad) · [Good](#vue-prop-name-casing-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Checks declared prop names, not the casing of attributes passed to a child.

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

**Bad**

The declared prop name user_name uses underscore-separated spelling.

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**Good**

The declaration and its template reference use the camelCase name userName.

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [All rules](all.md)

### `vue/require-component-is`

Require `v-bind:is` on `<component>` elements

[Bad](#vue-require-component-is-bad) · [Good](#vue-require-component-is-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The dynamic `<component>` has no `is` target, so Vue cannot choose a component to render.

```vue annotate="remove:2"
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**Good**

`:is="currentComponent"` supplies the component selection; the binding may change at runtime.

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [All rules](all.md)

### `vue/require-component-registration`

Require explicit import or registration for components

[Bad](#vue-require-component-registration-bad) · [Good](#vue-require-component-registration-good)

Default severity: `warning`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

List explicit component names supplied by application plugins or Musea previewSetup. PascalCase and kebab-case spellings are accepted; regular expressions are not interpreted. Options do not enable the rule. Later layers replace the list; an empty list clears inherited names.

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

**Bad**

`MissingWidget` is neither registered nor included in the configured global-component allowlist.

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**Good**

`MyButton` is listed in the example's `globals` option. That option exempts a known global component; it does not register or import it.

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [All rules](all.md)

### `vue/require-scoped-style`

Require scoped attribute on style tags

[Bad](#vue-require-scoped-style-bad) · [Good](#vue-require-scoped-style-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The `.button` style is unscoped and can affect matching elements outside this component.

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**Good**

Adding `scoped` applies Vue's component scope to the same selector and declarations.

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [All rules](all.md)

### `vue/require-toggle-inside-transition`

Require a toggle on the element wrapped by `<transition>`

[Bad](#vue-require-toggle-inside-transition-bad) · [Good](#vue-require-toggle-inside-transition-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The static child inside `<Transition>` has no conditional visibility or dynamic selection to trigger an enter/leave change.

```vue annotate="remove:3"
<template>
<transition>
<div>content</div>
</transition>
</template>
```

<span id="vue-require-toggle-inside-transition-good"></span>

**Good**

`v-if="show"` changes whether the child exists, giving the transition an enter/leave boundary.

```vue annotate="add:3"
<template>
<transition>
<div v-if="show">content</div>
</transition>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) · [All rules](all.md)

### `vue/require-v-for-key`

Require `v-bind:key` with `v-for` directives

[Bad](#vue-require-v-for-key-bad) · [Good](#vue-require-v-for-key-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

Each repeated `<li>` lacks a key that identifies its corresponding item during list updates.

```vue annotate="remove:2"
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**Good**

`:key="item.id"` gives each repeated node the item's identity rather than its current position.

```vue annotate="add:2"
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) · [All rules](all.md)

### `vue/scoped-event-names`

Recommend scoped event names using context:event format

[Bad](#vue-scoped-event-names-bad) · [Good](#vue-scoped-event-names-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

`playAudio`, `pauseAudio`, and `reloadAudio` encode their scope as camel-case suffixes rather than the rule's colon-separated event convention.

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

**Good**

`audio:play`, `audio:pause`, and `audio:reload` share an explicit `audio:` scope. The emitting component must use the same names.

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) · [All rules](all.md)

### `vue/sfc-element-order`

Enforce consistent order of SFC top-level elements

[Bad](#vue-sfc-element-order-bad) · [Good](#vue-sfc-element-order-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

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

**Bad**

The style block precedes the script block, contrary to the configured SFC block order.

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

**Good**

The blocks follow script → template → style. Projects can choose a different order through this rule's typed option.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) · [All rules](all.md)

### `vue/single-style-block`

Recommend having a single style block

[Bad](#vue-single-style-block-bad) · [Good](#vue-single-style-block-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The component splits its scoped panel and title styles across two style blocks.

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

**Good**

Both selectors stay scoped in one style block, satisfying the single-block convention without dropping either style.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) · [All rules](all.md)

### `vue/slot-name-casing`

Enforce kebab-case for named slots used via v-slot

[Bad](#vue-slot-name-casing-bad) · [Good](#vue-slot-name-casing-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The named slot `mySlot` uses camelCase where the rule requires a hyphenated name.

```vue annotate="remove:2"
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

<span id="vue-slot-name-casing-good"></span>

**Good**

`#my-slot` uses kebab-case. Rename the corresponding slot outlet to the same name.

```vue annotate="add:2"
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [All rules](all.md)

### `vue/this-in-template`

Disallow `this.` in template expressions

[Bad](#vue-this-in-template-bad) · [Good](#vue-this-in-template-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

Template expressions explicitly access `this.message`, `this.className`, and `this.handleClick`, although Vue exposes those bindings directly.

```vue annotate="remove:2,3,4"
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

<span id="vue-this-in-template-good"></span>

**Good**

Use `message`, `className`, and `handleClick` directly. The literal string `'this.is.a.string'` stays unchanged because it is not a member access.

```vue annotate="add:2,3,4,5"
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) · [All rules](all.md)

### `vue/use-unique-element-ids`

Enforce unique element IDs using useId() instead of static literals

[Bad](#vue-use-unique-element-ids-bad) · [Good](#vue-use-unique-element-ids-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The literal `email` ID is reused by every instance of this component, which can misdirect its label when several instances are rendered.

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**Good**

`useId()` produces the instance's `emailId`; bind the same value to the label's `for` and the input's `id`.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [All rules](all.md)

### `vue/use-v-on-exact`

Enforce `.exact` modifier on `v-on` when there are modifier-based handlers

[Bad](#vue-use-v-on-exact-bad) · [Good](#vue-use-v-on-exact-good)

Default severity: `warning`  
Presets: `essential`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The plain click handler can also run on Ctrl-click, overlapping the separate `.ctrl` handler.

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**Good**

`.exact` limits the ordinary click handler to clicks without modifier keys; the Ctrl-specific handler remains separate.

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

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) · [All rules](all.md)

### `vue/v-bind-style`

Enforce `v-bind` directive style

[Bad](#vue-v-bind-style-bad) · [Good](#vue-v-bind-style-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

`v-bind:class` uses the long form where the configured binding style requires the colon shorthand.

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**Good**

`:class` retains the same expression with the required shorthand; this rule concerns spelling rather than the value's type.

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [All rules](all.md)

### `vue/v-on-event-hyphenation`

Enforce hyphenation of custom event names in v-on on components

[Bad](#vue-v-on-event-hyphenation-bad) · [Good](#vue-v-on-event-hyphenation-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](options.md).

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

**Bad**

The custom component listener uses `@myEvent` instead of a hyphenated event name.

```vue annotate="remove:2,3"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
</template>
```

<span id="vue-v-on-event-hyphenation-good"></span>

**Good**

`@my-event` uses the required custom-event spelling. Native-element listeners and dynamic event arguments shown below are outside this check.

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) · [All rules](all.md)

### `vue/v-on-handler-style`

Enforce writing v-on handlers as a method reference or an inline function

[Bad](#vue-v-on-handler-style-bad) · [Good](#vue-v-on-handler-style-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The handlers put mutations and multiple statements directly in the event attribute.

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

<span id="vue-v-on-handler-style-good"></span>

**Good**

Use a handler reference, or an arrow/function expression when inline logic is needed. The function boundary makes the handler form explicit.

```vue annotate="add:2,3,4,5"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) · [All rules](all.md)

### `vue/v-on-style`

Enforce `v-on` directive style

[Bad](#vue-v-on-style-bad) · [Good](#vue-v-on-style-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

`v-on:click` uses the long event-listener form where the rule requires shorthand.

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**Good**

`@click` keeps the same handler while using the configured shorthand.

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [All rules](all.md)

### `vue/v-slot-style`

Enforce `v-slot` directive style

[Bad](#vue-v-slot-style-bad) · [Good](#vue-v-slot-style-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The component uses `#default` and the template uses `v-slot:header`, opposite to the rule's context-specific styles.

```vue annotate="remove:2,4"
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

<span id="vue-v-slot-style-good"></span>

**Good**

Use `v-slot` for the component's default slot and `#header` for the template's named slot.

```vue annotate="add:2,4"
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [All rules](all.md)

### `vue/valid-attribute-name`

Require valid attribute names

[Bad](#vue-valid-attribute-name-bad) · [Good](#vue-valid-attribute-name-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Bad diagnostic: `parser/template`

Malformed attribute spelling is diagnosed by parser/template before this defensive rule sees an attribute. Bad therefore reports parser/template; it does not promise a separate vue/valid-attribute-name finding.

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

**Bad**

The quote inside `my"attr` makes the attribute name malformed. This example produces the parser's `parser/template` diagnostic rather than promising a separate rule diagnostic.

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**Good**

`my-attr` is a well-formed attribute name, so the template parser can read the attribute and its value.

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [All rules](all.md)

### `vue/valid-template-root`

Enforce a valid `<template>` root for Vue 3 fragment semantics

[Bad](#vue-valid-template-root-bad) · [Good](#vue-valid-template-root-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

A plain nested `<template>` occupies the template root without a directive that gives it a rendering role.

```vue annotate="remove:2"
<template>
<template>content</template>
</template>
```

<span id="vue-valid-template-root-good"></span>

**Good**

The `<div>` is a renderable root element. This example does not impose a universal single-root restriction on Vue 3 fragments.

```vue annotate="add:2"
<template>
<div>content</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [All rules](all.md)

### `vue/valid-v-bind`

Enforce valid `v-bind` directives

[Bad](#vue-valid-v-bind-bad) · [Good](#vue-valid-v-bind-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The bare `v-bind` has no object expression, and the empty argument form has no attribute name.

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

<span id="vue-valid-v-bind-good"></span>

**Good**

Provide an attribute and expression, bind an object, or use Vue 3.4+ same-name shorthand such as `:loading`.

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) · [All rules](all.md)

### `vue/valid-v-cloak`

Enforce valid `v-cloak` directives

[Bad](#vue-valid-v-cloak-bad) · [Good](#vue-valid-v-cloak-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

`v-cloak` is given a value, argument, or modifier even though it accepts none of those.

```vue annotate="remove:2,3,4"
<template>
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

<span id="vue-valid-v-cloak-good"></span>

**Good**

Use bare `v-cloak`; CSS can hide the element until Vue removes that attribute after mounting.

```vue annotate="add:2"
<template>
<div v-cloak></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) · [All rules](all.md)

### `vue/valid-v-else`

Enforce valid `v-else` directives

[Bad](#vue-valid-v-else-bad) · [Good](#vue-valid-v-else-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The examples give `v-else` an expression, combine it with `v-if`, or omit its adjacent preceding conditional branch.

```vue annotate="remove:2,3"
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

<span id="vue-valid-v-else-good"></span>

**Good**

Place bare `v-else` immediately after the corresponding `v-if` branch.

```vue annotate="add:2"
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) · [All rules](all.md)

### `vue/valid-v-for`

Enforce valid `v-for` directives

[Bad](#vue-valid-v-for-bad) · [Good](#vue-valid-v-for-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The loops omit their iteration expression or add an unsupported `.stop` modifier.

```vue annotate="remove:2,3,4"
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

<span id="vue-valid-v-for-good"></span>

**Good**

Use `item in items` or `(item, index) of items` with a complete iteration expression and the shown keys.

```vue annotate="add:2,3"
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) · [All rules](all.md)

### `vue/valid-v-html`

Enforce valid `v-html` directives

[Bad](#vue-valid-v-html-bad) · [Good](#vue-valid-v-html-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

`v-html` lacks its expression or uses an argument/modifier that this directive does not support.

```vue annotate="remove:2,3,4"
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

<span id="vue-valid-v-html-good"></span>

**Good**

`v-html="html"` supplies a valid expression. Syntax validity does not sanitize HTML or make untrusted content safe.

```vue annotate="add:2"
<template>
<div v-html="html"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) · [All rules](all.md)

### `vue/valid-v-if`

Enforce valid `v-if` directives

[Bad](#vue-valid-v-if-bad) · [Good](#vue-valid-v-if-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The conditions omit an expression or combine `v-if` with an else directive on the same node.

```vue annotate="remove:2,3,4"
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

<span id="vue-valid-v-if-good"></span>

**Good**

Each `v-if` has a nonempty condition such as `ready` or `count > 0`, without an incompatible else directive.

```vue annotate="add:2,3"
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) · [All rules](all.md)

### `vue/valid-v-memo`

Enforce valid `v-memo` directives

[Bad](#vue-valid-v-memo-bad) · [Good](#vue-valid-v-memo-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

Bare `v-memo` gives Vue no dependency expression for deciding when to reuse the subtree.

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

<span id="vue-valid-v-memo-good"></span>

**Good**

`v-memo="[valueA, valueB]"` supplies the dependency array used for memoization.

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [All rules](all.md)

### `vue/valid-v-model`

Enforce valid `v-model` directives

[Bad](#vue-valid-v-model-bad) · [Good](#vue-valid-v-model-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

A native `<div>` cannot use `v-model` as a form control, and a bare input directive has no writable target expression.

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

<span id="vue-valid-v-model-good"></span>

**Good**

Bind the input, select, textarea, or custom component to the shown writable variables.

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) · [All rules](all.md)

### `vue/valid-v-on`

Enforce valid `v-on` directives

[Bad](#vue-valid-v-on-bad) · [Good](#vue-valid-v-on-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The listener forms omit an event argument or their required handler/object expression.

```vue annotate="remove:2,3,4"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

<span id="vue-valid-v-on-good"></span>

**Good**

Use an event with its handler, or pass a listener object to argument-free `v-on`.

```vue annotate="add:2,3"
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [All rules](all.md)

### `vue/valid-v-once`

Enforce valid `v-once` directives

[Bad](#vue-valid-v-once-bad) · [Good](#vue-valid-v-once-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

`v-once` has a value, argument, or modifier, although this directive is a value-free render-once marker.

```vue annotate="remove:2,3,4"
<template>
<div v-once="foo"></div>
<div v-once:arg></div>
<div v-once.mod></div>
</template>
```

<span id="vue-valid-v-once-good"></span>

**Good**

Bare `v-once` marks the subtree for one-time rendering without unsupported syntax.

```vue annotate="add:2"
<template>
<div v-once></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) · [All rules](all.md)

### `vue/valid-v-show`

Enforce valid `v-show` directives

[Bad](#vue-valid-v-show-bad) · [Good](#vue-valid-v-show-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

`v-show` lacks its visibility expression or is placed on a `<template>` that has no DOM element whose display can be changed.

```vue annotate="remove:2,3"
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

<span id="vue-valid-v-show-good"></span>

**Good**

Apply the visibility expression to a rendered element such as `<div>`.

```vue annotate="add:2,3"
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [All rules](all.md)

### `vue/valid-v-slot`

Enforce valid `v-slot` directives

[Bad](#vue-valid-v-slot-bad) · [Good](#vue-valid-v-slot-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The slot directive is on a native `<div>` or conflicts with other default/named slot declarations.

```vue annotate="remove:2,3,4"
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

<span id="vue-valid-v-slot-good"></span>

**Good**

Declare a component's default slot on that component, or its named slot on a child `<template #header>`.

```vue annotate="add:2,3,4,5"
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) · [All rules](all.md)

### `vue/valid-v-text`

Enforce valid `v-text` directives

[Bad](#vue-valid-v-text-bad) · [Good](#vue-valid-v-text-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

`v-text` lacks its text expression or uses an unsupported argument/modifier.

```vue annotate="remove:2,3,4"
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

<span id="vue-valid-v-text-good"></span>

**Good**

`v-text="msg"` is syntactically valid. The separate `vue/no-v-text` style rule can still prefer interpolation.

```vue annotate="add:2"
<template>
<div v-text="msg"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [All rules](all.md)

### `vue/warn-custom-block`

Warn about custom blocks in SFC files

[Bad](#vue-warn-custom-block-bad) · [Good](#vue-warn-custom-block-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

The SFC contains an `<i18n>` custom block, which needs an external integration beyond ordinary template/script/style processing.

```vue annotate="remove:1,2,3,4"
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

<span id="vue-warn-custom-block-good"></span>

**Good**

The example uses standard template and script-setup blocks. This optional portability warning does not mean every custom block is invalid Vue.

```vue annotate="add:4,5,6,7"
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [All rules](all.md)

### `vue/warn-custom-directive`

Warn about custom directives that need registration

[Bad](#vue-warn-custom-directive-bad) · [Good](#vue-warn-custom-directive-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

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

**Bad**

`v-focus`, `v-mask`, and `v-click-outside` require project-specific directive implementations that this optional convention flags.

```vue annotate="remove:2,3,4"
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**Good**

The example uses built-in `v-if`, `v-model`, and `v-on`. A correctly registered custom directive can still be valid Vue when this policy is disabled.

```vue annotate="add:2,3,4"
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [All rules](all.md)
