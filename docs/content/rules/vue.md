---
title: Vue rules
---

# Vue rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. The complete catalogue keeps all examples and current support boundaries on one page.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/a11y-img-alt`](./all.md#vue-a11y-img-alt) | [Bad](./all.md#vue-a11y-img-alt-bad) · [Good](./all.md#vue-a11y-img-alt-good) | Require alt attribute on images for accessibility |
| [`vue/attribute-hyphenation`](./all.md#vue-attribute-hyphenation) | [Bad](./all.md#vue-attribute-hyphenation-bad) · [Good](./all.md#vue-attribute-hyphenation-good) | Enforce attribute naming style on custom components |
| [`vue/attribute-order`](./all.md#vue-attribute-order) | [Bad](./all.md#vue-attribute-order-bad) · [Good](./all.md#vue-attribute-order-good) | Enforce a consistent order of attributes |
| [`vue/component-definition-name-casing`](./all.md#vue-component-definition-name-casing) | [Bad](./all.md#vue-component-definition-name-casing-bad) · [Good](./all.md#vue-component-definition-name-casing-good) | Enforce PascalCase or kebab-case for component definition names |
| [`vue/component-name-in-template-casing`](./all.md#vue-component-name-in-template-casing) | [Bad](./all.md#vue-component-name-in-template-casing-bad) · [Good](./all.md#vue-component-name-in-template-casing-good) | Enforce specific casing for component names in templates |
| [`vue/html-button-has-type`](./all.md#vue-html-button-has-type) | [Bad](./all.md#vue-html-button-has-type-bad) · [Good](./all.md#vue-html-button-has-type-good) | Require an explicit valid type on button elements |
| [`vue/html-quotes`](./all.md#vue-html-quotes) | [Bad](./all.md#vue-html-quotes-bad) · [Good](./all.md#vue-html-quotes-good) | Enforce quotes style of HTML attributes |
| [`vue/html-self-closing`](./all.md#vue-html-self-closing) | [Bad](./all.md#vue-html-self-closing-bad) · [Good](./all.md#vue-html-self-closing-good) | Enforce self-closing style |
| [`vue/max-template-complexity`](./all.md#vue-max-template-complexity) | [Bad](./all.md#vue-max-template-complexity-bad) · [Good](./all.md#vue-max-template-complexity-good) | Limit a component's own template complexity (cyclomatic and cognitive) |
| [`vue/multi-word-component-names`](./all.md#vue-multi-word-component-names) | [Bad](./all.md#vue-multi-word-component-names-bad) · [Good](./all.md#vue-multi-word-component-names-good) | Require component names to be multi-word |
| [`vue/mustache-interpolation-spacing`](./all.md#vue-mustache-interpolation-spacing) | [Bad](./all.md#vue-mustache-interpolation-spacing-bad) · [Good](./all.md#vue-mustache-interpolation-spacing-good) | Enforce consistent spacing inside mustache interpolations |
| [`vue/no-array-index-key`](./all.md#vue-no-array-index-key) | [Bad](./all.md#vue-no-array-index-key-bad) · [Good](./all.md#vue-no-array-index-key-good) | Disallow using the v-for index variable directly as the :key |
| [`vue/no-bare-strings-in-template`](./all.md#vue-no-bare-strings-in-template) | [Bad](./all.md#vue-no-bare-strings-in-template-bad) · [Good](./all.md#vue-no-bare-strings-in-template-good) | Disallow raw human-readable text in the template that should be internationalized |
| [`vue/no-boolean-attr-value`](./all.md#vue-no-boolean-attr-value) | [Bad](./all.md#vue-no-boolean-attr-value-bad) · [Good](./all.md#vue-no-boolean-attr-value-good) | Disallow explicit values for boolean HTML attributes |
| [`vue/no-child-content`](./all.md#vue-no-child-content) | [Bad](./all.md#vue-no-child-content-bad) · [Good](./all.md#vue-no-child-content-good) | Disallow child content when using v-html or v-text |
| [`vue/no-deprecated-filter`](./all.md#vue-no-deprecated-filter) | [Bad](./all.md#vue-no-deprecated-filter-bad) · [Good](./all.md#vue-no-deprecated-filter-good) | Disallow deprecated Vue 2 filter syntax using the pipe operator |
| [`vue/no-deprecated-functional-template`](./all.md#vue-no-deprecated-functional-template) | [Bad](./all.md#vue-no-deprecated-functional-template-bad) · [Good](./all.md#vue-no-deprecated-functional-template-good) | Disallow the `functional` attribute on the SFC `&lt;template&gt;` |
| [`vue/no-deprecated-html-element-is`](./all.md#vue-no-deprecated-html-element-is) | [Bad](./all.md#vue-no-deprecated-html-element-is-bad) · [Good](./all.md#vue-no-deprecated-html-element-is-good) | Disallow the `is` attribute on native HTML elements |
| [`vue/no-deprecated-inline-template`](./all.md#vue-no-deprecated-inline-template) | [Bad](./all.md#vue-no-deprecated-inline-template-bad) · [Good](./all.md#vue-no-deprecated-inline-template-good) | Disallow the deprecated `inline-template` attribute |
| [`vue/no-deprecated-router-link-tag-prop`](./all.md#vue-no-deprecated-router-link-tag-prop) | [Bad](./all.md#vue-no-deprecated-router-link-tag-prop-bad) · [Good](./all.md#vue-no-deprecated-router-link-tag-prop-good) | Disallow the `tag` prop on &lt;router-link&gt; |
| [`vue/no-deprecated-scope-attribute`](./all.md#vue-no-deprecated-scope-attribute) | [Bad](./all.md#vue-no-deprecated-scope-attribute-bad) · [Good](./all.md#vue-no-deprecated-scope-attribute-good) | Disallow the deprecated `scope` attribute on &lt;template&gt; |
| [`vue/no-deprecated-slot-attribute`](./all.md#vue-no-deprecated-slot-attribute) | [Bad](./all.md#vue-no-deprecated-slot-attribute-bad) · [Good](./all.md#vue-no-deprecated-slot-attribute-good) | Disallow the deprecated `slot` attribute |
| [`vue/no-deprecated-slot-scope-attribute`](./all.md#vue-no-deprecated-slot-scope-attribute) | [Bad](./all.md#vue-no-deprecated-slot-scope-attribute-bad) · [Good](./all.md#vue-no-deprecated-slot-scope-attribute-good) | Disallow the deprecated `slot-scope` attribute |
| [`vue/no-deprecated-v-bind-sync`](./all.md#vue-no-deprecated-v-bind-sync) | [Bad](./all.md#vue-no-deprecated-v-bind-sync-bad) · [Good](./all.md#vue-no-deprecated-v-bind-sync-good) | Disallow the deprecated `.sync` modifier on `v-bind` |
| [`vue/no-deprecated-v-on-native-modifier`](./all.md#vue-no-deprecated-v-on-native-modifier) | [Bad](./all.md#vue-no-deprecated-v-on-native-modifier-bad) · [Good](./all.md#vue-no-deprecated-v-on-native-modifier-good) | Disallow the deprecated `.native` modifier on `v-on` |
| [`vue/no-deprecated-v-on-number-modifiers`](./all.md#vue-no-deprecated-v-on-number-modifiers) | [Bad](./all.md#vue-no-deprecated-v-on-number-modifiers-bad) · [Good](./all.md#vue-no-deprecated-v-on-number-modifiers-good) | Disallow deprecated numeric `keyCode` modifiers on `v-on` |
| [`vue/no-dupe-v-else-if`](./all.md#vue-no-dupe-v-else-if) | [Bad](./all.md#vue-no-dupe-v-else-if-bad) · [Good](./all.md#vue-no-dupe-v-else-if-good) | Disallow duplicate conditions in `v-if` / `v-else-if` chains |
| [`vue/no-duplicate-attributes`](./all.md#vue-no-duplicate-attributes) | [Bad](./all.md#vue-no-duplicate-attributes-bad) · [Good](./all.md#vue-no-duplicate-attributes-good) | Disallow duplicate attributes on the same element |
| [`vue/no-empty-component-block`](./all.md#vue-no-empty-component-block) | [Bad](./all.md#vue-no-empty-component-block-bad) · [Good](./all.md#vue-no-empty-component-block-good) | Disallow empty SFC blocks |
| [`vue/no-inline-style`](./all.md#vue-no-inline-style) | [Bad](./all.md#vue-no-inline-style-bad) · [Good](./all.md#vue-no-inline-style-good) | Discourage use of inline style attributes |
| [`vue/no-invalid-html-attribute`](./all.md#vue-no-invalid-html-attribute) | [Bad](./all.md#vue-no-invalid-html-attribute-bad) · [Good](./all.md#vue-no-invalid-html-attribute-good) | Disallow invalid static values for HTML attributes |
| [`vue/no-lone-template`](./all.md#vue-no-lone-template) | [Bad](./all.md#vue-no-lone-template-bad) · [Good](./all.md#vue-no-lone-template-good) | Disallow unnecessary `&lt;template&gt;` elements |
| [`vue/no-multi-spaces`](./all.md#vue-no-multi-spaces) | [Bad](./all.md#vue-no-multi-spaces-bad) · [Good](./all.md#vue-no-multi-spaces-good) | Disallow multiple consecutive spaces |
| [`vue/no-multiple-objects-in-class`](./all.md#vue-no-multiple-objects-in-class) | [Bad](./all.md#vue-no-multiple-objects-in-class-bad) · [Good](./all.md#vue-no-multiple-objects-in-class-good) | Disallow multiple object literals inside a :class array binding |
| [`vue/no-multiple-template-root`](./all.md#vue-no-multiple-template-root) | [Bad](./all.md#vue-no-multiple-template-root-bad) · [Good](./all.md#vue-no-multiple-template-root-good) | Disallow multiple root nodes in a template |
| [`vue/no-mutating-props`](./all.md#vue-no-mutating-props) | [Bad](./all.md#vue-no-mutating-props-bad) · [Good](./all.md#vue-no-mutating-props-good) | Disallow mutating component props |
| [`vue/no-negated-v-if-condition`](./all.md#vue-no-negated-v-if-condition) | [Bad](./all.md#vue-no-negated-v-if-condition-bad) · [Good](./all.md#vue-no-negated-v-if-condition-good) | Disallow a negated v-if condition when the chain has a v-else |
| [`vue/no-non-component-keep-alive-child`](./all.md#vue-no-non-component-keep-alive-child) | [Bad](./all.md#vue-no-non-component-keep-alive-child-bad) · [Good](./all.md#vue-no-non-component-keep-alive-child-good) | Disallow plain element wrappers directly below `&lt;KeepAlive&gt;` |
| [`vue/no-preprocessor-lang`](./all.md#vue-no-preprocessor-lang) | [Bad](./all.md#vue-no-preprocessor-lang-bad) · [Good](./all.md#vue-no-preprocessor-lang-good) | Discourage CSS preprocessor usage in favor of modern CSS |
| [`vue/no-reserved-component-names`](./all.md#vue-no-reserved-component-names) | [Bad](./all.md#vue-no-reserved-component-names-bad) · [Good](./all.md#vue-no-reserved-component-names-good) | Disallow the use of reserved names as component names |
| [`vue/no-root-v-if`](./all.md#vue-no-root-v-if) | [Bad](./all.md#vue-no-root-v-if-bad) · [Good](./all.md#vue-no-root-v-if-good) | Disallow v-if on the single root element of a template |
| [`vue/no-script-non-standard-lang`](./all.md#vue-no-script-non-standard-lang) | [Bad](./all.md#vue-no-script-non-standard-lang-bad) · [Good](./all.md#vue-no-script-non-standard-lang-good) | Discourage non-standard script lang values |
| [`vue/no-src-attribute`](./all.md#vue-no-src-attribute) | [Bad](./all.md#vue-no-src-attribute-bad) · [Good](./all.md#vue-no-src-attribute-good) | Discourage src attribute on SFC blocks |
| [`vue/no-static-inline-styles`](./all.md#vue-no-static-inline-styles) | [Bad](./all.md#vue-no-static-inline-styles-bad) · [Good](./all.md#vue-no-static-inline-styles-good) | Disallow static inline style attributes |
| [`vue/no-template-key`](./all.md#vue-no-template-key) | [Bad](./all.md#vue-no-template-key-bad) · [Good](./all.md#vue-no-template-key-good) | Disallow `key` attribute on `&lt;template&gt;` |
| [`vue/no-template-lang`](./all.md#vue-no-template-lang) | [Bad](./all.md#vue-no-template-lang-bad) · [Good](./all.md#vue-no-template-lang-good) | Discourage lang attribute on template block |
| [`vue/no-template-shadow`](./all.md#vue-no-template-shadow) | [Bad](./all.md#vue-no-template-shadow-bad) · [Good](./all.md#vue-no-template-shadow-good) | Disallow variable names that shadow variables in outer scope |
| [`vue/no-template-target-blank`](./all.md#vue-no-template-target-blank) | [Bad](./all.md#vue-no-template-target-blank-bad) · [Good](./all.md#vue-no-template-target-blank-good) | Disallow target="_blank" without rel="noopener noreferrer" |
| [`vue/no-textarea-mustache`](./all.md#vue-no-textarea-mustache) | [Bad](./all.md#vue-no-textarea-mustache-bad) · [Good](./all.md#vue-no-textarea-mustache-good) | Disallow mustache interpolation in `&lt;textarea&gt;` |
| [`vue/no-undefined-refs`](./all.md#vue-no-undefined-refs) | [Bad](./all.md#vue-no-undefined-refs-bad) · [Good](./all.md#vue-no-undefined-refs-good) | Disallow undefined variable references in templates |
| [`vue/no-unsafe-url`](./all.md#vue-no-unsafe-url) | [Bad](./all.md#vue-no-unsafe-url-bad) · [Good](./all.md#vue-no-unsafe-url-good) | Warn about potentially unsafe URL bindings |
| [`vue/no-unsandboxed-iframe`](./all.md#vue-no-unsandboxed-iframe) | [Bad](./all.md#vue-no-unsandboxed-iframe-bad) · [Good](./all.md#vue-no-unsandboxed-iframe-good) | Require a sandbox attribute on iframe elements |
| [`vue/no-unused-components`](./all.md#vue-no-unused-components) | [Bad](./all.md#vue-no-unused-components-bad) · [Good](./all.md#vue-no-unused-components-good) | Disallow registering components that are not used inside templates |
| [`vue/no-unused-properties`](./all.md#vue-no-unused-properties) | [Bad](./all.md#vue-no-unused-properties-bad) · [Good](./all.md#vue-no-unused-properties-good) | Disallow unused properties defined in defineProps |
| [`vue/no-unused-refs`](./all.md#vue-no-unused-refs) | [Bad](./all.md#vue-no-unused-refs-bad) · [Good](./all.md#vue-no-unused-refs-good) | Report template refs (ref="x") never referenced in &lt;script&gt; |
| [`vue/no-unused-setup-bindings`](./all.md#vue-no-unused-setup-bindings) | [Bad](./all.md#vue-no-unused-setup-bindings-bad) · [Good](./all.md#vue-no-unused-setup-bindings-good) | Disallow unread script setup bindings |
| [`vue/no-unused-vars`](./all.md#vue-no-unused-vars) | [Bad](./all.md#vue-no-unused-vars-bad) · [Good](./all.md#vue-no-unused-vars-good) | Disallow unused variable definitions in v-for and v-slot directives |
| [`vue/no-use-v-else-with-v-for`](./all.md#vue-no-use-v-else-with-v-for) | [Bad](./all.md#vue-no-use-v-else-with-v-for-bad) · [Good](./all.md#vue-no-use-v-else-with-v-for-good) | Disallow using `v-else-if` or `v-else` on the same element as `v-for` |
| [`vue/no-use-v-if-with-v-for`](./all.md#vue-no-use-v-if-with-v-for) | [Bad](./all.md#vue-no-use-v-if-with-v-for-bad) · [Good](./all.md#vue-no-use-v-if-with-v-for-good) | Disallow using `v-if` on the same element as `v-for` |
| [`vue/no-useless-mustaches`](./all.md#vue-no-useless-mustaches) | [Bad](./all.md#vue-no-useless-mustaches-bad) · [Good](./all.md#vue-no-useless-mustaches-good) | Disallow a mustache interpolation whose expression is a constant string literal |
| [`vue/no-useless-template-attributes`](./all.md#vue-no-useless-template-attributes) | [Bad](./all.md#vue-no-useless-template-attributes-bad) · [Good](./all.md#vue-no-useless-template-attributes-good) | Disallow useless attributes on `&lt;template&gt;` elements |
| [`vue/no-useless-v-bind`](./all.md#vue-no-useless-v-bind) | [Bad](./all.md#vue-no-useless-v-bind-bad) · [Good](./all.md#vue-no-useless-v-bind-good) | Disallow a v-bind whose value is a plain string literal |
| [`vue/no-v-for-template-key-on-child`](./all.md#vue-no-v-for-template-key-on-child) | [Bad](./all.md#vue-no-v-for-template-key-on-child-bad) · [Good](./all.md#vue-no-v-for-template-key-on-child-good) | Disallow `key` on the child of a `&lt;template v-for&gt;` |
| [`vue/no-v-html`](./all.md#vue-no-v-html) | [Bad](./all.md#vue-no-v-html-bad) · [Good](./all.md#vue-no-v-html-good) | Warn against v-html to prevent XSS vulnerabilities |
| [`vue/no-v-text`](./all.md#vue-no-v-text) | [Bad](./all.md#vue-no-v-text-bad) · [Good](./all.md#vue-no-v-text-good) | Disallow the v-text directive; prefer mustache interpolation |
| [`vue/no-v-text-v-html-on-component`](./all.md#vue-no-v-text-v-html-on-component) | [Bad](./all.md#vue-no-v-text-v-html-on-component-bad) · [Good](./all.md#vue-no-v-text-v-html-on-component-good) | Disallow v-text / v-html on component elements |
| [`vue/permitted-contents`](./all.md#vue-permitted-contents) | [Bad](./all.md#vue-permitted-contents-bad) · [Good](./all.md#vue-permitted-contents-good) | Enforce HTML content model rules |
| [`vue/prefer-props-shorthand`](./all.md#vue-prefer-props-shorthand) | [Bad](./all.md#vue-prefer-props-shorthand-bad) · [Good](./all.md#vue-prefer-props-shorthand-good) | Recommend shorthand syntax for props (Vue 3.4+) |
| [`vue/prefer-true-attribute-shorthand`](./all.md#vue-prefer-true-attribute-shorthand) | [Bad](./all.md#vue-prefer-true-attribute-shorthand-bad) · [Good](./all.md#vue-prefer-true-attribute-shorthand-good) | Prefer the shorthand for a boolean attribute bound to `true` |
| [`vue/prop-name-casing`](./all.md#vue-prop-name-casing) | [Bad](./all.md#vue-prop-name-casing-bad) · [Good](./all.md#vue-prop-name-casing-good) | Enforce a casing for declared prop names |
| [`vue/require-component-is`](./all.md#vue-require-component-is) | [Bad](./all.md#vue-require-component-is-bad) · [Good](./all.md#vue-require-component-is-good) | Require `v-bind:is` on `&lt;component&gt;` elements |
| [`vue/require-component-registration`](./all.md#vue-require-component-registration) | [Bad](./all.md#vue-require-component-registration-bad) · [Good](./all.md#vue-require-component-registration-good) | Require explicit import or registration for components |
| [`vue/require-scoped-style`](./all.md#vue-require-scoped-style) | [Bad](./all.md#vue-require-scoped-style-bad) · [Good](./all.md#vue-require-scoped-style-good) | Require scoped attribute on style tags |
| [`vue/require-toggle-inside-transition`](./all.md#vue-require-toggle-inside-transition) | [Bad](./all.md#vue-require-toggle-inside-transition-bad) · [Good](./all.md#vue-require-toggle-inside-transition-good) | Require a toggle on the element wrapped by `&lt;transition&gt;` |
| [`vue/require-v-for-key`](./all.md#vue-require-v-for-key) | [Bad](./all.md#vue-require-v-for-key-bad) · [Good](./all.md#vue-require-v-for-key-good) | Require `v-bind:key` with `v-for` directives |
| [`vue/scoped-event-names`](./all.md#vue-scoped-event-names) | [Bad](./all.md#vue-scoped-event-names-bad) · [Good](./all.md#vue-scoped-event-names-good) | Recommend scoped event names using context:event format |
| [`vue/sfc-element-order`](./all.md#vue-sfc-element-order) | [Bad](./all.md#vue-sfc-element-order-bad) · [Good](./all.md#vue-sfc-element-order-good) | Enforce consistent order of SFC top-level elements |
| [`vue/single-style-block`](./all.md#vue-single-style-block) | [Bad](./all.md#vue-single-style-block-bad) · [Good](./all.md#vue-single-style-block-good) | Recommend having a single style block |
| [`vue/slot-name-casing`](./all.md#vue-slot-name-casing) | [Bad](./all.md#vue-slot-name-casing-bad) · [Good](./all.md#vue-slot-name-casing-good) | Enforce kebab-case for named slots used via v-slot |
| [`vue/this-in-template`](./all.md#vue-this-in-template) | [Bad](./all.md#vue-this-in-template-bad) · [Good](./all.md#vue-this-in-template-good) | Disallow `this.` in template expressions |
| [`vue/use-unique-element-ids`](./all.md#vue-use-unique-element-ids) | [Bad](./all.md#vue-use-unique-element-ids-bad) · [Good](./all.md#vue-use-unique-element-ids-good) | Enforce unique element IDs using useId() instead of static literals |
| [`vue/use-v-on-exact`](./all.md#vue-use-v-on-exact) | [Bad](./all.md#vue-use-v-on-exact-bad) · [Good](./all.md#vue-use-v-on-exact-good) | Enforce `.exact` modifier on `v-on` when there are modifier-based handlers |
| [`vue/v-bind-style`](./all.md#vue-v-bind-style) | [Bad](./all.md#vue-v-bind-style-bad) · [Good](./all.md#vue-v-bind-style-good) | Enforce `v-bind` directive style |
| [`vue/v-on-event-hyphenation`](./all.md#vue-v-on-event-hyphenation) | [Bad](./all.md#vue-v-on-event-hyphenation-bad) · [Good](./all.md#vue-v-on-event-hyphenation-good) | Enforce hyphenation of custom event names in v-on on components |
| [`vue/v-on-handler-style`](./all.md#vue-v-on-handler-style) | [Bad](./all.md#vue-v-on-handler-style-bad) · [Good](./all.md#vue-v-on-handler-style-good) | Enforce writing v-on handlers as a method reference or an inline function |
| [`vue/v-on-style`](./all.md#vue-v-on-style) | [Bad](./all.md#vue-v-on-style-bad) · [Good](./all.md#vue-v-on-style-good) | Enforce `v-on` directive style |
| [`vue/v-slot-style`](./all.md#vue-v-slot-style) | [Bad](./all.md#vue-v-slot-style-bad) · [Good](./all.md#vue-v-slot-style-good) | Enforce `v-slot` directive style |
| [`vue/valid-attribute-name`](./all.md#vue-valid-attribute-name) | [Bad](./all.md#vue-valid-attribute-name-bad) · [Good](./all.md#vue-valid-attribute-name-good) | Require valid attribute names |
| [`vue/valid-template-root`](./all.md#vue-valid-template-root) | [Bad](./all.md#vue-valid-template-root-bad) · [Good](./all.md#vue-valid-template-root-good) | Enforce a valid `&lt;template&gt;` root for Vue 3 fragment semantics |
| [`vue/valid-v-bind`](./all.md#vue-valid-v-bind) | [Bad](./all.md#vue-valid-v-bind-bad) · [Good](./all.md#vue-valid-v-bind-good) | Enforce valid `v-bind` directives |
| [`vue/valid-v-cloak`](./all.md#vue-valid-v-cloak) | [Bad](./all.md#vue-valid-v-cloak-bad) · [Good](./all.md#vue-valid-v-cloak-good) | Enforce valid `v-cloak` directives |
| [`vue/valid-v-else`](./all.md#vue-valid-v-else) | [Bad](./all.md#vue-valid-v-else-bad) · [Good](./all.md#vue-valid-v-else-good) | Enforce valid `v-else` directives |
| [`vue/valid-v-for`](./all.md#vue-valid-v-for) | [Bad](./all.md#vue-valid-v-for-bad) · [Good](./all.md#vue-valid-v-for-good) | Enforce valid `v-for` directives |
| [`vue/valid-v-html`](./all.md#vue-valid-v-html) | [Bad](./all.md#vue-valid-v-html-bad) · [Good](./all.md#vue-valid-v-html-good) | Enforce valid `v-html` directives |
| [`vue/valid-v-if`](./all.md#vue-valid-v-if) | [Bad](./all.md#vue-valid-v-if-bad) · [Good](./all.md#vue-valid-v-if-good) | Enforce valid `v-if` directives |
| [`vue/valid-v-memo`](./all.md#vue-valid-v-memo) | [Bad](./all.md#vue-valid-v-memo-bad) · [Good](./all.md#vue-valid-v-memo-good) | Enforce valid `v-memo` directives |
| [`vue/valid-v-model`](./all.md#vue-valid-v-model) | [Bad](./all.md#vue-valid-v-model-bad) · [Good](./all.md#vue-valid-v-model-good) | Enforce valid `v-model` directives |
| [`vue/valid-v-on`](./all.md#vue-valid-v-on) | [Bad](./all.md#vue-valid-v-on-bad) · [Good](./all.md#vue-valid-v-on-good) | Enforce valid `v-on` directives |
| [`vue/valid-v-once`](./all.md#vue-valid-v-once) | [Bad](./all.md#vue-valid-v-once-bad) · [Good](./all.md#vue-valid-v-once-good) | Enforce valid `v-once` directives |
| [`vue/valid-v-show`](./all.md#vue-valid-v-show) | [Bad](./all.md#vue-valid-v-show-bad) · [Good](./all.md#vue-valid-v-show-good) | Enforce valid `v-show` directives |
| [`vue/valid-v-slot`](./all.md#vue-valid-v-slot) | [Bad](./all.md#vue-valid-v-slot-bad) · [Good](./all.md#vue-valid-v-slot-good) | Enforce valid `v-slot` directives |
| [`vue/valid-v-text`](./all.md#vue-valid-v-text) | [Bad](./all.md#vue-valid-v-text-bad) · [Good](./all.md#vue-valid-v-text-good) | Enforce valid `v-text` directives |
| [`vue/warn-custom-block`](./all.md#vue-warn-custom-block) | [Bad](./all.md#vue-warn-custom-block-bad) · [Good](./all.md#vue-warn-custom-block-good) | Warn about custom blocks in SFC files |
| [`vue/warn-custom-directive`](./all.md#vue-warn-custom-directive) | [Bad](./all.md#vue-warn-custom-directive-bad) · [Good](./all.md#vue-warn-custom-directive-good) | Warn about custom directives that need registration |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)

For attributes across imported components, see [vue/cross-file-attrs-fallthrough](./project/vue-cross-file-attrs-fallthrough.md).
