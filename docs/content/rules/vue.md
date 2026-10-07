---
title: Vue rules
---

# Vue rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Purpose |
| --- | --- |
| [`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | Require alt attribute on images for accessibility |
| [`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | Enforce attribute naming style on custom components |
| [`vue/attribute-order`](./reference/vue-attribute-order.md) | Enforce a consistent order of attributes |
| [`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | Enforce PascalCase or kebab-case for component definition names |
| [`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | Enforce specific casing for component names in templates |
| [`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | Require an explicit valid type on button elements |
| [`vue/html-quotes`](./reference/vue-html-quotes.md) | Enforce quotes style of HTML attributes |
| [`vue/html-self-closing`](./reference/vue-html-self-closing.md) | Enforce self-closing style |
| [`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | Limit a component's own template complexity (cyclomatic and cognitive) |
| [`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | Require component names to be multi-word |
| [`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | Enforce consistent spacing inside mustache interpolations |
| [`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | Disallow using the v-for index variable directly as the :key |
| [`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | Disallow raw human-readable text in the template that should be internationalized |
| [`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | Disallow explicit values for boolean HTML attributes |
| [`vue/no-child-content`](./reference/vue-no-child-content.md) | Disallow child content when using v-html or v-text |
| [`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | Disallow deprecated Vue 2 filter syntax using the pipe operator |
| [`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | Disallow the `functional` attribute on the SFC `&lt;template&gt;` |
| [`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | Disallow the `is` attribute on native HTML elements |
| [`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | Disallow the deprecated `inline-template` attribute |
| [`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | Disallow the `tag` prop on &lt;router-link&gt; |
| [`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | Disallow the deprecated `scope` attribute on &lt;template&gt; |
| [`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | Disallow the deprecated `slot` attribute |
| [`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | Disallow the deprecated `slot-scope` attribute |
| [`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | Disallow the deprecated `.sync` modifier on `v-bind` |
| [`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | Disallow the deprecated `.native` modifier on `v-on` |
| [`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | Disallow deprecated numeric `keyCode` modifiers on `v-on` |
| [`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | Disallow duplicate conditions in `v-if` / `v-else-if` chains |
| [`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | Disallow duplicate attributes on the same element |
| [`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | Disallow empty SFC blocks |
| [`vue/no-inline-style`](./reference/vue-no-inline-style.md) | Discourage use of inline style attributes |
| [`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | Disallow invalid static values for HTML attributes |
| [`vue/no-lone-template`](./reference/vue-no-lone-template.md) | Disallow unnecessary `&lt;template&gt;` elements |
| [`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | Disallow multiple consecutive spaces |
| [`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | Disallow multiple object literals inside a :class array binding |
| [`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | Disallow multiple root nodes in a template |
| [`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | Disallow mutating component props |
| [`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | Disallow a negated v-if condition when the chain has a v-else |
| [`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | Disallow plain element wrappers directly below `&lt;KeepAlive&gt;` |
| [`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | Discourage CSS preprocessor usage in favor of modern CSS |
| [`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | Disallow the use of reserved names as component names |
| [`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | Disallow v-if on the single root element of a template |
| [`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | Discourage non-standard script lang values |
| [`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | Discourage src attribute on SFC blocks |
| [`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | Disallow static inline style attributes |
| [`vue/no-template-key`](./reference/vue-no-template-key.md) | Disallow `key` attribute on `&lt;template&gt;` |
| [`vue/no-template-lang`](./reference/vue-no-template-lang.md) | Discourage lang attribute on template block |
| [`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | Disallow variable names that shadow variables in outer scope |
| [`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | Disallow target="_blank" without rel="noopener noreferrer" |
| [`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | Disallow mustache interpolation in `&lt;textarea&gt;` |
| [`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | Disallow undefined variable references in templates |
| [`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | Warn about potentially unsafe URL bindings |
| [`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | Require a sandbox attribute on iframe elements |
| [`vue/no-unused-components`](./reference/vue-no-unused-components.md) | Disallow registering components that are not used inside templates |
| [`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | Disallow unused properties defined in defineProps |
| [`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | Report template refs (ref="x") never referenced in &lt;script&gt; |
| [`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | Disallow unread script setup bindings |
| [`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | Disallow unused variable definitions in v-for and v-slot directives |
| [`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | Disallow using `v-else-if` or `v-else` on the same element as `v-for` |
| [`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | Disallow using `v-if` on the same element as `v-for` |
| [`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | Disallow a mustache interpolation whose expression is a constant string literal |
| [`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | Disallow useless attributes on `&lt;template&gt;` elements |
| [`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | Disallow a v-bind whose value is a plain string literal |
| [`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | Disallow `key` on the child of a `&lt;template v-for&gt;` |
| [`vue/no-v-html`](./reference/vue-no-v-html.md) | Warn against v-html to prevent XSS vulnerabilities |
| [`vue/no-v-text`](./reference/vue-no-v-text.md) | Disallow the v-text directive; prefer mustache interpolation |
| [`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | Disallow v-text / v-html on component elements |
| [`vue/permitted-contents`](./reference/vue-permitted-contents.md) | Enforce HTML content model rules |
| [`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | Recommend shorthand syntax for props (Vue 3.4+) |
| [`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | Prefer the shorthand for a boolean attribute bound to `true` |
| [`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | Enforce a casing for declared prop names |
| [`vue/require-component-is`](./reference/vue-require-component-is.md) | Require `v-bind:is` on `&lt;component&gt;` elements |
| [`vue/require-component-registration`](./reference/vue-require-component-registration.md) | Require explicit import or registration for components |
| [`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | Require scoped attribute on style tags |
| [`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | Require a toggle on the element wrapped by `&lt;transition&gt;` |
| [`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | Require `v-bind:key` with `v-for` directives |
| [`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | Recommend scoped event names using context:event format |
| [`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | Enforce consistent order of SFC top-level elements |
| [`vue/single-style-block`](./reference/vue-single-style-block.md) | Recommend having a single style block |
| [`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | Enforce kebab-case for named slots used via v-slot |
| [`vue/this-in-template`](./reference/vue-this-in-template.md) | Disallow `this.` in template expressions |
| [`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | Enforce unique element IDs using useId() instead of static literals |
| [`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | Enforce `.exact` modifier on `v-on` when there are modifier-based handlers |
| [`vue/v-bind-style`](./reference/vue-v-bind-style.md) | Enforce `v-bind` directive style |
| [`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | Enforce hyphenation of custom event names in v-on on components |
| [`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | Enforce writing v-on handlers as a method reference or an inline function |
| [`vue/v-on-style`](./reference/vue-v-on-style.md) | Enforce `v-on` directive style |
| [`vue/v-slot-style`](./reference/vue-v-slot-style.md) | Enforce `v-slot` directive style |
| [`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | Require valid attribute names |
| [`vue/valid-template-root`](./reference/vue-valid-template-root.md) | Enforce a valid `&lt;template&gt;` root for Vue 3 fragment semantics |
| [`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | Enforce valid `v-bind` directives |
| [`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | Enforce valid `v-cloak` directives |
| [`vue/valid-v-else`](./reference/vue-valid-v-else.md) | Enforce valid `v-else` directives |
| [`vue/valid-v-for`](./reference/vue-valid-v-for.md) | Enforce valid `v-for` directives |
| [`vue/valid-v-html`](./reference/vue-valid-v-html.md) | Enforce valid `v-html` directives |
| [`vue/valid-v-if`](./reference/vue-valid-v-if.md) | Enforce valid `v-if` directives |
| [`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | Enforce valid `v-memo` directives |
| [`vue/valid-v-model`](./reference/vue-valid-v-model.md) | Enforce valid `v-model` directives |
| [`vue/valid-v-on`](./reference/vue-valid-v-on.md) | Enforce valid `v-on` directives |
| [`vue/valid-v-once`](./reference/vue-valid-v-once.md) | Enforce valid `v-once` directives |
| [`vue/valid-v-show`](./reference/vue-valid-v-show.md) | Enforce valid `v-show` directives |
| [`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | Enforce valid `v-slot` directives |
| [`vue/valid-v-text`](./reference/vue-valid-v-text.md) | Enforce valid `v-text` directives |
| [`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | Warn about custom blocks in SFC files |
| [`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | Warn about custom directives that need registration |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)

For attributes across imported components, see [vue/cross-file-attrs-fallthrough](./project/vue-cross-file-attrs-fallthrough.md).
