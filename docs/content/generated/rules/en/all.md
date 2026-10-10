---
title: All lint rules
---

# All lint rules

All 251 source catalog entries with purpose, scope, configuration, and Bad/Good examples on this page. Each example explains the finding and repair, with current support gaps stated explicitly.

With Vite+, import `defineConfig` from `@vizejs/vite-plugin/vite-plus`, configure `lint.vize.rules`, and run `vp run lint` for Vize and Oxlint diagnostics.

Severity is the implementation default; override it with `off`, `warn`, or `error`. See [Rule Options](./options.md) for rule-option support and [Cross-file rules](./cross-file.md) for project-graph findings.

`_none_` means explicit enablement or host configuration is required. `general-recommended` is displayed as `happy-path`.

See the [ESLint migration map](./migration.md) for rule IDs, differences, and unsupported mappings.

## Single-file rules (251)

| Rule | Examples | Severity | Presets | Fixable | Options | Implementation | Description | Category |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [Bad](#petite-vue-no-unsupported-directive-bad) · [Good](#petite-vue-no-unsupported-directive-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) | Disallow directives that petite-vue does not support | Essential |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [Bad](#petite-vue-valid-v-effect-bad) · [Good](#petite-vue-valid-v-effect-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) | Require v-effect to have a non-empty expression | Essential |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [Bad](#petite-vue-valid-v-scope-bad) · [Good](#petite-vue-valid-v-scope-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) | Require v-scope to bind an object literal | Essential |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [Bad](#vue-multi-word-component-names-bad) · [Good](#vue-multi-word-component-names-good) | `error` | `essential`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) | Require component names to be multi-word | Essential |
| [`vue/no-child-content`](#vue-no-child-content) | [Bad](#vue-no-child-content-bad) · [Good](#vue-no-child-content-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) | Disallow child content when using v-html or v-text | Essential |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [Bad](#vue-no-deprecated-filter-bad) · [Good](#vue-no-deprecated-filter-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) | Disallow deprecated Vue 2 filter syntax using the pipe operator | Essential |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [Bad](#vue-no-deprecated-functional-template-bad) · [Good](#vue-no-deprecated-functional-template-good) | `error` | `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) | Disallow the `functional` attribute on the SFC `&lt;template&gt;` | Essential |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [Bad](#vue-no-deprecated-html-element-is-bad) · [Good](#vue-no-deprecated-html-element-is-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) | Disallow the `is` attribute on native HTML elements | Essential |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [Bad](#vue-no-deprecated-inline-template-bad) · [Good](#vue-no-deprecated-inline-template-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) | Disallow the deprecated `inline-template` attribute | Essential |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [Bad](#vue-no-deprecated-router-link-tag-prop-bad) · [Good](#vue-no-deprecated-router-link-tag-prop-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) | Disallow the `tag` prop on &lt;router-link&gt; | Essential |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [Bad](#vue-no-deprecated-scope-attribute-bad) · [Good](#vue-no-deprecated-scope-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) | Disallow the deprecated `scope` attribute on &lt;template&gt; | Essential |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [Bad](#vue-no-deprecated-slot-attribute-bad) · [Good](#vue-no-deprecated-slot-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) | Disallow the deprecated `slot` attribute | Essential |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [Bad](#vue-no-deprecated-slot-scope-attribute-bad) · [Good](#vue-no-deprecated-slot-scope-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) | Disallow the deprecated `slot-scope` attribute | Essential |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [Bad](#vue-no-deprecated-v-bind-sync-bad) · [Good](#vue-no-deprecated-v-bind-sync-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) | Disallow the deprecated `.sync` modifier on `v-bind` | Essential |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [Bad](#vue-no-deprecated-v-on-native-modifier-bad) · [Good](#vue-no-deprecated-v-on-native-modifier-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) | Disallow the deprecated `.native` modifier on `v-on` | Essential |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [Bad](#vue-no-deprecated-v-on-number-modifiers-bad) · [Good](#vue-no-deprecated-v-on-number-modifiers-good) | `error` | `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) | Disallow deprecated numeric `keyCode` modifiers on `v-on` | Essential |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [Bad](#vue-no-dupe-v-else-if-bad) · [Good](#vue-no-dupe-v-else-if-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) | Disallow duplicate conditions in `v-if` / `v-else-if` chains | Essential |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [Bad](#vue-no-duplicate-attributes-bad) · [Good](#vue-no-duplicate-attributes-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) | Disallow duplicate attributes on the same element | Essential |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [Bad](#vue-no-multiple-template-root-bad) · [Good](#vue-no-multiple-template-root-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) | Disallow multiple root nodes in a template | Essential |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [Bad](#vue-no-mutating-props-bad) · [Good](#vue-no-mutating-props-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L42) | Disallow mutating component props | Essential |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [Bad](#vue-no-reserved-component-names-bad) · [Good](#vue-no-reserved-component-names-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) | Disallow the use of reserved names as component names | Essential |
| [`vue/no-template-key`](#vue-no-template-key) | [Bad](#vue-no-template-key-bad) · [Good](#vue-no-template-key-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) | Disallow `key` attribute on `&lt;template&gt;` | Essential |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [Bad](#vue-no-textarea-mustache-bad) · [Good](#vue-no-textarea-mustache-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) | Disallow mustache interpolation in `&lt;textarea&gt;` | Essential |
| [`vue/no-unused-components`](#vue-no-unused-components) | [Bad](#vue-no-unused-components-bad) · [Good](#vue-no-unused-components-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) | Disallow registering components that are not used inside templates | Essential |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [Bad](#vue-no-unused-vars-bad) · [Good](#vue-no-unused-vars-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) | Disallow unused variable definitions in v-for and v-slot directives | Essential |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [Bad](#vue-no-use-v-if-with-v-for-bad) · [Good](#vue-no-use-v-if-with-v-for-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) | Disallow using `v-if` on the same element as `v-for` | Essential |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [Bad](#vue-no-useless-template-attributes-bad) · [Good](#vue-no-useless-template-attributes-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) | Disallow useless attributes on `&lt;template&gt;` elements | Essential |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [Bad](#vue-no-v-for-template-key-on-child-bad) · [Good](#vue-no-v-for-template-key-on-child-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) | Disallow `key` on the child of a `&lt;template v-for&gt;` | Essential |
| [`vue/no-v-html`](#vue-no-v-html) | [Bad](#vue-no-v-html-bad) · [Good](#vue-no-v-html-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) | Warn against v-html to prevent XSS vulnerabilities | Essential |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [Bad](#vue-no-v-text-v-html-on-component-bad) · [Good](#vue-no-v-text-v-html-on-component-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) | Disallow v-text / v-html on component elements | Essential |
| [`vue/permitted-contents`](#vue-permitted-contents) | [Bad](#vue-permitted-contents-bad) · [Good](#vue-permitted-contents-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) | Enforce HTML content model rules | Essential |
| [`vue/require-component-is`](#vue-require-component-is) | [Bad](#vue-require-component-is-bad) · [Good](#vue-require-component-is-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) | Require `v-bind:is` on `&lt;component&gt;` elements | Essential |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [Bad](#vue-require-toggle-inside-transition-bad) · [Good](#vue-require-toggle-inside-transition-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) | Require a toggle on the element wrapped by `&lt;transition&gt;` | Essential |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [Bad](#vue-require-v-for-key-bad) · [Good](#vue-require-v-for-key-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) | Require `v-bind:key` with `v-for` directives | Essential |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [Bad](#vue-use-v-on-exact-bad) · [Good](#vue-use-v-on-exact-good) | `warning` | `essential`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) | Enforce `.exact` modifier on `v-on` when there are modifier-based handlers | Essential |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [Bad](#vue-valid-attribute-name-bad) · [Good](#vue-valid-attribute-name-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) | Require valid attribute names | Essential |
| [`vue/valid-template-root`](#vue-valid-template-root) | [Bad](#vue-valid-template-root-bad) · [Good](#vue-valid-template-root-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) | Enforce a valid `&lt;template&gt;` root for Vue 3 fragment semantics | Essential |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [Bad](#vue-valid-v-bind-bad) · [Good](#vue-valid-v-bind-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) | Enforce valid `v-bind` directives | Essential |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [Bad](#vue-valid-v-cloak-bad) · [Good](#vue-valid-v-cloak-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) | Enforce valid `v-cloak` directives | Essential |
| [`vue/valid-v-else`](#vue-valid-v-else) | [Bad](#vue-valid-v-else-bad) · [Good](#vue-valid-v-else-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) | Enforce valid `v-else` directives | Essential |
| [`vue/valid-v-for`](#vue-valid-v-for) | [Bad](#vue-valid-v-for-bad) · [Good](#vue-valid-v-for-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) | Enforce valid `v-for` directives | Essential |
| [`vue/valid-v-html`](#vue-valid-v-html) | [Bad](#vue-valid-v-html-bad) · [Good](#vue-valid-v-html-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) | Enforce valid `v-html` directives | Essential |
| [`vue/valid-v-if`](#vue-valid-v-if) | [Bad](#vue-valid-v-if-bad) · [Good](#vue-valid-v-if-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) | Enforce valid `v-if` directives | Essential |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [Bad](#vue-valid-v-memo-bad) · [Good](#vue-valid-v-memo-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) | Enforce valid `v-memo` directives | Essential |
| [`vue/valid-v-model`](#vue-valid-v-model) | [Bad](#vue-valid-v-model-bad) · [Good](#vue-valid-v-model-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) | Enforce valid `v-model` directives | Essential |
| [`vue/valid-v-on`](#vue-valid-v-on) | [Bad](#vue-valid-v-on-bad) · [Good](#vue-valid-v-on-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) | Enforce valid `v-on` directives | Essential |
| [`vue/valid-v-once`](#vue-valid-v-once) | [Bad](#vue-valid-v-once-bad) · [Good](#vue-valid-v-once-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) | Enforce valid `v-once` directives | Essential |
| [`vue/valid-v-show`](#vue-valid-v-show) | [Bad](#vue-valid-v-show-bad) · [Good](#vue-valid-v-show-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) | Enforce valid `v-show` directives | Essential |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [Bad](#vue-valid-v-slot-bad) · [Good](#vue-valid-v-slot-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) | Enforce valid `v-slot` directives | Essential |
| [`vue/valid-v-text`](#vue-valid-v-text) | [Bad](#vue-valid-v-text-bad) · [Good](#vue-valid-v-text-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) | Enforce valid `v-text` directives | Essential |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [Bad](#vue-attribute-hyphenation-bad) · [Good](#vue-attribute-hyphenation-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) | Enforce attribute naming style on custom components | Strongly Recommended |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [Bad](#vue-component-definition-name-casing-bad) · [Good](#vue-component-definition-name-casing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) | Enforce PascalCase or kebab-case for component definition names | Strongly Recommended |
| [`vue/html-quotes`](#vue-html-quotes) | [Bad](#vue-html-quotes-bad) · [Good](#vue-html-quotes-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) | Enforce quotes style of HTML attributes | Strongly Recommended |
| [`vue/html-self-closing`](#vue-html-self-closing) | [Bad](#vue-html-self-closing-bad) · [Good](#vue-html-self-closing-good) | `warning` | `nuxt`, `opinionated` | Yes | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) | Enforce self-closing style | Strongly Recommended |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [Bad](#vue-mustache-interpolation-spacing-bad) · [Good](#vue-mustache-interpolation-spacing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) | Enforce consistent spacing inside mustache interpolations | Strongly Recommended |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [Bad](#vue-no-multi-spaces-bad) · [Good](#vue-no-multi-spaces-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) | Disallow multiple consecutive spaces | Strongly Recommended |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [Bad](#vue-no-template-shadow-bad) · [Good](#vue-no-template-shadow-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) | Disallow variable names that shadow variables in outer scope | Strongly Recommended |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [Bad](#vue-no-unused-properties-bad) · [Good](#vue-no-unused-properties-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) | Disallow unused properties defined in defineProps | Strongly Recommended |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [Bad](#vue-prop-name-casing-bad) · [Good](#vue-prop-name-casing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) | Enforce a casing for declared prop names | Strongly Recommended |
| [`vue/v-bind-style`](#vue-v-bind-style) | [Bad](#vue-v-bind-style-bad) · [Good](#vue-v-bind-style-good) | `warning` | `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) | Enforce `v-bind` directive style | Strongly Recommended |
| [`vue/v-on-style`](#vue-v-on-style) | [Bad](#vue-v-on-style-bad) · [Good](#vue-v-on-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) | Enforce `v-on` directive style | Strongly Recommended |
| [`vue/v-slot-style`](#vue-v-slot-style) | [Bad](#vue-v-slot-style-bad) · [Good](#vue-v-slot-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) | Enforce `v-slot` directive style | Strongly Recommended |
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [Bad](#ssr-no-browser-globals-in-ssr-bad) · [Good](#ssr-no-browser-globals-in-ssr-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) | Disallow browser-only globals in SSR context | Recommended |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [Bad](#ssr-no-hydration-mismatch-bad) · [Good](#ssr-no-hydration-mismatch-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) | Disallow non-deterministic values that cause hydration mismatch | Recommended |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [Bad](#vue-a11y-img-alt-bad) · [Good](#vue-a11y-img-alt-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) | Require alt attribute on images for accessibility | Recommended |
| [`vue/attribute-order`](#vue-attribute-order) | [Bad](#vue-attribute-order-bad) · [Good](#vue-attribute-order-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) | Enforce a consistent order of attributes | Recommended |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [Bad](#vue-component-name-in-template-casing-bad) · [Good](#vue-component-name-in-template-casing-good) | `warning` | `nuxt`, `opinionated` | Yes | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) | Enforce specific casing for component names in templates | Recommended |
| [`vue/html-button-has-type`](#vue-html-button-has-type) | [Bad](#vue-html-button-has-type-bad) · [Good](#vue-html-button-has-type-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) | Require an explicit valid type on button elements | Recommended |
| [`vue/max-template-complexity`](#vue-max-template-complexity) | [Bad](#vue-max-template-complexity-bad) · [Good](#vue-max-template-complexity-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) | Limit a component's own template complexity (cyclomatic and cognitive) | Recommended |
| [`vue/no-array-index-key`](#vue-no-array-index-key) | [Bad](#vue-no-array-index-key-bad) · [Good](#vue-no-array-index-key-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) | Disallow using the v-for index variable directly as the :key | Recommended |
| [`vue/no-bare-strings-in-template`](#vue-no-bare-strings-in-template) | [Bad](#vue-no-bare-strings-in-template-bad) · [Good](#vue-no-bare-strings-in-template-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47) | Disallow raw human-readable text in the template that should be internationalized | Recommended |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [Bad](#vue-no-boolean-attr-value-bad) · [Good](#vue-no-boolean-attr-value-good) | `warning` | `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) | Disallow explicit values for boolean HTML attributes | Recommended |
| [`vue/no-empty-component-block`](#vue-no-empty-component-block) | [Bad](#vue-no-empty-component-block-bad) · [Good](#vue-no-empty-component-block-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) | Disallow empty SFC blocks | Recommended |
| [`vue/no-inline-style`](#vue-no-inline-style) | [Bad](#vue-no-inline-style-bad) · [Good](#vue-no-inline-style-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) | Discourage use of inline style attributes | Recommended |
| [`vue/no-invalid-html-attribute`](#vue-no-invalid-html-attribute) | [Bad](#vue-no-invalid-html-attribute-bad) · [Good](#vue-no-invalid-html-attribute-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) | Disallow invalid static values for HTML attributes | Recommended |
| [`vue/no-lone-template`](#vue-no-lone-template) | [Bad](#vue-no-lone-template-bad) · [Good](#vue-no-lone-template-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) | Disallow unnecessary `&lt;template&gt;` elements | Recommended |
| [`vue/no-multiple-objects-in-class`](#vue-no-multiple-objects-in-class) | [Bad](#vue-no-multiple-objects-in-class-bad) · [Good](#vue-no-multiple-objects-in-class-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) | Disallow multiple object literals inside a :class array binding | Recommended |
| [`vue/no-negated-v-if-condition`](#vue-no-negated-v-if-condition) | [Bad](#vue-no-negated-v-if-condition-bad) · [Good](#vue-no-negated-v-if-condition-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) | Disallow a negated v-if condition when the chain has a v-else | Recommended |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [Bad](#vue-no-non-component-keep-alive-child-bad) · [Good](#vue-no-non-component-keep-alive-child-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L14) | Disallow plain element wrappers directly below `&lt;KeepAlive&gt;` | Recommended |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [Bad](#vue-no-preprocessor-lang-bad) · [Good](#vue-no-preprocessor-lang-good) | `warning` | `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) | Discourage CSS preprocessor usage in favor of modern CSS | Recommended |
| [`vue/no-root-v-if`](#vue-no-root-v-if) | [Bad](#vue-no-root-v-if-bad) · [Good](#vue-no-root-v-if-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) | Disallow v-if on the single root element of a template | Recommended |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [Bad](#vue-no-script-non-standard-lang-bad) · [Good](#vue-no-script-non-standard-lang-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) | Discourage non-standard script lang values | Recommended |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [Bad](#vue-no-src-attribute-bad) · [Good](#vue-no-src-attribute-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) | Discourage src attribute on SFC blocks | Recommended |
| [`vue/no-static-inline-styles`](#vue-no-static-inline-styles) | [Bad](#vue-no-static-inline-styles-bad) · [Good](#vue-no-static-inline-styles-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) | Disallow static inline style attributes | Recommended |
| [`vue/no-template-lang`](#vue-no-template-lang) | [Bad](#vue-no-template-lang-bad) · [Good](#vue-no-template-lang-good) | `warning` | `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) | Discourage lang attribute on template block | Recommended |
| [`vue/no-template-target-blank`](#vue-no-template-target-blank) | [Bad](#vue-no-template-target-blank-bad) · [Good](#vue-no-template-target-blank-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) | Disallow target="_blank" without rel="noopener noreferrer" | Recommended |
| [`vue/no-undefined-refs`](#vue-no-undefined-refs) | [Bad](#vue-no-undefined-refs-bad) · [Good](#vue-no-undefined-refs-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) | Disallow undefined variable references in templates | Recommended |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [Bad](#vue-no-unsafe-url-bad) · [Good](#vue-no-unsafe-url-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) | Warn about potentially unsafe URL bindings | Recommended |
| [`vue/no-unsandboxed-iframe`](#vue-no-unsandboxed-iframe) | [Bad](#vue-no-unsandboxed-iframe-bad) · [Good](#vue-no-unsandboxed-iframe-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) | Require a sandbox attribute on iframe elements | Recommended |
| [`vue/no-unused-refs`](#vue-no-unused-refs) | [Bad](#vue-no-unused-refs-bad) · [Good](#vue-no-unused-refs-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) | Report template refs (ref="x") never referenced in &lt;script&gt; | Recommended |
| [`vue/no-unused-setup-bindings`](#vue-no-unused-setup-bindings) | [Bad](#vue-no-unused-setup-bindings-bad) · [Good](#vue-no-unused-setup-bindings-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) | Disallow unread script setup bindings | Recommended |
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [Bad](#vue-no-use-v-else-with-v-for-bad) · [Good](#vue-no-use-v-else-with-v-for-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) | Disallow using `v-else-if` or `v-else` on the same element as `v-for` | Recommended |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [Bad](#vue-no-useless-mustaches-bad) · [Good](#vue-no-useless-mustaches-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) | Disallow a mustache interpolation whose expression is a constant string literal | Recommended |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [Bad](#vue-no-useless-v-bind-bad) · [Good](#vue-no-useless-v-bind-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) | Disallow a v-bind whose value is a plain string literal | Recommended |
| [`vue/no-v-text`](#vue-no-v-text) | [Bad](#vue-no-v-text-bad) · [Good](#vue-no-v-text-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) | Disallow the v-text directive; prefer mustache interpolation | Recommended |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [Bad](#vue-prefer-props-shorthand-bad) · [Good](#vue-prefer-props-shorthand-good) | `warning` | `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) | Recommend shorthand syntax for props (Vue 3.4+) | Recommended |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [Bad](#vue-prefer-true-attribute-shorthand-bad) · [Good](#vue-prefer-true-attribute-shorthand-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) | Prefer the shorthand for a boolean attribute bound to `true` | Recommended |
| [`vue/require-component-registration`](#vue-require-component-registration) | [Bad](#vue-require-component-registration-bad) · [Good](#vue-require-component-registration-good) | `warning` | `opinionated` | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) | Require explicit import or registration for components | Recommended |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [Bad](#vue-require-scoped-style-bad) · [Good](#vue-require-scoped-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) | Require scoped attribute on style tags | Recommended |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [Bad](#vue-scoped-event-names-bad) · [Good](#vue-scoped-event-names-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) | Recommend scoped event names using context:event format | Recommended |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [Bad](#vue-sfc-element-order-bad) · [Good](#vue-sfc-element-order-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) | Enforce consistent order of SFC top-level elements | Recommended |
| [`vue/single-style-block`](#vue-single-style-block) | [Bad](#vue-single-style-block-bad) · [Good](#vue-single-style-block-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) | Recommend having a single style block | Recommended |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [Bad](#vue-slot-name-casing-bad) · [Good](#vue-slot-name-casing-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) | Enforce kebab-case for named slots used via v-slot | Recommended |
| [`vue/this-in-template`](#vue-this-in-template) | [Bad](#vue-this-in-template-bad) · [Good](#vue-this-in-template-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) | Disallow `this.` in template expressions | Recommended |
| [`vue/v-on-event-hyphenation`](#vue-v-on-event-hyphenation) | [Bad](#vue-v-on-event-hyphenation-bad) · [Good](#vue-v-on-event-hyphenation-good) | `warning` | `nuxt`, `opinionated` | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) | Enforce hyphenation of custom event names in v-on on components | Recommended |
| [`vue/v-on-handler-style`](#vue-v-on-handler-style) | [Bad](#vue-v-on-handler-style-bad) · [Good](#vue-v-on-handler-style-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) | Enforce writing v-on handlers as a method reference or an inline function | Recommended |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [Bad](#vue-warn-custom-block-bad) · [Good](#vue-warn-custom-block-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) | Warn about custom blocks in SFC files | Recommended |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [Bad](#vue-warn-custom-directive-bad) · [Good](#vue-warn-custom-directive-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) | Warn about custom directives that need registration | Recommended |
| [`a11y/alt-text`](#a11y-alt-text) | [Bad](#a11y-alt-text-bad) · [Good](#a11y-alt-text-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) | Require alternative text for media elements | Accessibility |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [Bad](#a11y-anchor-has-content-bad) · [Good](#a11y-anchor-has-content-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) | Require anchor elements to have accessible content | Accessibility |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [Bad](#a11y-anchor-is-valid-bad) · [Good](#a11y-anchor-is-valid-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) | Enforce valid href on anchor elements | Accessibility |
| [`a11y/aria-props`](#a11y-aria-props) | [Bad](#a11y-aria-props-bad) · [Good](#a11y-aria-props-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) | Disallow invalid ARIA attributes | Accessibility |
| [`a11y/aria-role`](#a11y-aria-role) | [Bad](#a11y-aria-role-bad) · [Good](#a11y-aria-role-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) | Elements with ARIA roles must use a valid, non-abstract ARIA role | Accessibility |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [Bad](#a11y-aria-unsupported-elements-bad) · [Good](#a11y-aria-unsupported-elements-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) | Disallow ARIA attributes on elements that do not support them | Accessibility |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [Bad](#a11y-click-events-have-key-events-bad) · [Good](#a11y-click-events-have-key-events-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) | Require keyboard event handlers with click events | Accessibility |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [Bad](#a11y-form-control-has-label-bad) · [Good](#a11y-form-control-has-label-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) | Require form controls to have associated labels | Accessibility |
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [Bad](#a11y-heading-has-content-bad) · [Good](#a11y-heading-has-content-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) | Require heading elements to have accessible content | Accessibility |
| [`a11y/heading-levels`](#a11y-heading-levels) | [Bad](#a11y-heading-levels-bad) · [Good](#a11y-heading-levels-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) | Disallow skipping heading levels | Accessibility |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [Bad](#a11y-iframe-has-title-bad) · [Good](#a11y-iframe-has-title-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) | Require iframe elements to have a title attribute | Accessibility |
| [`a11y/img-alt`](#a11y-img-alt) | [Bad](#a11y-img-alt-bad) · [Good](#a11y-img-alt-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) | Require alt attribute on images for accessibility | Accessibility |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [Bad](#a11y-interactive-supports-focus-bad) · [Good](#a11y-interactive-supports-focus-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) | Require interactive role elements to be focusable | Accessibility |
| [`a11y/label-has-for`](#a11y-label-has-for) | [Bad](#a11y-label-has-for-bad) · [Good](#a11y-label-has-for-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) | Require labels to have associated form controls | Accessibility |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [Bad](#a11y-landmark-roles-bad) · [Good](#a11y-landmark-roles-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) | Validate landmark role placement and uniqueness | Accessibility |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [Bad](#a11y-media-has-caption-bad) · [Good](#a11y-media-has-caption-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) | Require media elements to have captions | Accessibility |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [Bad](#a11y-mouse-events-have-key-events-bad) · [Good](#a11y-mouse-events-have-key-events-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) | Require focus/blur events with mouse events | Accessibility |
| [`a11y/no-access-key`](#a11y-no-access-key) | [Bad](#a11y-no-access-key-bad) · [Good](#a11y-no-access-key-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) | Disallow the use of the accesskey attribute | Accessibility |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [Bad](#a11y-no-aria-hidden-on-focusable-bad) · [Good](#a11y-no-aria-hidden-on-focusable-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) | Disallow aria-hidden="true" on focusable elements | Accessibility |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [Bad](#a11y-no-autofocus-bad) · [Good](#a11y-no-autofocus-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) | Disallow the use of the autofocus attribute | Accessibility |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [Bad](#a11y-no-distracting-elements-bad) · [Good](#a11y-no-distracting-elements-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) | Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt; | Accessibility |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [Bad](#a11y-no-i-for-icon-bad) · [Good](#a11y-no-i-for-icon-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) | Disallow using &lt;i&gt; element for icons | Accessibility |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [Bad](#a11y-no-redundant-roles-bad) · [Good](#a11y-no-redundant-roles-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) | Disallow redundant ARIA roles | Accessibility |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [Bad](#a11y-no-refer-to-non-existent-id-bad) · [Good](#a11y-no-refer-to-non-existent-id-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) | Disallow references to non-existent IDs | Accessibility |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [Bad](#a11y-no-role-presentation-on-focusable-bad) · [Good](#a11y-no-role-presentation-on-focusable-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) | Disallow role="presentation" or role="none" on focusable elements | Accessibility |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [Bad](#a11y-no-static-element-interactions-bad) · [Good](#a11y-no-static-element-interactions-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) | Disallow event handlers on static elements | Accessibility |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [Bad](#a11y-placeholder-label-option-bad) · [Good](#a11y-placeholder-label-option-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) | Require disabled or hidden on select placeholder option | Accessibility |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [Bad](#a11y-role-has-required-aria-props-bad) · [Good](#a11y-role-has-required-aria-props-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) | Require ARIA roles to have required properties | Accessibility |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [Bad](#a11y-tabindex-no-positive-bad) · [Good](#a11y-tabindex-no-positive-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) | Disallow positive tabindex values | Accessibility |
| [`a11y/use-list`](#a11y-use-list) | [Bad](#a11y-use-list-bad) · [Good](#a11y-use-list-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) | Suggest using list elements for bullet-like text | Accessibility |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Bad](#vue-use-unique-element-ids-bad) · [Good](#vue-use-unique-element-ids-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) | Enforce unique element IDs using useId() instead of static literals | Accessibility |
| [`html/deprecated-attr`](#html-deprecated-attr) | [Bad](#html-deprecated-attr-bad) · [Good](#html-deprecated-attr-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) | Disallow deprecated HTML attributes | HTML Conformance |
| [`html/deprecated-element`](#html-deprecated-element) | [Bad](#html-deprecated-element-bad) · [Good](#html-deprecated-element-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) | Disallow deprecated HTML elements | HTML Conformance |
| [`html/id-duplication`](#html-id-duplication) | [Bad](#html-id-duplication-bad) · [Good](#html-id-duplication-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) | Disallow duplicate element IDs | HTML Conformance |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [Bad](#html-no-consecutive-br-bad) · [Good](#html-no-consecutive-br-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) | Disallow consecutive &lt;br&gt; elements | HTML Conformance |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [Bad](#html-no-dupe-style-properties-bad) · [Good](#html-no-dupe-style-properties-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) | Disallow duplicate properties in inline style attributes | HTML Conformance |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [Bad](#html-no-duplicate-class-bad) · [Good](#html-no-duplicate-class-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) | Disallow duplicate class names in a static class attribute | HTML Conformance |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [Bad](#html-no-duplicate-dt-bad) · [Good](#html-no-duplicate-dt-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) | Disallow duplicate &lt;dt&gt; names in &lt;dl&gt; | HTML Conformance |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [Bad](#html-no-empty-palpable-content-bad) · [Good](#html-no-empty-palpable-content-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) | Disallow empty elements that expect visible content | HTML Conformance |
| [`html/require-datetime`](#html-require-datetime) | [Bad](#html-require-datetime-bad) · [Good](#html-require-datetime-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) | Require datetime attribute on &lt;time&gt; element | HTML Conformance |
| [`type/no-floating-promises`](#type-no-floating-promises) | [Bad](#type-no-floating-promises-bad) · [Good](#type-no-floating-promises-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) | Disallow floating (unhandled) Promises | Type Aware |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [Bad](#type-no-reactivity-loss-bad) · [Good](#type-no-reactivity-loss-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) | Disallow plain snapshots of reactive values across assignments and calls | Type Aware |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [Bad](#type-no-unsafe-template-binding-bad) · [Good](#type-no-unsafe-template-binding-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) | Disallow template bindings that resolve to unsafe types | Type Aware |
| [`type/require-typed-emits`](#type-require-typed-emits) | [Bad](#type-require-typed-emits-bad) · [Good](#type-require-typed-emits-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) | Require type definition for defineEmits | Type Aware |
| [`type/require-typed-props`](#type-require-typed-props) | [Bad](#type-require-typed-props-bad) · [Good](#type-require-typed-props-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) | Require type definition for defineProps | Type Aware |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [Bad](#type-strict-boolean-expressions-bad) · [Good](#type-strict-boolean-expressions-good) | `warning` | _none_ | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) | Require safe boolean expressions in script and template conditions | Type Aware |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [Bad](#script-no-get-current-instance-bad) · [Good](#script-no-get-current-instance-good) | `error` | `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) | Disallow getCurrentInstance() in Vapor mode (returns null) | Vapor |
| [`script/no-next-tick`](#script-no-next-tick) | [Bad](#script-no-next-tick-bad) · [Good](#script-no-next-tick-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) | Disallow nextTick() usage in Vapor-oriented components | Vapor |
| [`script/no-options-api`](#script-no-options-api) | [Bad](#script-no-options-api-bad) · [Good](#script-no-options-api-good) | `error` | `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) | Disallow Options API patterns in Vapor mode | Vapor |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [Bad](#vapor-no-inline-template-bad) · [Good](#vapor-no-inline-template-good) | `error` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) | Disallow deprecated inline-template attribute | Vapor |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [Bad](#vapor-no-vue-lifecycle-events-bad) · [Good](#vapor-no-vue-lifecycle-events-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) | Disallow @vue:xxx per-element lifecycle events (not supported in Vapor) | Vapor |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [Bad](#vapor-prefer-static-class-bad) · [Good](#vapor-prefer-static-class-good) | `warning` | `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) | Prefer static class over dynamic class binding for string literals | Vapor |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [Bad](#vapor-require-vapor-attribute-bad) · [Good](#vapor-require-vapor-attribute-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) | Suggest adding vapor attribute to script setup | Vapor |
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [Bad](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Good](#ecosystem-nuxt-prefer-nuxt-link-good) | `warning` | `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) | Prefer NuxtLink for internal application links | Ecosystem |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [Bad](#ecosystem-pinia-prefer-store-to-refs-bad) · [Good](#ecosystem-pinia-prefer-store-to-refs-good) | `warning` | `ecosystem` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) | Prefer storeToRefs() when destructuring Pinia stores | Ecosystem |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [Bad](#ecosystem-router-link-require-to-bad) · [Good](#ecosystem-router-link-require-to-good) | `error` | `ecosystem` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) | Require a `to` target on RouterLink and NuxtLink components | Ecosystem |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [Bad](#ecosystem-void-link-require-href-bad) · [Good](#ecosystem-void-link-require-href-good) | `error` | `ecosystem` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) | Require `href` on Void Vue Link components | Ecosystem |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [Bad](#ecosystem-void-link-valid-method-bad) · [Good](#ecosystem-void-link-valid-method-good) | `warning` | `ecosystem` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) | Validate static Void Vue Link method props | Ecosystem |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [Bad](#ecosystem-vue-i18n-no-missing-key-bad) · [Good](#ecosystem-vue-i18n-no-missing-key-good) | `warning` | `ecosystem` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) | Report static vue-i18n keys that are absent from local SFC messages | Ecosystem |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [Bad](#ecosystem-vue-router-prefer-named-link-bad) · [Good](#ecosystem-vue-router-prefer-named-link-good) | `warning` | `ecosystem` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) | Prefer named route objects over static path strings in RouterLink | Ecosystem |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [Bad](#ecosystem-vue-router-prefer-named-push-bad) · [Good](#ecosystem-vue-router-prefer-named-push-good) | `warning` | `ecosystem` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) | Prefer named route objects for Vue Router programmatic navigation | Ecosystem |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [Bad](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Good](#ecosystem-vue-test-utils-no-html-snapshot-good) | `warning` | `ecosystem` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) | Avoid snapshotting wrapper.html() in Vue Test Utils tests | Ecosystem |
| [`css/no-display-none`](#css-no-display-none) | [Bad](#css-no-display-none-bad) · [Good](#css-no-display-none-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) | Suggest using v-show instead of display: none | CSS |
| [`css/no-hardcoded-values`](#css-no-hardcoded-values) | [Bad](#css-no-hardcoded-values-bad) · [Good](#css-no-hardcoded-values-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) | Suggest using CSS variables instead of hardcoded values | CSS |
| [`css/no-id-selectors`](#css-no-id-selectors) | [Bad](#css-no-id-selectors-bad) · [Good](#css-no-id-selectors-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) | Discourage use of ID selectors in CSS | CSS |
| [`css/no-important`](#css-no-important) | [Bad](#css-no-important-bad) · [Good](#css-no-important-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) | Discourage use of !important in CSS | CSS |
| [`css/no-utility-classes`](#css-no-utility-classes) | [Bad](#css-no-utility-classes-bad) · [Good](#css-no-utility-classes-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) | Warn against implementing utility classes in component styles | CSS |
| [`css/no-v-bind-performance`](#css-no-v-bind-performance) | [Bad](#css-no-v-bind-performance-bad) · [Good](#css-no-v-bind-performance-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) | Warn about performance cost of CSS v-bind() | CSS |
| [`css/prefer-logical-properties`](#css-prefer-logical-properties) | [Bad](#css-prefer-logical-properties-bad) · [Good](#css-prefer-logical-properties-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) | Recommend CSS logical properties for better i18n support | CSS |
| [`css/prefer-nested-selectors`](#css-prefer-nested-selectors) | [Bad](#css-prefer-nested-selectors-bad) · [Good](#css-prefer-nested-selectors-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) | Recommend using CSS nesting for descendant selectors | CSS |
| [`css/prefer-slotted`](#css-prefer-slotted) | [Bad](#css-prefer-slotted-bad) · [Good](#css-prefer-slotted-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) | Recommend ::v-slotted() for styling slot content | CSS |
| [`css/require-font-display`](#css-require-font-display) | [Bad](#css-require-font-display-bad) · [Good](#css-require-font-display-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) | Require font-display in @font-face rules | CSS |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [Bad](#musea-no-empty-variant-bad) · [Good](#musea-no-empty-variant-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) | Disallow empty &lt;variant&gt; blocks | Musea |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [Bad](#musea-prefer-design-tokens-bad) · [Good](#musea-prefer-design-tokens-good) | `warning` | _none_ | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) | Prefer design token CSS variables over hardcoded primitive values | Musea |
| [`musea/require-component`](#musea-require-component) | [Bad](#musea-require-component-bad) · [Good](#musea-require-component-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) | Require component attribute in &lt;art&gt; block | Musea |
| [`musea/require-title`](#musea-require-title) | [Bad](#musea-require-title-bad) · [Good](#musea-require-title-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) | Require title attribute in &lt;art&gt; block | Musea |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [Bad](#musea-unique-variant-names-bad) · [Good](#musea-unique-variant-names-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) | Require unique variant names | Musea |
| [`musea/valid-variant`](#musea-valid-variant) | [Bad](#musea-valid-variant-bad) · [Good](#musea-valid-variant-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) | Require name attribute in &lt;variant&gt; blocks | Musea |
| [`script/component-options-name-casing`](#script-component-options-name-casing) | [Bad](#script-component-options-name-casing-bad) · [Good](#script-component-options-name-casing-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) | Enforce PascalCase for the component `name` option | Script |
| [`script/custom-event-name-casing`](#script-custom-event-name-casing) | [Bad](#script-custom-event-name-casing-bad) · [Good](#script-custom-event-name-casing-good) | `error` | _none_ | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) | Enforce camelCase for emitted custom event names | Script |
| [`script/define-emits-declaration`](#script-define-emits-declaration) | [Bad](#script-define-emits-declaration-bad) · [Good](#script-define-emits-declaration-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) | Enforce the type-based defineEmits&lt;{}&gt;() form over the runtime/array form | Script |
| [`script/define-macros-order`](#script-define-macros-order) | [Bad](#script-define-macros-order-bad) · [Good](#script-define-macros-order-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L46) | Enforce a consistent order of the Vue compiler macros in &lt;script setup&gt; | Script |
| [`script/define-props-declaration`](#script-define-props-declaration) | [Bad](#script-define-props-declaration-bad) · [Good](#script-define-props-declaration-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) | Enforce type-based defineProps&lt;{ ... }&gt;() over the runtime/object form | Script |
| [`script/define-props-destructuring`](#script-define-props-destructuring) | [Bad](#script-define-props-destructuring-bad) · [Good](#script-define-props-destructuring-good) | `warning` | _none_ | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) | Enforce consistent style for defineProps destructuring in &lt;script setup&gt; | Script |
| [`script/no-arrow-functions-in-watch`](#script-no-arrow-functions-in-watch) | [Bad](#script-no-arrow-functions-in-watch-bad) · [Good](#script-no-arrow-functions-in-watch-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) | Disallow arrow functions as Options API watch handlers | Script |
| [`script/no-async-in-computed`](#script-no-async-in-computed) | [Bad](#script-no-async-in-computed-bad) · [Good](#script-no-async-in-computed-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) | Disallow async functions in computed properties | Script |
| [`script/no-boolean-default`](#script-no-boolean-default) | [Bad](#script-no-boolean-default-bad) · [Good](#script-no-boolean-default-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) | Disallow a default on a Boolean prop | Script |
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [Bad](#script-no-deep-destructure-in-props-bad) · [Good](#script-no-deep-destructure-in-props-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) | Disallow deeply nested destructuring in defineProps | Script |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [Bad](#script-no-deprecated-data-object-declaration-bad) · [Good](#script-no-deprecated-data-object-declaration-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) | Disallow an object literal as the component data option (Vue 3 requires a function) | Script |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [Bad](#script-no-deprecated-destroyed-lifecycle-bad) · [Good](#script-no-deprecated-destroyed-lifecycle-good) | `error` | _none_ | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) | Disallow deprecated destroyed and beforeDestroy lifecycle hooks | Script |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [Bad](#script-no-deprecated-dollar-listeners-api-bad) · [Good](#script-no-deprecated-dollar-listeners-api-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) | Disallow the $listeners instance property removed in Vue 3 (merged into $attrs) | Script |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [Bad](#script-no-deprecated-dollar-scopedslots-api-bad) · [Good](#script-no-deprecated-dollar-scopedslots-api-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) | Disallow the $scopedSlots instance property removed in Vue 3 (use $slots) | Script |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [Bad](#script-no-deprecated-events-api-bad) · [Good](#script-no-deprecated-events-api-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) | Disallow the removed Vue 2 events API ($on / $off / $once) | Script |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [Bad](#script-no-deprecated-props-default-this-bad) · [Good](#script-no-deprecated-props-default-this-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) | Disallow `this` inside a prop default/validator function (removed in Vue 3) | Script |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [Bad](#script-no-dupe-keys-bad) · [Good](#script-no-dupe-keys-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L52) | Disallow duplicate keys across Options API props/data/computed/methods/setup/inject | Script |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [Bad](#script-no-duplicate-attr-inheritance-bad) · [Good](#script-no-duplicate-attr-inheritance-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) | Flag a component that applies its fallthrough attributes twice | Script |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [Bad](#script-no-export-in-script-setup-bad) · [Good](#script-no-export-in-script-setup-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) | Disallow export statements inside &lt;script setup&gt; | Script |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [Bad](#script-no-import-compiler-macros-bad) · [Good](#script-no-import-compiler-macros-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) | Disallow importing Vue compiler macros that are auto-imported | Script |
| [`script/no-internal-imports`](#script-no-internal-imports) | [Bad](#script-no-internal-imports-bad) · [Good](#script-no-internal-imports-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) | Disallow importing from Vue internal modules | Script |
| [`script/no-multiple-slot-args`](#script-no-multiple-slot-args) | [Bad](#script-no-multiple-slot-args-bad) · [Good](#script-no-multiple-slot-args-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) | Disallow passing more than one argument to a scoped-slot function call | Script |
| [`script/no-potential-component-option-typo`](#script-no-potential-component-option-typo) | [Bad](#script-no-potential-component-option-typo-bad) · [Good](#script-no-potential-component-option-typo-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) | Flag likely typos in Options API component option names | Script |
| [`script/no-reactive-destructure`](#script-no-reactive-destructure) | [Bad](#script-no-reactive-destructure-bad) · [Good](#script-no-reactive-destructure-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) | Disallow destructuring reactive objects which loses reactivity | Script |
| [`script/no-ref-as-operand`](#script-no-ref-as-operand) | [Bad](#script-no-ref-as-operand-bad) · [Good](#script-no-ref-as-operand-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) | Require ref-bound variables to be accessed via `.value` when used as an operand | Script |
| [`script/no-required-prop-with-default`](#script-no-required-prop-with-default) | [Bad](#script-no-required-prop-with-default-bad) · [Good](#script-no-required-prop-with-default-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) | Disallow a prop that is both required: true and has a default | Script |
| [`script/no-reserved-identifiers`](#script-no-reserved-identifiers) | [Bad](#script-no-reserved-identifiers-bad) · [Good](#script-no-reserved-identifiers-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) | Disallow using Vue compiler reserved identifiers | Script |
| [`script/no-reserved-keys`](#script-no-reserved-keys) | [Bad](#script-no-reserved-keys-bad) · [Good](#script-no-reserved-keys-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) | Disallow Vue-reserved names as Options API props/data/computed/methods/setup/inject keys | Script |
| [`script/no-reserved-props`](#script-no-reserved-props) | [Bad](#script-no-reserved-props-bad) · [Good](#script-no-reserved-props-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) | Disallow reserved names in a component's props declaration | Script |
| [`script/no-restricted-globals`](#script-no-restricted-globals) | [Bad](#script-no-restricted-globals-bad) · [Good](#script-no-restricted-globals-good) | `error` | _none_ | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) | Disallow references to runtime-environment globals that must go through a typed wrapper | Script |
| [`script/no-restricted-members`](#script-no-restricted-members) | [Bad](#script-no-restricted-members-bad) · [Good](#script-no-restricted-members-good) | `error` | _none_ | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) | Disallow project-configured object.property member accesses | Script |
| [`script/no-side-effects-in-computed-properties`](#script-no-side-effects-in-computed-properties) | [Bad](#script-no-side-effects-in-computed-properties-bad) · [Good](#script-no-side-effects-in-computed-properties-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) | Disallow side effects in Options API computed getters | Script |
| [`script/no-top-level-ref-in-script`](#script-no-top-level-ref-in-script) | [Bad](#script-no-top-level-ref-in-script-bad) · [Good](#script-no-top-level-ref-in-script-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) | Disallow top-level ref/reactive to prevent Cross-Request State Pollution | Script |
| [`script/no-unstable-nested-components`](#script-no-unstable-nested-components) | [Bad](#script-no-unstable-nested-components-bad) · [Good](#script-no-unstable-nested-components-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) | Disallow component definitions inside setup or render functions | Script |
| [`script/no-unused-emit-declarations`](#script-no-unused-emit-declarations) | [Bad](#script-no-unused-emit-declarations-bad) · [Good](#script-no-unused-emit-declarations-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) | Flag declared events that are never emitted | Script |
| [`script/no-use-computed-property-like-method`](#script-no-use-computed-property-like-method) | [Bad](#script-no-use-computed-property-like-method-bad) · [Good](#script-no-use-computed-property-like-method-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) | Disallow calling an Options API computed property like a method | Script |
| [`script/no-with-defaults`](#script-no-with-defaults) | [Bad](#script-no-with-defaults-bad) · [Good](#script-no-with-defaults-good) | `warning` | `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) | Discourage withDefaults in favor of destructuring defaults (Vue 3.5+) | Script |
| [`script/prefer-computed`](#script-prefer-computed) | [Bad](#script-prefer-computed-bad) · [Good](#script-prefer-computed-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) | Prefer computed() for derived reactive state | Script |
| [`script/prefer-define-options`](#script-prefer-define-options) | [Bad](#script-prefer-define-options-bad) · [Good](#script-prefer-define-options-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) | Prefer defineOptions() over a plain &lt;script&gt; that only sets name/inheritAttrs | Script |
| [`script/prefer-import-from-vue`](#script-prefer-import-from-vue) | [Bad](#script-prefer-import-from-vue-bad) · [Good](#script-prefer-import-from-vue-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) | Prefer importing from 'vue' instead of internal packages | Script |
| [`script/prefer-ref-over-reactive`](#script-prefer-ref-over-reactive) | [Bad](#script-prefer-ref-over-reactive-bad) · [Good](#script-prefer-ref-over-reactive-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) | Recommend using ref() over reactive() for state management | Script |
| [`script/prefer-use-attrs`](#script-prefer-use-attrs) | [Bad](#script-prefer-use-attrs-bad) · [Good](#script-prefer-use-attrs-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) | Recommend using useAttrs() over context.attrs | Script |
| [`script/prefer-use-id`](#script-prefer-use-id) | [Bad](#script-prefer-use-id-bad) · [Good](#script-prefer-use-id-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) | Recommend using useId() for generating unique IDs (Vue 3.5+) | Script |
| [`script/prefer-use-slots`](#script-prefer-use-slots) | [Bad](#script-prefer-use-slots-bad) · [Good](#script-prefer-use-slots-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) | Recommend using useSlots() over context.slots | Script |
| [`script/prefer-use-template-ref`](#script-prefer-use-template-ref) | [Bad](#script-prefer-use-template-ref-bad) · [Good](#script-prefer-use-template-ref-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) | Recommend useTemplateRef over ref for template references (Vue 3.5+) | Script |
| [`script/require-default-prop`](#script-require-default-prop) | [Bad](#script-require-default-prop-bad) · [Good](#script-require-default-prop-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) | Require a default value for every optional, non-Boolean prop | Script |
| [`script/require-explicit-emits`](#script-require-explicit-emits) | [Bad](#script-require-explicit-emits-bad) · [Good](#script-require-explicit-emits-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) | Require emitted events to be declared in defineEmits or the emits option | Script |
| [`script/require-explicit-slots`](#script-require-explicit-slots) | [Bad](#script-require-explicit-slots-bad) · [Good](#script-require-explicit-slots-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) | Require slots consumed via useSlots() to be explicitly typed with defineSlots&lt;...&gt;() | Script |
| [`script/require-function-return-type`](#script-require-function-return-type) | [Bad](#script-require-function-return-type-bad) · [Good](#script-require-function-return-type-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) | Require return type annotations on functions | Script |
| [`script/require-prop-type-constructor`](#script-require-prop-type-constructor) | [Bad](#script-require-prop-type-constructor-bad) · [Good](#script-require-prop-type-constructor-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L59) | Require prop `type` values to be constructors rather than string literals | Script |
| [`script/require-prop-types`](#script-require-prop-types) | [Bad](#script-require-prop-types-bad) · [Good](#script-require-prop-types-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) | Require every prop to declare a type | Script |
| [`script/require-symbol-provide`](#script-require-symbol-provide) | [Bad](#script-require-symbol-provide-bad) · [Good](#script-require-symbol-provide-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) | Recommend using Symbol as injection key for provide/inject | Script |
| [`script/require-typed-object-prop`](#script-require-typed-object-prop) | [Bad](#script-require-typed-object-prop-bad) · [Good](#script-require-typed-object-prop-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) | Require an explicit type on a prop whose runtime type is `Object` or `Array` | Script |
| [`script/require-typed-ref`](#script-require-typed-ref) | [Bad](#script-require-typed-ref-bad) · [Good](#script-require-typed-ref-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) | Require an explicit type argument on a ref() initialized with no value, null, or undefined | Script |
| [`script/require-valid-default-prop`](#script-require-valid-default-prop) | [Bad](#script-require-valid-default-prop-bad) · [Good](#script-require-valid-default-prop-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) | Require a prop's default value to be valid for its declared type | Script |
| [`script/return-in-computed-property`](#script-return-in-computed-property) | [Bad](#script-return-in-computed-property-bad) · [Good](#script-return-in-computed-property-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) | Require a return value in every computed getter | Script |
| [`script/return-in-emits-validator`](#script-return-in-emits-validator) | [Bad](#script-return-in-emits-validator-bad) · [Good](#script-return-in-emits-validator-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) | Require a return value in every Options API emits validator | Script |
| [`script/valid-define-emits`](#script-valid-define-emits) | [Bad](#script-valid-define-emits-bad) · [Good](#script-valid-define-emits-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) | Enforce valid defineEmits() usage (no type+runtime args, no local references, single call) | Script |
| [`script/valid-define-options`](#script-valid-define-options) | [Bad](#script-valid-define-options-bad) · [Good](#script-valid-define-options-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) | Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots) | Script |
| [`script/valid-define-props`](#script-valid-define-props) | [Bad](#script-valid-define-props-bad) · [Good](#script-valid-define-props-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) | Enforce valid defineProps() usage (single call, not both type and runtime args, no local references) | Script |
| [`script/valid-next-tick`](#script-valid-next-tick) | [Bad](#script-valid-next-tick-bad) · [Good](#script-valid-next-tick-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) | Require the result of a nextTick() call to be awaited, chained, or given a callback | Script |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [Bad](#nuxt-no-nuxt-config-test-key-bad) · [Good](#nuxt-no-nuxt-config-test-key-good) | `error` | `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) | Disallow setting `test` key in Nuxt config | Nuxt |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [Bad](#nuxt-no-page-meta-runtime-values-bad) · [Good](#nuxt-no-page-meta-runtime-values-good) | `error` | `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) | Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup | Nuxt |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [Bad](#nuxt-nuxt-config-keys-order-bad) · [Good](#nuxt-nuxt-config-keys-order-good) | `error` | `nuxt` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) | Prefer recommended order of Nuxt config properties | Nuxt |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [Bad](#nuxt-prefer-import-meta-bad) · [Good](#nuxt-prefer-import-meta-good) | `error` | `nuxt` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) | Prefer using `import.meta.*` over `process.*` | Nuxt |

## Project rules and analyzer contracts (66)

These project entries supplement the 251 single-file catalog entries above. Each example includes its component/project context or an explicit tracked-graph scenario. CLI, experimental library producers, and contracts without a producer have different support boundaries; see the [cross-file overview](./cross-file.md).

| Rule / code | Examples | Current support |
| --- | --- | --- |
| [`ecosystem/vue-router-unknown-route`](#ecosystem-vue-router-unknown-route) | [Bad](#ecosystem-vue-router-unknown-route-bad) · [Good](#ecosystem-vue-router-unknown-route-good) | CLI project pass |
| [`ecosystem/vue-router-extra-param`](#ecosystem-vue-router-extra-param) | [Bad](#ecosystem-vue-router-extra-param-bad) · [Good](#ecosystem-vue-router-extra-param-good) | CLI project pass |
| [`ecosystem/vue-router-param-type`](#ecosystem-vue-router-param-type) | [Bad](#ecosystem-vue-router-param-type-bad) · [Good](#ecosystem-vue-router-param-type-good) | CLI project pass |
| [`ecosystem/vue-router-missing-param`](#ecosystem-vue-router-missing-param) | [Bad](#ecosystem-vue-router-missing-param-bad) · [Good](#ecosystem-vue-router-missing-param-good) | CLI project pass |
| [`html/cross-component-nesting`](#html-cross-component-nesting) | [Bad](#html-cross-component-nesting-bad) · [Good](#html-cross-component-nesting-good) | CLI project pass |
| [`vue/cross-file-attrs-fallthrough`](#vue-cross-file-attrs-fallthrough) | [Bad](#vue-cross-file-attrs-fallthrough-bad) · [Good](#vue-cross-file-attrs-fallthrough-good) | CLI project pass |
| [`vize:croquis/cf/array-mutation`](#vize-croquis-cf-array-mutation) | [Bad](#vize-croquis-cf-array-mutation-bad) · [Good](#vize-croquis-cf-array-mutation-good) | Contract only; no producer |
| [`vize:croquis/cf/async-boundary`](#vize-croquis-cf-async-boundary) | [Bad](#vize-croquis-cf-async-boundary-bad) · [Good](#vize-croquis-cf-async-boundary-good) | CLI project pass |
| [`vize:croquis/cf/async-no-suspense`](#vize-croquis-cf-async-no-suspense) | [Bad](#vize-croquis-cf-async-no-suspense-bad) · [Good](#vize-croquis-cf-async-no-suspense-good) | Library producer; required source fact missing |
| [`vize:croquis/cf/browser-api-ssr`](#vize-croquis-cf-browser-api-ssr) | [Bad](#vize-croquis-cf-browser-api-ssr-bad) · [Good](#vize-croquis-cf-browser-api-ssr-good) | CLI project pass |
| [`vize:croquis/cf/circular-dep`](#vize-croquis-cf-circular-dep) | [Bad](#vize-croquis-cf-circular-dep-bad) · [Good](#vize-croquis-cf-circular-dep-good) | Contract only; no producer |
| [`vize:croquis/cf/circular-reactive-dependency`](#vize-croquis-cf-circular-reactive-dependency) | [Bad](#vize-croquis-cf-circular-reactive-dependency-bad) · [Good](#vize-croquis-cf-circular-reactive-dependency-good) | CLI: tracked-graph scenario |
| [`vize:croquis/cf/closure-captures-reactive`](#vize-croquis-cf-closure-captures-reactive) | [Bad](#vize-croquis-cf-closure-captures-reactive-bad) · [Good](#vize-croquis-cf-closure-captures-reactive-good) | Contract only; no producer |
| [`vize:croquis/cf/composable-outside-setup`](#vize-croquis-cf-composable-outside-setup) | [Bad](#vize-croquis-cf-composable-outside-setup-bad) · [Good](#vize-croquis-cf-composable-outside-setup-good) | Contract only; no producer |
| [`vize:croquis/cf/computed-side-effects`](#vize-croquis-cf-computed-side-effects) | [Bad](#vize-croquis-cf-computed-side-effects-bad) · [Good](#vize-croquis-cf-computed-side-effects-good) | Contract only; no producer |
| [`vize:croquis/cf/deep-import`](#vize-croquis-cf-deep-import) | [Bad](#vize-croquis-cf-deep-import-bad) · [Good](#vize-croquis-cf-deep-import-good) | Contract only; no producer |
| [`vize:croquis/cf/destructuring-breaks-reactivity`](#vize-croquis-cf-destructuring-breaks-reactivity) | [Bad](#vize-croquis-cf-destructuring-breaks-reactivity-bad) · [Good](#vize-croquis-cf-destructuring-breaks-reactivity-good) | CLI project pass |
| [`vize:croquis/cf/di-outside-setup`](#vize-croquis-cf-di-outside-setup) | [Bad](#vize-croquis-cf-di-outside-setup-bad) · [Good](#vize-croquis-cf-di-outside-setup-good) | Contract only; no producer |
| [`vize:croquis/cf/dom-access-without-next-tick`](#vize-croquis-cf-dom-access-without-next-tick) | [Bad](#vize-croquis-cf-dom-access-without-next-tick-bad) · [Good](#vize-croquis-cf-dom-access-without-next-tick-good) | Contract only; no producer |
| [`vize:croquis/cf/duplicate-id`](#vize-croquis-cf-duplicate-id) | [Bad](#vize-croquis-cf-duplicate-id-bad) · [Good](#vize-croquis-cf-duplicate-id-good) | CLI project pass |
| [`vize:croquis/cf/event-listener-leak`](#vize-croquis-cf-event-listener-leak) | [Bad](#vize-croquis-cf-event-listener-leak-bad) · [Good](#vize-croquis-cf-event-listener-leak-good) | Contract only; no producer |
| [`vize:croquis/cf/event-modifier`](#vize-croquis-cf-event-modifier) | [Bad](#vize-croquis-cf-event-modifier-bad) · [Good](#vize-croquis-cf-event-modifier-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/hydration-risk`](#vize-croquis-cf-hydration-risk) | [Bad](#vize-croquis-cf-hydration-risk-bad) · [Good](#vize-croquis-cf-hydration-risk-good) | CLI project pass |
| [`vize:croquis/cf/inherit-attrs-unused`](#vize-croquis-cf-inherit-attrs-unused) | [Bad](#vize-croquis-cf-inherit-attrs-unused-bad) · [Good](#vize-croquis-cf-inherit-attrs-unused-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/inject-without-symbol`](#vize-croquis-cf-inject-without-symbol) | [Bad](#vize-croquis-cf-inject-without-symbol-bad) · [Good](#vize-croquis-cf-inject-without-symbol-good) | CLI project pass |
| [`vize:croquis/cf/injected-async-mutation-race`](#vize-croquis-cf-injected-async-mutation-race) | [Bad](#vize-croquis-cf-injected-async-mutation-race-bad) · [Good](#vize-croquis-cf-injected-async-mutation-race-good) | CLI project pass |
| [`vize:croquis/cf/lifecycle-outside-setup`](#vize-croquis-cf-lifecycle-outside-setup) | [Bad](#vize-croquis-cf-lifecycle-outside-setup-bad) · [Good](#vize-croquis-cf-lifecycle-outside-setup-good) | Contract only; no producer |
| [`vize:croquis/cf/lifecycle-without-cleanup`](#vize-croquis-cf-lifecycle-without-cleanup) | [Bad](#vize-croquis-cf-lifecycle-without-cleanup-bad) · [Good](#vize-croquis-cf-lifecycle-without-cleanup-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/missing-required-prop`](#vize-croquis-cf-missing-required-prop) | [Bad](#vize-croquis-cf-missing-required-prop-bad) · [Good](#vize-croquis-cf-missing-required-prop-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/missing-suspense`](#vize-croquis-cf-missing-suspense) | [Bad](#vize-croquis-cf-missing-suspense-bad) · [Good](#vize-croquis-cf-missing-suspense-good) | Contract only; no producer |
| [`vize:croquis/cf/module-scope-reactive`](#vize-croquis-cf-module-scope-reactive) | [Bad](#vize-croquis-cf-module-scope-reactive-bad) · [Good](#vize-croquis-cf-module-scope-reactive-good) | Contract only; no producer |
| [`vize:croquis/cf/multi-root-attrs`](#vize-croquis-cf-multi-root-attrs) | [Bad](#vize-croquis-cf-multi-root-attrs-bad) · [Good](#vize-croquis-cf-multi-root-attrs-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/mutated-after-escape`](#vize-croquis-cf-mutated-after-escape) | [Bad](#vize-croquis-cf-mutated-after-escape-bad) · [Good](#vize-croquis-cf-mutated-after-escape-good) | Contract only; no producer |
| [`vize:croquis/cf/non-reactive-provide`](#vize-croquis-cf-non-reactive-provide) | [Bad](#vize-croquis-cf-non-reactive-provide-bad) · [Good](#vize-croquis-cf-non-reactive-provide-good) | CLI project pass |
| [`vize:croquis/cf/non-unique-id`](#vize-croquis-cf-non-unique-id) | [Bad](#vize-croquis-cf-non-unique-id-bad) · [Good](#vize-croquis-cf-non-unique-id-good) | CLI project pass |
| [`vize:croquis/cf/object-identity-comparison`](#vize-croquis-cf-object-identity-comparison) | [Bad](#vize-croquis-cf-object-identity-comparison-bad) · [Good](#vize-croquis-cf-object-identity-comparison-good) | Contract only; no producer |
| [`vize:croquis/cf/pinia-getter`](#vize-croquis-cf-pinia-getter) | [Bad](#vize-croquis-cf-pinia-getter-bad) · [Good](#vize-croquis-cf-pinia-getter-good) | Contract only; no producer |
| [`vize:croquis/cf/prop-type-mismatch`](#vize-croquis-cf-prop-type-mismatch) | [Bad](#vize-croquis-cf-prop-type-mismatch-bad) · [Good](#vize-croquis-cf-prop-type-mismatch-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/provide-inject-type`](#vize-croquis-cf-provide-inject-type) | [Bad](#vize-croquis-cf-provide-inject-type-bad) · [Good](#vize-croquis-cf-provide-inject-type-good) | CLI project pass |
| [`vize:croquis/cf/provide-without-symbol`](#vize-croquis-cf-provide-without-symbol) | [Bad](#vize-croquis-cf-provide-without-symbol-bad) · [Good](#vize-croquis-cf-provide-without-symbol-good) | CLI project pass |
| [`vize:croquis/cf/reactive-export`](#vize-croquis-cf-reactive-export) | [Bad](#vize-croquis-cf-reactive-export-bad) · [Good](#vize-croquis-cf-reactive-export-good) | Contract only; no producer |
| [`vize:croquis/cf/reactivity-outside-setup`](#vize-croquis-cf-reactivity-outside-setup) | [Bad](#vize-croquis-cf-reactivity-outside-setup-bad) · [Good](#vize-croquis-cf-reactivity-outside-setup-good) | Contract only; no producer |
| [`vize:croquis/cf/reassignment-breaks-reactivity`](#vize-croquis-cf-reassignment-breaks-reactivity) | [Bad](#vize-croquis-cf-reassignment-breaks-reactivity-bad) · [Good](#vize-croquis-cf-reassignment-breaks-reactivity-good) | CLI project pass |
| [`vize:croquis/cf/reference-escapes-scope`](#vize-croquis-cf-reference-escapes-scope) | [Bad](#vize-croquis-cf-reference-escapes-scope-bad) · [Good](#vize-croquis-cf-reference-escapes-scope-good) | Contract only; no producer |
| [`vize:croquis/cf/setup-context-violation`](#vize-croquis-cf-setup-context-violation) | [Bad](#vize-croquis-cf-setup-context-violation-bad) · [Good](#vize-croquis-cf-setup-context-violation-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/shallow-deep-access`](#vize-croquis-cf-shallow-deep-access) | [Bad](#vize-croquis-cf-shallow-deep-access-bad) · [Good](#vize-croquis-cf-shallow-deep-access-good) | Contract only; no producer |
| [`vize:croquis/cf/spread-breaks-reactivity`](#vize-croquis-cf-spread-breaks-reactivity) | [Bad](#vize-croquis-cf-spread-breaks-reactivity-bad) · [Good](#vize-croquis-cf-spread-breaks-reactivity-good) | CLI project pass |
| [`vize:croquis/cf/suspense-no-fallback`](#vize-croquis-cf-suspense-no-fallback) | [Bad](#vize-croquis-cf-suspense-no-fallback-bad) · [Good](#vize-croquis-cf-suspense-no-fallback-good) | Contract only; no producer |
| [`vize:croquis/cf/template-ref-timing`](#vize-croquis-cf-template-ref-timing) | [Bad](#vize-croquis-cf-template-ref-timing-bad) · [Good](#vize-croquis-cf-template-ref-timing-good) | Contract only; no producer |
| [`vize:croquis/cf/toraw-mutation`](#vize-croquis-cf-toraw-mutation) | [Bad](#vize-croquis-cf-toraw-mutation-bad) · [Good](#vize-croquis-cf-toraw-mutation-good) | Contract only; no producer |
| [`vize:croquis/cf/uncaught-error`](#vize-croquis-cf-uncaught-error) | [Bad](#vize-croquis-cf-uncaught-error-bad) · [Good](#vize-croquis-cf-uncaught-error-good) | CLI project pass |
| [`vize:croquis/cf/undeclared-emit`](#vize-croquis-cf-undeclared-emit) | [Bad](#vize-croquis-cf-undeclared-emit-bad) · [Good](#vize-croquis-cf-undeclared-emit-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/undeclared-prop`](#vize-croquis-cf-undeclared-prop) | [Bad](#vize-croquis-cf-undeclared-prop-bad) · [Good](#vize-croquis-cf-undeclared-prop-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/undefined-slot`](#vize-croquis-cf-undefined-slot) | [Bad](#vize-croquis-cf-undefined-slot-bad) · [Good](#vize-croquis-cf-undefined-slot-good) | Contract only; no producer |
| [`vize:croquis/cf/unhandled-event`](#vize-croquis-cf-unhandled-event) | [Bad](#vize-croquis-cf-unhandled-event-bad) · [Good](#vize-croquis-cf-unhandled-event-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/unmatched-inject`](#vize-croquis-cf-unmatched-inject) | [Bad](#vize-croquis-cf-unmatched-inject-bad) · [Good](#vize-croquis-cf-unmatched-inject-good) | CLI project pass |
| [`vize:croquis/cf/unmatched-listener`](#vize-croquis-cf-unmatched-listener) | [Bad](#vize-croquis-cf-unmatched-listener-bad) · [Good](#vize-croquis-cf-unmatched-listener-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/unregistered-component`](#vize-croquis-cf-unregistered-component) | [Bad](#vize-croquis-cf-unregistered-component-bad) · [Good](#vize-croquis-cf-unregistered-component-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/unresolved-import`](#vize-croquis-cf-unresolved-import) | [Bad](#vize-croquis-cf-unresolved-import-bad) · [Good](#vize-croquis-cf-unresolved-import-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/unused-attrs`](#vize-croquis-cf-unused-attrs) | [Bad](#vize-croquis-cf-unused-attrs-bad) · [Good](#vize-croquis-cf-unused-attrs-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/unused-emit`](#vize-croquis-cf-unused-emit) | [Bad](#vize-croquis-cf-unused-emit-bad) · [Good](#vize-croquis-cf-unused-emit-good) | Experimental library producer; not this CLI code |
| [`vize:croquis/cf/unused-provide`](#vize-croquis-cf-unused-provide) | [Bad](#vize-croquis-cf-unused-provide-bad) · [Good](#vize-croquis-cf-unused-provide-good) | CLI project pass |
| [`vize:croquis/cf/value-extraction-breaks-reactivity`](#vize-croquis-cf-value-extraction-breaks-reactivity) | [Bad](#vize-croquis-cf-value-extraction-breaks-reactivity-bad) · [Good](#vize-croquis-cf-value-extraction-breaks-reactivity-good) | CLI project pass |
| [`vize:croquis/cf/watch-can-be-computed`](#vize-croquis-cf-watch-can-be-computed) | [Bad](#vize-croquis-cf-watch-can-be-computed-bad) · [Good](#vize-croquis-cf-watch-can-be-computed-good) | Contract only; no producer |
| [`vize:croquis/cf/watcheffect-async`](#vize-croquis-cf-watcheffect-async) | [Bad](#vize-croquis-cf-watcheffect-async-bad) · [Good](#vize-croquis-cf-watcheffect-async-good) | CLI project pass |
| [`vize:croquis/cf/watcher-outside-setup`](#vize-croquis-cf-watcher-outside-setup) | [Bad](#vize-croquis-cf-watcher-outside-setup-bad) · [Good](#vize-croquis-cf-watcher-outside-setup-good) | Contract only; no producer |

## Single-file examples

<span id="petite-vue-no-unsupported-directive"></span>

### `petite-vue/no-unsupported-directive`

Disallow directives that petite-vue does not support

[Bad](#petite-vue-no-unsupported-directive-bad) · [Good](#petite-vue-no-unsupported-directive-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-no-unsupported-directive-bad"></span>

**Bad**

`v-memo`, `v-slot:header`, and the custom `v-my-directive` are absent from petite-vue’s supported directive list. The petite-vue script marks this HTML as the relevant dialect.

```html
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-no-unsupported-directive-good"></span>

**Good**

The replacement uses supported `v-scope`, `v-effect`, `v-if`, `v-bind`, and `v-on` syntax instead of relying on unsupported directives.

```html
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [All rules](all.md)

<span id="petite-vue-valid-v-effect"></span>

### `petite-vue/valid-v-effect`

Require v-effect to have a non-empty expression

[Bad](#petite-vue-valid-v-effect-bad) · [Good](#petite-vue-valid-v-effect-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-effect": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-effect-bad"></span>

**Bad**

Each `v-effect` has no executable expression: its value is missing, empty, or only whitespace.

```html
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-effect-good"></span>

**Good**

Both `v-effect` values contain an expression: one updates `el.textContent`, and the other increments `count`. This rule checks for a nonempty expression, not the effect’s business logic.

```html
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) · [All rules](all.md)

<span id="petite-vue-valid-v-scope"></span>

### `petite-vue/valid-v-scope`

Require v-scope to bind an object literal

[Bad](#petite-vue-valid-v-scope-bad) · [Good](#petite-vue-valid-v-scope-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-scope": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-scope-bad"></span>

**Bad**

The four nonempty `v-scope` values are an identifier, a call, arithmetic, and a number; none parses as an object literal.

```html
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-scope-good"></span>

**Good**

A valueless `v-scope` uses the root scope. The other values are object literals, including the parenthesized object, which the rule accepts.

```html
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) · [All rules](all.md)

<span id="vue-multi-word-component-names"></span>

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

<span id="vue-no-child-content"></span>

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

```vue
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**Good**

Removing the child text leaves v-text as the single source of paragraph content.

```vue
<template>
  <p v-text="message" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [All rules](all.md)

<span id="vue-no-deprecated-filter"></span>

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

```vue
<template>
{{ message | capitalize }}
</template>
```

<span id="vue-no-deprecated-filter-good"></span>

**Good**

Calling capitalize(message) applies the transformation as an ordinary expression.

```vue
<template>
{{ capitalize(message) }}
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) · [All rules](all.md)

<span id="vue-no-deprecated-functional-template"></span>

### `vue/no-deprecated-functional-template`

Disallow the `functional` attribute on the SFC `<template>`

[Bad](#vue-no-deprecated-functional-template-bad) · [Good](#vue-no-deprecated-functional-template-good)

Default severity: `error`  
Presets: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
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

```vue
<template functional>
<div>{{ props.msg }}</div>
</template>
```

<span id="vue-no-deprecated-functional-template-good"></span>

**Good**

The ordinary template omits functional and reads the component binding msg directly.

```vue
<template>
<div>{{ msg }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) · [All rules](all.md)

<span id="vue-no-deprecated-html-element-is"></span>

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

```vue
<template>
<div is="MyComponent" />
</template>
```

<span id="vue-no-deprecated-html-element-is-good"></span>

**Good**

A dynamic component uses :is; the native-element spelling explicitly uses the vue: prefix.

```vue
<template>
<component :is="MyComponent" />
<div is="vue:MyComponent" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) · [All rules](all.md)

<span id="vue-no-deprecated-inline-template"></span>

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

```vue
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

<span id="vue-no-deprecated-inline-template-good"></span>

**Good**

The same content is passed normally without the inline-template attribute.

```vue
<template>
<Card><p>Details</p></Card>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) · [All rules](all.md)

<span id="vue-no-deprecated-router-link-tag-prop"></span>

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

```vue
<template>
<router-link to="/home" tag="button">Home</router-link>
</template>
```

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

**Good**

The slot provides navigate to an explicitly authored button.

```vue
<template>
<router-link to="/home" v-slot="{ navigate }">
<button @click="navigate">Home</button>
</router-link>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [All rules](all.md)

<span id="vue-no-deprecated-scope-attribute"></span>

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

```vue
<template>
<Card><template scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-scope-attribute-good"></span>

**Good**

The default-slot directive declares the same props binding through current slot syntax.

```vue
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) · [All rules](all.md)

<span id="vue-no-deprecated-slot-attribute"></span>

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

```vue
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

```vue
<template>
<Foo>
<template v-slot:header><h1>Title</h1></template>
</Foo>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) · [All rules](all.md)

<span id="vue-no-deprecated-slot-scope-attribute"></span>

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

```vue
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

**Good**

The #default directive receives those props without slot-scope.

```vue
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) · [All rules](all.md)

<span id="vue-no-deprecated-v-bind-sync"></span>

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

```vue
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

<span id="vue-no-deprecated-v-bind-sync-good"></span>

**Good**

Use an ordinary one-way title binding or v-model:title when an update channel is required.

```vue
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) · [All rules](all.md)

<span id="vue-no-deprecated-v-on-native-modifier"></span>

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

```vue
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

**Good**

The handlers omit .native and preserve other event modifiers such as .stop.

```vue
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) · [All rules](all.md)

<span id="vue-no-deprecated-v-on-number-modifiers"></span>

### `vue/no-deprecated-v-on-number-modifiers`

Disallow deprecated numeric `keyCode` modifiers on `v-on`

[Bad](#vue-no-deprecated-v-on-number-modifiers-bad) · [Good](#vue-no-deprecated-v-on-number-modifiers-good)

Default severity: `error`  
Presets: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
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

```vue
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

**Good**

The handlers use the named enter and esc key modifiers.

```vue
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [All rules](all.md)

<span id="vue-no-dupe-v-else-if"></span>

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

```vue
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**Good**

The second branch tests loading, a distinct state that can reach the else-if.

```vue
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [All rules](all.md)

<span id="vue-no-duplicate-attributes"></span>

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

```vue
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**Good**

Both class tokens appear in a single class attribute.

```vue
<template>
  <button class="primary large">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [All rules](all.md)

<span id="vue-no-multiple-template-root"></span>

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

```vue
<template>
<p>First</p>
<p>Second</p>
</template>
```

<span id="vue-no-multiple-template-root-good"></span>

**Good**

A section wraps the paragraphs into one root; enable this convention only when a single-root contract is intended.

```vue
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) · [All rules](all.md)

<span id="vue-no-mutating-props"></span>

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

```vue
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**Good**

The component emits update:count with the next value, leaving the parent responsible for updating the prop.

```vue
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

<span id="vue-no-reserved-component-names"></span>

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

```vue
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**Good**

AppButton is an application component name and does not reuse the native button name.

```vue
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

<span id="vue-no-template-key"></span>

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

```vue
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**Good**

The key belongs to a template v-for iteration, where it identifies each repeated fragment.

```vue
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [All rules](all.md)

<span id="vue-no-textarea-mustache"></span>

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

```vue
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**Good**

v-model binds the editable textarea value to message.

```vue
<template>
  <textarea v-model="message"></textarea>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [All rules](all.md)

<span id="vue-no-unused-components"></span>

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

```vue
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

```vue
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) · [All rules](all.md)

<span id="vue-no-unused-vars"></span>

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

```vue
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

```vue
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

<span id="vue-no-use-v-if-with-v-for"></span>

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

```vue
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**Good**

A computed collection filters the visible items before the template iterates over them.

```vue
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

<span id="vue-no-useless-template-attributes"></span>

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

```vue
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**Good**

The class moves to the paragraph that actually renders while v-if stays on the structural template.

```vue
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [All rules](all.md)

<span id="vue-no-v-for-template-key-on-child"></span>

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

```vue
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

<span id="vue-no-v-for-template-key-on-child-good"></span>

**Good**

The key moves to template v-for, identifying the complete repeated fragment.

```vue
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) · [All rules](all.md)

<span id="vue-no-v-html"></span>

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

```vue
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**Good**

Mustache interpolation displays content as escaped text instead of injecting HTML.

```vue
<template>
  <article>{{ content }}</article>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [All rules](all.md)

<span id="vue-no-v-text-v-html-on-component"></span>

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

```vue
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**Good**

Native HTML targets can receive the directives; MyComponent receives its content through the default slot.

```vue
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [All rules](all.md)

<span id="vue-permitted-contents"></span>

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

```vue
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

```vue
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [All rules](all.md)

<span id="vue-require-component-is"></span>

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

```vue
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**Good**

`:is="currentComponent"` supplies the component selection; the binding may change at runtime.

```vue
<template>
  <component :is="currentComponent" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [All rules](all.md)

<span id="vue-require-toggle-inside-transition"></span>

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

```vue
<template>
<transition>
<div>content</div>
</transition>
</template>
```

<span id="vue-require-toggle-inside-transition-good"></span>

**Good**

`v-if="show"` changes whether the child exists, giving the transition an enter/leave boundary.

```vue
<template>
<transition>
<div v-if="show">content</div>
</transition>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) · [All rules](all.md)

<span id="vue-require-v-for-key"></span>

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

```vue
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**Good**

`:key="item.id"` gives each repeated node the item's identity rather than its current position.

```vue
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) · [All rules](all.md)

<span id="vue-use-v-on-exact"></span>

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

```vue
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**Good**

`.exact` limits the ordinary click handler to clicks without modifier keys; the Ctrl-specific handler remains separate.

```vue
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

<span id="vue-valid-attribute-name"></span>

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

```vue
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**Good**

`my-attr` is a well-formed attribute name, so the template parser can read the attribute and its value.

```vue
<template>
<div my-attr="value"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [All rules](all.md)

<span id="vue-valid-template-root"></span>

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

```vue
<template>
<template>content</template>
</template>
```

<span id="vue-valid-template-root-good"></span>

**Good**

The `<div>` is a renderable root element. This example does not impose a universal single-root restriction on Vue 3 fragments.

```vue
<template>
<div>content</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [All rules](all.md)

<span id="vue-valid-v-bind"></span>

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

```vue
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

<span id="vue-valid-v-bind-good"></span>

**Good**

Provide an attribute and expression, bind an object, or use Vue 3.4+ same-name shorthand such as `:loading`.

```vue
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) · [All rules](all.md)

<span id="vue-valid-v-cloak"></span>

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

```vue
<template>
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

<span id="vue-valid-v-cloak-good"></span>

**Good**

Use bare `v-cloak`; CSS can hide the element until Vue removes that attribute after mounting.

```vue
<template>
<div v-cloak></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) · [All rules](all.md)

<span id="vue-valid-v-else"></span>

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

```vue
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

<span id="vue-valid-v-else-good"></span>

**Good**

Place bare `v-else` immediately after the corresponding `v-if` branch.

```vue
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) · [All rules](all.md)

<span id="vue-valid-v-for"></span>

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

```vue
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

<span id="vue-valid-v-for-good"></span>

**Good**

Use `item in items` or `(item, index) of items` with a complete iteration expression and the shown keys.

```vue
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) · [All rules](all.md)

<span id="vue-valid-v-html"></span>

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

```vue
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

<span id="vue-valid-v-html-good"></span>

**Good**

`v-html="html"` supplies a valid expression. Syntax validity does not sanitize HTML or make untrusted content safe.

```vue
<template>
<div v-html="html"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) · [All rules](all.md)

<span id="vue-valid-v-if"></span>

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

```vue
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

<span id="vue-valid-v-if-good"></span>

**Good**

Each `v-if` has a nonempty condition such as `ready` or `count > 0`, without an incompatible else directive.

```vue
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) · [All rules](all.md)

<span id="vue-valid-v-memo"></span>

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

```vue
<template>
  <div v-memo></div>
</template>
```

<span id="vue-valid-v-memo-good"></span>

**Good**

`v-memo="[valueA, valueB]"` supplies the dependency array used for memoization.

```vue
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [All rules](all.md)

<span id="vue-valid-v-model"></span>

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

```vue
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

<span id="vue-valid-v-model-good"></span>

**Good**

Bind the input, select, textarea, or custom component to the shown writable variables.

```vue
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) · [All rules](all.md)

<span id="vue-valid-v-on"></span>

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

```vue
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

<span id="vue-valid-v-on-good"></span>

**Good**

Use an event with its handler, or pass a listener object to argument-free `v-on`.

```vue
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [All rules](all.md)

<span id="vue-valid-v-once"></span>

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

```vue
<template>
<div v-once="foo"></div>
<div v-once:arg></div>
<div v-once.mod></div>
</template>
```

<span id="vue-valid-v-once-good"></span>

**Good**

Bare `v-once` marks the subtree for one-time rendering without unsupported syntax.

```vue
<template>
<div v-once></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) · [All rules](all.md)

<span id="vue-valid-v-show"></span>

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

```vue
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

<span id="vue-valid-v-show-good"></span>

**Good**

Apply the visibility expression to a rendered element such as `<div>`.

```vue
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [All rules](all.md)

<span id="vue-valid-v-slot"></span>

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

```vue
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

<span id="vue-valid-v-slot-good"></span>

**Good**

Declare a component's default slot on that component, or its named slot on a child `<template #header>`.

```vue
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) · [All rules](all.md)

<span id="vue-valid-v-text"></span>

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

```vue
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

<span id="vue-valid-v-text-good"></span>

**Good**

`v-text="msg"` is syntactically valid. The separate `vue/no-v-text` style rule can still prefer interpolation.

```vue
<template>
<div v-text="msg"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [All rules](all.md)

<span id="vue-attribute-hyphenation"></span>

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

```vue
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**Good**

The first-name spelling follows the configured hyphenated component-attribute convention.

```vue
<template>
<UserCard first-name="Ada" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [All rules](all.md)

<span id="vue-component-definition-name-casing"></span>

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

<span id="vue-html-quotes"></span>

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

```vue
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**Good**

Both ordinary attributes and directive expressions use double quotes.

```vue
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [All rules](all.md)

<span id="vue-html-self-closing"></span>

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

```vue
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**Good**

The component and void elements use self-closing syntax; a div with content retains its closing tag.

```vue
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

<span id="vue-mustache-interpolation-spacing"></span>

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

```vue
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**Good**

Spaces separate the expression from both opening and closing mustache delimiters.

```vue
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [All rules](all.md)

<span id="vue-no-multi-spaces"></span>

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

```vue
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**Good**

Single spaces separate the same attributes.

```vue
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [All rules](all.md)

<span id="vue-no-template-shadow"></span>

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

```vue
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**Good**

The inner loop declares child, leaving item available for the outer row and child for the nested row.

```vue
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [All rules](all.md)

<span id="vue-no-unused-properties"></span>

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

```vue
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

<span id="vue-prop-name-casing"></span>

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

```vue
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**Good**

The declaration and its template reference use the camelCase name userName.

```vue
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [All rules](all.md)

<span id="vue-v-bind-style"></span>

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

```vue
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**Good**

`:class` retains the same expression with the required shorthand; this rule concerns spelling rather than the value's type.

```vue
<template>
  <div :class="panelClass"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [All rules](all.md)

<span id="vue-v-on-style"></span>

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

```vue
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**Good**

`@click` keeps the same handler while using the configured shorthand.

```vue
<template>
  <div @click="handleClick"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [All rules](all.md)

<span id="vue-v-slot-style"></span>

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

```vue
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

```vue
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [All rules](all.md)

<span id="ssr-no-browser-globals-in-ssr"></span>

### `ssr/no-browser-globals-in-ssr`

Disallow browser-only globals in SSR context

[Bad](#ssr-no-browser-globals-in-ssr-bad) · [Good](#ssr-no-browser-globals-in-ssr-good)

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
        "ssr/no-browser-globals-in-ssr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

**Bad**

Setup reads `window.innerWidth` immediately, although `window` does not exist when the component runs on the server.

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

<span id="ssr-no-browser-globals-in-ssr-good"></span>

**Good**

The initial width is a server-safe ref value, and the browser access moves into `onMounted`, which runs on the client rather than during SSR setup.

```vue
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) · [All rules](all.md)

<span id="ssr-no-hydration-mismatch"></span>

### `ssr/no-hydration-mismatch`

Disallow non-deterministic values that cause hydration mismatch

[Bad](#ssr-no-hydration-mismatch-bad) · [Good](#ssr-no-hydration-mismatch-good)

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
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-hydration-mismatch-bad"></span>

**Bad**

The template evaluates `Math.random()` during rendering, so the server and client can produce different text for the same paragraph.

```vue
<template>
  <p>{{ Math.random() }}</p>
</template>
```

<span id="ssr-no-hydration-mismatch-good"></span>

**Good**

The paragraph renders the stable `seed` state instead of a fresh random result. In this Nuxt-style example, `useState` supplies the shared state and the initializer is the constant `"stable"`.

```vue
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [All rules](all.md)

<span id="vue-a11y-img-alt"></span>

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

```vue
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

<span id="vue-a11y-img-alt-good"></span>

**Good**

Informative images get descriptive alt text, decoration gets an empty alt, and the dynamic image binds its description.

```vue
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

<span id="vue-attribute-order"></span>

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

```vue
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**Good**

v-if comes first, followed by id and then the event handler, following the rule ordering.

```vue
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [All rules](all.md)

<span id="vue-component-name-in-template-casing"></span>

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

```vue
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

```vue
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

<span id="vue-html-button-has-type"></span>

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

```vue
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

<span id="vue-html-button-has-type-good"></span>

**Good**

Buttons specify button, submit, or reset; a bound type is treated as dynamic.

```vue
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) · [All rules](all.md)

<span id="vue-max-template-complexity"></span>

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

```vue
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

```vue
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) · [All rules](all.md)

<span id="vue-no-array-index-key"></span>

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

```vue
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

<span id="vue-no-array-index-key-good"></span>

**Good**

The key comes from item.id, preserving the identity of each item across position changes.

```vue
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) · [All rules](all.md)

<span id="vue-no-bare-strings-in-template"></span>

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

```vue
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

```vue
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

<span id="vue-no-boolean-attr-value"></span>

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

```vue
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**Good**

The presence of each boolean attribute expresses the same enabled state without a value.

```vue
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [All rules](all.md)

<span id="vue-no-empty-component-block"></span>

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

```vue
<template></template>

<script></script>

<style>
</style>
```

<span id="vue-no-empty-component-block-good"></span>

**Good**

Each retained block contains actual markup, script declarations, or style declarations.

```vue
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

<span id="vue-no-inline-style"></span>

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

```vue
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**Good**

Classes express the fixed color; the ratio-dependent width remains a dynamic style binding, outside the static-attribute check.

```vue
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [All rules](all.md)

<span id="vue-no-invalid-html-attribute"></span>

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

```vue
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

<span id="vue-no-invalid-html-attribute-good"></span>

**Good**

The anchor uses help, a rel value appropriate for a linked help resource.

```vue
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) · [All rules](all.md)

<span id="vue-no-lone-template"></span>

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

```vue
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**Good**

Removing the unnecessary wrapper leaves the paragraph directly inside div.

```vue
<template>
<div><p>Details</p></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [All rules](all.md)

<span id="vue-no-multiple-objects-in-class"></span>

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

```vue
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

<span id="vue-no-multiple-objects-in-class-good"></span>

**Good**

One object contains the class conditions; arrays with one object and a string or with non-literal entries remain allowed.

```vue
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) · [All rules](all.md)

<span id="vue-no-negated-v-if-condition"></span>

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

```vue
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

<span id="vue-no-non-component-keep-alive-child"></span>

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

```vue
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

```vue
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

<span id="vue-no-preprocessor-lang"></span>

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

```vue
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**Good**

The same CSS declarations omit the preprocessor lang. This is the convention repair, not an executable Bad/Good diagnostic difference today.

```vue
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [All rules](all.md)

<span id="vue-no-root-v-if"></span>

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

```vue
<template>
<div v-if="show">content</div>
</template>
```

<span id="vue-no-root-v-if-good"></span>

**Good**

A stable outer div remains the root while the nested paragraph carries the visibility condition.

```vue
<template>
<div>
<p v-if="show">content</p>
</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [All rules](all.md)

<span id="vue-no-script-non-standard-lang"></span>

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

```vue
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**Good**

The script uses an ordinary TypeScript declaration with lang=ts, illustrating the intended language convention.

```vue
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [All rules](all.md)

<span id="vue-no-src-attribute"></span>

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

```vue
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**Good**

Each SFC block contains its own content without an external src attribute.

```vue
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

<span id="vue-no-static-inline-styles"></span>

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

```vue
<template>
<p style="color: red">Notice</p>
</template>
```

<span id="vue-no-static-inline-styles-good"></span>

**Good**

A notice class and scoped stylesheet hold the constant color outside the template attribute.

```vue
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [All rules](all.md)

<span id="vue-no-template-lang"></span>

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

```vue
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**Good**

An ordinary HTML template omits lang and uses the paragraph directly. This illustrates the convention without claiming a current SFC finding.

```vue
<template>
<p>Notice</p>
</template>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [All rules](all.md)

<span id="vue-no-template-target-blank"></span>

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

```vue
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

<span id="vue-no-template-target-blank-good"></span>

**Good**

The same link includes noopener noreferrer alongside target=_blank.

```vue
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [All rules](all.md)

<span id="vue-no-undefined-refs"></span>

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

```vue
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

<span id="vue-no-undefined-refs-good"></span>

**Good**

The interpolation reads the existing message binding.

```vue
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) · [All rules](all.md)

<span id="vue-no-unsafe-url"></span>

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

```vue
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**Good**

The anchor uses the ordinary local /next navigation destination.

```vue
<template>
<a href="/next">Continue</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [All rules](all.md)

<span id="vue-no-unsandboxed-iframe"></span>

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

```vue
<template>
<iframe src="/embed"></iframe>
</template>
```

<span id="vue-no-unsandboxed-iframe-good"></span>

**Good**

sandbox applies restrictions; allow-scripts explicitly opts into that one capability when needed.

```vue
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) · [All rules](all.md)

<span id="vue-no-unused-refs"></span>

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

```vue
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

<span id="vue-no-unused-refs-good"></span>

**Good**

The inputEl template ref has a same-named ref binding in script setup.

```vue
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) · [All rules](all.md)

<span id="vue-no-unused-setup-bindings"></span>

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

```vue
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

<span id="vue-no-unused-setup-bindings-good"></span>

**Good**

The paragraph interpolates message, using the declared binding.

```vue
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [All rules](all.md)

<span id="vue-no-use-v-else-with-v-for"></span>

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

```vue
<template>
<p v-if="ready">Ready</p>
<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

<span id="vue-no-use-v-else-with-v-for-good"></span>

**Good**

A separate template owns v-else, and its child paragraph owns v-for.

```vue
<template>
<p v-if="ready">Ready</p>
<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) · [All rules](all.md)

<span id="vue-no-useless-mustaches"></span>

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

```vue
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

<span id="vue-no-useless-mustaches-good"></span>

**Good**

Literal text is written directly; variable expressions, interpolated template strings, and intentional separator whitespace remain interpolation cases.

```vue
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) · [All rules](all.md)

<span id="vue-no-useless-v-bind"></span>

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

```vue
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

<span id="vue-no-useless-v-bind-good"></span>

**Good**

The constant value becomes a static attribute; variable and interpolated values retain their binding.

```vue
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [All rules](all.md)

<span id="vue-no-v-text"></span>

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

```vue
<template>
<div v-text="message"></div>
</template>
```

<span id="vue-no-v-text-good"></span>

**Good**

Mustache interpolation expresses the same text binding directly in the element content.

```vue
<template>
<div>{{ message }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) · [All rules](all.md)

<span id="vue-prefer-props-shorthand"></span>

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

```vue
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

```vue
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

<span id="vue-prefer-true-attribute-shorthand"></span>

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

```vue
<template>
<input :disabled="true" />
</template>
```

<span id="vue-prefer-true-attribute-shorthand-good"></span>

**Good**

The native attribute uses its boolean shorthand. False bindings and component props retain their explicit values.

```vue
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) · [All rules](all.md)

<span id="vue-require-component-registration"></span>

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

```vue
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**Good**

`MyButton` is listed in the example's `globals` option. That option exempts a known global component; it does not register or import it.

```vue
<template>
<MyButton />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [All rules](all.md)

<span id="vue-require-scoped-style"></span>

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

```vue
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**Good**

Adding `scoped` applies Vue's component scope to the same selector and declarations.

```vue
<style scoped>
.button {
  color: red;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [All rules](all.md)

<span id="vue-scoped-event-names"></span>

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

```vue
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

```vue
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

<span id="vue-sfc-element-order"></span>

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

```vue
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

```vue
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

<span id="vue-single-style-block"></span>

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

```vue
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

<span id="vue-slot-name-casing"></span>

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

```vue
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

<span id="vue-slot-name-casing-good"></span>

**Good**

`#my-slot` uses kebab-case. Rename the corresponding slot outlet to the same name.

```vue
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [All rules](all.md)

<span id="vue-this-in-template"></span>

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

```vue
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

<span id="vue-this-in-template-good"></span>

**Good**

Use `message`, `className`, and `handleClick` directly. The literal string `'this.is.a.string'` stays unchanged because it is not a member access.

```vue
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) · [All rules](all.md)

<span id="vue-v-on-event-hyphenation"></span>

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

```vue
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
</template>
```

<span id="vue-v-on-event-hyphenation-good"></span>

**Good**

`@my-event` uses the required custom-event spelling. Native-element listeners and dynamic event arguments shown below are outside this check.

```vue
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) · [All rules](all.md)

<span id="vue-v-on-handler-style"></span>

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

```vue
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

<span id="vue-v-on-handler-style-good"></span>

**Good**

Use a handler reference, or an arrow/function expression when inline logic is needed. The function boundary makes the handler form explicit.

```vue
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) · [All rules](all.md)

<span id="vue-warn-custom-block"></span>

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

```vue
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

```vue
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [All rules](all.md)

<span id="vue-warn-custom-directive"></span>

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

```vue
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**Good**

The example uses built-in `v-if`, `v-model`, and `v-on`. A correctly registered custom directive can still be valid Vue when this policy is disabled.

```vue
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [All rules](all.md)

<span id="a11y-alt-text"></span>

### `a11y/alt-text`

Require alternative text for media elements

[Bad](#a11y-alt-text-bad) · [Good](#a11y-alt-text-good)

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
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-alt-text-bad"></span>

**Bad**

The image submit control supplies only its image URL; it has no `alt` text describing the action.

```vue
<template>
  <input type="image" src="/submit.png" />
</template>
```

<span id="a11y-alt-text-good"></span>

**Good**

`alt="Submit search"` gives the image control an accessible name that describes submitting the search.

```vue
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [All rules](all.md)

<span id="a11y-anchor-has-content"></span>

### `a11y/anchor-has-content`

Require anchor elements to have accessible content

[Bad](#a11y-anchor-has-content-bad) · [Good](#a11y-anchor-has-content-good)

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
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-has-content-bad"></span>

**Bad**

The `/settings` link has no text or other naming content, so its destination has no accessible description.

```vue
<template>
  <a href="/settings"></a>
</template>
```

<span id="a11y-anchor-has-content-good"></span>

**Good**

The visible `Settings` text supplies content for the same destination link.

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [All rules](all.md)

<span id="a11y-anchor-is-valid"></span>

### `a11y/anchor-is-valid`

Enforce valid href on anchor elements

[Bad](#a11y-anchor-is-valid-bad) · [Good](#a11y-anchor-is-valid-good)

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
        "a11y/anchor-is-valid": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-is-valid-bad"></span>

**Bad**

The first anchor uses `#` for an action; the second uses a JavaScript URL. Neither provides an ordinary navigation destination.

```vue
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

<span id="a11y-anchor-is-valid-good"></span>

**Good**

A native button performs `openPanel`, while the remaining anchor has the real `/docs/javascript-urls` destination.

```vue
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript-urls">JavaScript URL guide</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) · [All rules](all.md)

<span id="a11y-aria-props"></span>

### `a11y/aria-props`

Disallow invalid ARIA attributes

[Bad](#a11y-aria-props-bad) · [Good](#a11y-aria-props-good)

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
        "a11y/aria-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-props-bad"></span>

**Bad**

`aria-lable` is misspelled and is not a supported ARIA attribute.

```vue
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

<span id="a11y-aria-props-good"></span>

**Good**

The supported `aria-label` attribute supplies the button name.

```vue
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [All rules](all.md)

<span id="a11y-aria-role"></span>

### `a11y/aria-role`

Elements with ARIA roles must use a valid, non-abstract ARIA role

[Bad](#a11y-aria-role-bad) · [Good](#a11y-aria-role-good)

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
        "a11y/aria-role": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-role-bad"></span>

**Bad**

`datepicker` is not a recognized ARIA role for this section.

```vue
<template>
  <section role="datepicker">...</section>
</template>
```

<span id="a11y-aria-role-good"></span>

**Good**

The section uses the recognized `dialog` role and a label describing the date selection.

```vue
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [All rules](all.md)

<span id="a11y-aria-unsupported-elements"></span>

### `a11y/aria-unsupported-elements`

Disallow ARIA attributes on elements that do not support them

[Bad](#a11y-aria-unsupported-elements-bad) · [Good](#a11y-aria-unsupported-elements-good)

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
        "a11y/aria-unsupported-elements": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-unsupported-elements-bad"></span>

**Bad**

The metadata element carries `aria-hidden`, although `meta` does not support ARIA attributes.

```vue
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

<span id="a11y-aria-unsupported-elements-good"></span>

**Good**

Removing the ARIA attribute leaves the charset declaration intact.

```vue
<template>
  <meta charset="utf-8" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [All rules](all.md)

<span id="a11y-click-events-have-key-events"></span>

### `a11y/click-events-have-key-events`

Require keyboard event handlers with click events

[Bad](#a11y-click-events-have-key-events-bad) · [Good](#a11y-click-events-have-key-events-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Checks non-interactive elements without an interactive role. Native buttons and elements with an interactive ARIA role are outside this rule's finding.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/click-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-click-events-have-key-events-bad"></span>

**Bad**

The non-interactive `div` has a click handler but no keyboard event handling.

```vue
<template>
<div @click="activate">Activate</div>
</template>
```

<span id="a11y-click-events-have-key-events-good"></span>

**Good**

A native `button` provides keyboard activation for the same `activate` handler.

```vue
<template>
<button @click="activate">Activate</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) · [All rules](all.md)

<span id="a11y-form-control-has-label"></span>

### `a11y/form-control-has-label`

Require form controls to have associated labels

[Bad](#a11y-form-control-has-label-bad) · [Good](#a11y-form-control-has-label-good)

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
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-form-control-has-label-bad"></span>

**Bad**

The search input has no label identifying what the user should enter.

```vue
<template>
  <input type="search" />
</template>
```

<span id="a11y-form-control-has-label-good"></span>

**Good**

Wrapping the input in a label associates the visible `Search` text with the control.

```vue
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [All rules](all.md)

<span id="a11y-heading-has-content"></span>

### `a11y/heading-has-content`

Require heading elements to have accessible content

[Bad](#a11y-heading-has-content-bad) · [Good](#a11y-heading-has-content-good)

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
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-has-content-bad"></span>

**Bad**

The `h2` contributes a heading level but has no heading content.

```vue
<template>
  <h2></h2>
</template>
```

<span id="a11y-heading-has-content-good"></span>

**Good**

`Billing settings` supplies the content of the existing level-two heading.

```vue
<template>
  <h2>Billing settings</h2>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) · [All rules](all.md)

<span id="a11y-heading-levels"></span>

### `a11y/heading-levels`

Disallow skipping heading levels

[Bad](#a11y-heading-levels-bad) · [Good](#a11y-heading-levels-good)

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
        "a11y/heading-levels": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-levels-bad"></span>

**Bad**

The heading sequence jumps directly from `h1` to `h3`, skipping level two.

```vue
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

<span id="a11y-heading-levels-good"></span>

**Good**

Changing the billing heading to `h2` preserves a consecutive heading hierarchy.

```vue
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [All rules](all.md)

<span id="a11y-iframe-has-title"></span>

### `a11y/iframe-has-title`

Require iframe elements to have a title attribute

[Bad](#a11y-iframe-has-title-bad) · [Good](#a11y-iframe-has-title-good)

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
        "a11y/iframe-has-title": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-iframe-has-title-bad"></span>

**Bad**

The checkout frame has a source URL but no `title` describing the embedded content.

```vue
<template>
  <iframe src="/checkout"></iframe>
</template>
```

<span id="a11y-iframe-has-title-good"></span>

**Good**

`title="Checkout preview"` names the content of that frame.

```vue
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) · [All rules](all.md)

<span id="a11y-img-alt"></span>

### `a11y/img-alt`

Require alt attribute on images for accessibility

[Bad](#a11y-img-alt-bad) · [Good](#a11y-img-alt-good)

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
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-img-alt-bad"></span>

**Bad**

The avatar image is missing its `alt` attribute.

```vue
<template>
  <img src="/avatar.png" />
</template>
```

<span id="a11y-img-alt-good"></span>

**Good**

`alt="User avatar"` supplies a text alternative for the avatar.

```vue
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [All rules](all.md)

<span id="a11y-interactive-supports-focus"></span>

### `a11y/interactive-supports-focus`

Require interactive role elements to be focusable

[Bad](#a11y-interactive-supports-focus-bad) · [Good](#a11y-interactive-supports-focus-good)

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
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-interactive-supports-focus-bad"></span>

**Bad**

Giving a `span` the button role and a click handler does not make the element keyboard-focusable.

```vue
<template>
  <span role="button" @click="open">Open</span>
</template>
```

<span id="a11y-interactive-supports-focus-good"></span>

**Good**

The native button is focusable and retains the same `open` action.

```vue
<template>
  <button type="button" @click="open">Open</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [All rules](all.md)

<span id="a11y-label-has-for"></span>

### `a11y/label-has-for`

Require labels to have associated form controls

[Bad](#a11y-label-has-for-bad) · [Good](#a11y-label-has-for-good)

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
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-label-has-for-bad"></span>

**Bad**

The separate label is neither associated through `for` nor wrapped around the input.

```vue
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

<span id="a11y-label-has-for-good"></span>

**Good**

`for="email"` matches the input ID and explicitly associates the two elements.

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [All rules](all.md)

<span id="a11y-landmark-roles"></span>

### `a11y/landmark-roles`

Validate landmark role placement and uniqueness

[Bad](#a11y-landmark-roles-bad) · [Good](#a11y-landmark-roles-good)

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
        "a11y/landmark-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-landmark-roles-bad"></span>

**Bad**

Two `main` elements declare duplicate main landmarks in the same template.

```vue
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

<span id="a11y-landmark-roles-good"></span>

**Good**

The dashboard remains the main landmark; the settings area becomes a named navigation landmark.

```vue
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) · [All rules](all.md)

<span id="a11y-media-has-caption"></span>

### `a11y/media-has-caption`

Require media elements to have captions

[Bad](#a11y-media-has-caption-bad) · [Good](#a11y-media-has-caption-good)

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
        "a11y/media-has-caption": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-media-has-caption-bad"></span>

**Bad**

The video has playback controls but no caption track.

```vue
<template>
  <video src="/demo.mp4" controls />
</template>
```

<span id="a11y-media-has-caption-good"></span>

**Good**

A `track` with `kind="captions"` supplies the English captions for the same video.

```vue
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) · [All rules](all.md)

<span id="a11y-mouse-events-have-key-events"></span>

### `a11y/mouse-events-have-key-events`

Require focus/blur events with mouse events

[Bad](#a11y-mouse-events-have-key-events-bad) · [Good](#a11y-mouse-events-have-key-events-good)

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
        "a11y/mouse-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-mouse-events-have-key-events-bad"></span>

**Bad**

Preview visibility changes only through mouse enter and leave handlers.

```vue
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

<span id="a11y-mouse-events-have-key-events-good"></span>

**Good**

The same preview actions run on focus and blur, and the button can receive keyboard focus.

```vue
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) · [All rules](all.md)

<span id="a11y-no-access-key"></span>

### `a11y/no-access-key`

Disallow the use of the accesskey attribute

[Bad](#a11y-no-access-key-bad) · [Good](#a11y-no-access-key-good)

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
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-access-key-bad"></span>

**Bad**

The `accesskey="s"` shortcut may conflict with browser or assistive-technology shortcuts.

```vue
<template>
  <button accesskey="s">Save</button>
</template>
```

<span id="a11y-no-access-key-good"></span>

**Good**

Removing `accesskey` keeps the ordinary Save button available.

```vue
<template>
  <button>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [All rules](all.md)

<span id="a11y-no-aria-hidden-on-focusable"></span>

### `a11y/no-aria-hidden-on-focusable`

Disallow aria-hidden="true" on focusable elements

[Bad](#a11y-no-aria-hidden-on-focusable-bad) · [Good](#a11y-no-aria-hidden-on-focusable-good)

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
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

**Bad**

The focusable Close button is hidden from the accessibility tree with `aria-hidden="true"`.

```vue
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

**Good**

The button remains exposed and receives a `Close` label instead of being hidden.

```vue
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [All rules](all.md)

<span id="a11y-no-autofocus"></span>

### `a11y/no-autofocus`

Disallow the use of the autofocus attribute

[Bad](#a11y-no-autofocus-bad) · [Good](#a11y-no-autofocus-good)

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
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-autofocus-bad"></span>

**Bad**

The input requests automatic focus when it appears.

```vue
<template>
  <input autofocus name="query" />
</template>
```

<span id="a11y-no-autofocus-good"></span>

**Good**

Removing `autofocus` avoids this automatic focus request while retaining the query input.

```vue
<template>
  <input name="query" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) · [All rules](all.md)

<span id="a11y-no-distracting-elements"></span>

### `a11y/no-distracting-elements`

Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt;

[Bad](#a11y-no-distracting-elements-bad) · [Good](#a11y-no-distracting-elements-good)

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
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-distracting-elements-bad"></span>

**Bad**

The `marquee` element introduces automatically moving text.

```vue
<template>
  <marquee>Limited offer</marquee>
</template>
```

<span id="a11y-no-distracting-elements-good"></span>

**Good**

A paragraph displays the same offer without the distracting marquee element.

```vue
<template>
  <p>Limited offer</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) · [All rules](all.md)

<span id="a11y-no-i-for-icon"></span>

### `a11y/no-i-for-icon`

Disallow using &lt;i&gt; element for icons

[Bad](#a11y-no-i-for-icon-bad) · [Good](#a11y-no-i-for-icon-good)

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
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-i-for-icon-bad"></span>

**Bad**

The icon is rendered through `i`, whose text semantics do not describe an icon-only action.

```vue
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

<span id="a11y-no-i-for-icon-good"></span>

**Good**

A decorative span hides the icon glyph, while the separate `Delete item` text names the button action.

```vue
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [All rules](all.md)

<span id="a11y-no-redundant-roles"></span>

### `a11y/no-redundant-roles`

Disallow redundant ARIA roles

[Bad](#a11y-no-redundant-roles-bad) · [Good](#a11y-no-redundant-roles-good)

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
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-redundant-roles-bad"></span>

**Bad**

The native button already has the button role, so `role="button"` repeats its implicit semantics.

```vue
<template>
  <button role="button">Save</button>
</template>
```

<span id="a11y-no-redundant-roles-good"></span>

**Good**

Removing the repeated role keeps the button semantics supplied by HTML.

```vue
<template>
  <button>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) · [All rules](all.md)

<span id="a11y-no-refer-to-non-existent-id"></span>

### `a11y/no-refer-to-non-existent-id`

Disallow references to non-existent IDs

[Bad](#a11y-no-refer-to-non-existent-id-bad) · [Good](#a11y-no-refer-to-non-existent-id-good)

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
        "a11y/no-refer-to-non-existent-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

**Bad**

`aria-labelledby` points to `save-label`, but no element declares that ID.

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

<span id="a11y-no-refer-to-non-existent-id-good"></span>

**Good**

Adding the matching span resolves the reference and provides the button label.

```vue
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [All rules](all.md)

<span id="a11y-no-role-presentation-on-focusable"></span>

### `a11y/no-role-presentation-on-focusable`

Disallow role="presentation" or role="none" on focusable elements

[Bad](#a11y-no-role-presentation-on-focusable-bad) · [Good](#a11y-no-role-presentation-on-focusable-good)

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
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

**Bad**

The focusable billing link requests role=presentation, which conflicts with its interactive link role; browsers must ignore that presentation request.

```vue
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

<span id="a11y-no-role-presentation-on-focusable-good"></span>

**Good**

Remove the conflicting presentation request and rely on the native link role and billing destination.

```vue
<template>
  <a href="/billing">Billing</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [All rules](all.md)

<span id="a11y-no-static-element-interactions"></span>

### `a11y/no-static-element-interactions`

Disallow event handlers on static elements

[Bad](#a11y-no-static-element-interactions-bad) · [Good](#a11y-no-static-element-interactions-good)

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
        "a11y/no-static-element-interactions": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-static-element-interactions-bad"></span>

**Bad**

A static section receives an Enter-key action without an interactive role.

```vue
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

<span id="a11y-no-static-element-interactions-good"></span>

**Good**

A native button carries the same action with an appropriate interactive element.

```vue
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [All rules](all.md)

<span id="a11y-placeholder-label-option"></span>

### `a11y/placeholder-label-option`

Require disabled or hidden on select placeholder option

[Bad](#a11y-placeholder-label-option-bad) · [Good](#a11y-placeholder-label-option-good)

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
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-placeholder-label-option-bad"></span>

**Bad**

The empty-value prompt remains selectable as if it were a country value.

```vue
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

<span id="a11y-placeholder-label-option-good"></span>

**Good**

Adding `disabled` distinguishes the prompt from the selectable Japan option.

```vue
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) · [All rules](all.md)

<span id="a11y-role-has-required-aria-props"></span>

### `a11y/role-has-required-aria-props`

Require ARIA roles to have required properties

[Bad](#a11y-role-has-required-aria-props-bad) · [Good](#a11y-role-has-required-aria-props-good)

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
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-role-has-required-aria-props-bad"></span>

**Bad**

The checkbox role omits `aria-checked`, which conveys the checkbox state.

```vue
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

<span id="a11y-role-has-required-aria-props-good"></span>

**Good**

`aria-checked="false"` supplies the state required by the checkbox role.

```vue
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) · [All rules](all.md)

<span id="a11y-tabindex-no-positive"></span>

### `a11y/tabindex-no-positive`

Disallow positive tabindex values

[Bad](#a11y-tabindex-no-positive-bad) · [Good](#a11y-tabindex-no-positive-good)

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
        "a11y/tabindex-no-positive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-tabindex-no-positive-bad"></span>

**Bad**

A positive tabindex of 3 creates a custom focus order ahead of ordinary controls.

```vue
<template>
  <button tabindex="3">Save</button>
</template>
```

<span id="a11y-tabindex-no-positive-good"></span>

**Good**

The button uses its native focus order without a positive tabindex.

```vue
<template>
  <button>Save</button>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) · [All rules](all.md)

<span id="a11y-use-list"></span>

### `a11y/use-list`

Suggest using list elements for bullet-like text

[Bad](#a11y-use-list-bad) · [Good](#a11y-use-list-good)

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
        "a11y/use-list": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-use-list-bad"></span>

**Bad**

The task items are separate paragraphs with typed dash markers rather than list elements.

```vue
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

<span id="a11y-use-list-good"></span>

**Good**

An unordered list and list items express the same tasks with list semantics.

```vue
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [All rules](all.md)

<span id="vue-use-unique-element-ids"></span>

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

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**Good**

`useId()` produces the instance's `emailId`; bind the same value to the label's `for` and the input's `id`.

```vue
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

<span id="html-deprecated-attr"></span>

### `html/deprecated-attr`

Disallow deprecated HTML attributes

[Bad](#html-deprecated-attr-bad) · [Good](#html-deprecated-attr-good)

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
        "html/deprecated-attr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-attr-bad"></span>

**Bad**

The paragraph uses the deprecated presentational `align` attribute.

```vue
<template>
<p align="center">Notice</p>
</template>
```

<span id="html-deprecated-attr-good"></span>

**Good**

The class and `text-align: center` declaration express the alignment through CSS.

```vue
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [All rules](all.md)

<span id="html-deprecated-element"></span>

### `html/deprecated-element`

Disallow deprecated HTML elements

[Bad](#html-deprecated-element-bad) · [Good](#html-deprecated-element-good)

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
        "html/deprecated-element": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-element-bad"></span>

**Bad**

The `center` element uses a deprecated HTML presentation element.

```vue
<template>
  <center>Profile</center>
</template>
```

<span id="html-deprecated-element-good"></span>

**Good**

A section and a styling class replace the deprecated element while preserving the content.

```vue
<template>
  <section class="profile">Profile</section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) · [All rules](all.md)

<span id="html-id-duplication"></span>

### `html/id-duplication`

Disallow duplicate element IDs

[Bad](#html-id-duplication-bad) · [Good](#html-id-duplication-good)

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
        "html/id-duplication": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-id-duplication-bad"></span>

**Bad**

Both the input and help paragraph declare `id="email"`, so the label target is ambiguous.

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

<span id="html-id-duplication-good"></span>

**Good**

The input keeps `email`; the help paragraph uses `email-help`, and aria-describedby refers to that distinct ID.

```vue
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) · [All rules](all.md)

<span id="html-no-consecutive-br"></span>

### `html/no-consecutive-br`

Disallow consecutive &lt;br&gt; elements

[Bad](#html-no-consecutive-br-bad) · [Good](#html-no-consecutive-br-good)

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
        "html/no-consecutive-br": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-consecutive-br-bad"></span>

**Bad**

Two consecutive break elements create spacing between blocks inside a single paragraph.

```vue
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

<span id="html-no-consecutive-br-good"></span>

**Good**

Separate paragraphs express the two content blocks without repeated break elements.

```vue
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) · [All rules](all.md)

<span id="html-no-dupe-style-properties"></span>

### `html/no-dupe-style-properties`

Disallow duplicate properties in inline style attributes

[Bad](#html-no-dupe-style-properties-bad) · [Good](#html-no-dupe-style-properties-good)

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
        "html/no-dupe-style-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-dupe-style-properties-bad"></span>

**Bad**

Each static style repeats one property; `margin` and `MARGIN` also count as the same property.

```vue
<template>
<div style="color: red; color: blue">text</div>
<div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

<span id="html-no-dupe-style-properties-good"></span>

**Good**

The static style uses distinct color and background properties. Dynamic style bindings are outside this static-attribute check.

```vue
<template>
<div style="color: red; background: blue">text</div>
<div :style="{ color: a, color: b }">text</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) · [All rules](all.md)

<span id="html-no-duplicate-class"></span>

### `html/no-duplicate-class`

Disallow duplicate class names in a static class attribute

[Bad](#html-no-duplicate-class-bad) · [Good](#html-no-duplicate-class-good)

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
        "html/no-duplicate-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-class-bad"></span>

**Bad**

The static class list repeats the `btn` token.

```vue
<template>
<div class="btn btn primary">click</div>
</template>
```

<span id="html-no-duplicate-class-good"></span>

**Good**

The class list keeps one `btn` token and the distinct `primary` token.

```vue
<template>
<div class="btn primary">click</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) · [All rules](all.md)

<span id="html-no-duplicate-dt"></span>

### `html/no-duplicate-dt`

Disallow duplicate &lt;dt&gt; names in &lt;dl&gt;

[Bad](#html-no-duplicate-dt-bad) · [Good](#html-no-duplicate-dt-good)

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
        "html/no-duplicate-dt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-dt-bad"></span>

**Bad**

The same definition list repeats the `API` term for two descriptions.

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

<span id="html-no-duplicate-dt-good"></span>

**Good**

One API term is followed by both descriptions, avoiding the repeated term.

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) · [All rules](all.md)

<span id="html-no-empty-palpable-content"></span>

### `html/no-empty-palpable-content`

Disallow empty elements that expect visible content

[Bad](#html-no-empty-palpable-content-bad) · [Good](#html-no-empty-palpable-content-good)

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
        "html/no-empty-palpable-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-empty-palpable-content-bad"></span>

**Bad**

The paragraph, list item, and table cell all have empty palpable content.

```vue
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

<span id="html-no-empty-palpable-content-good"></span>

**Good**

Text fills the paragraph, interpolation supplies the list item, and aria-label explicitly names the otherwise empty cell.

```vue
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) · [All rules](all.md)

<span id="html-require-datetime"></span>

### `html/require-datetime`

Require datetime attribute on &lt;time&gt; element

[Bad](#html-require-datetime-bad) · [Good](#html-require-datetime-good)

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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-require-datetime-bad"></span>

**Bad**

The time element contains a human-readable date but no machine-readable datetime value.

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

<span id="html-require-datetime-good"></span>

**Good**

`datetime="2026-05-13"` supplies the corresponding machine-readable date.

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [All rules](all.md)

<span id="type-no-floating-promises"></span>

### `type/no-floating-promises`

Disallow floating (unhandled) Promises

[Bad](#type-no-floating-promises-bad) · [Good](#type-no-floating-promises-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-floating-promises": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-floating-promises-bad"></span>

**Bad**

The async `save` function returns a Promise, but the standalone `save()` call neither awaits nor returns it and does not explicitly mark intentional disposal.

```vue
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

<span id="type-no-floating-promises-good"></span>

**Good**

`void save()` explicitly marks the fire-and-forget intent accepted by this rule. This is an explicit disposal marker, not a rejection handler.

```vue
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) · [All rules](all.md)

<span id="type-no-reactivity-loss"></span>

### `type/no-reactivity-loss`

Disallow plain snapshots of reactive values across assignments and calls

[Bad](#type-no-reactivity-loss-bad) · [Good](#type-no-reactivity-loss-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-reactivity-loss": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-reactivity-loss-bad"></span>

**Bad**

`const count = state.count` takes a plain numeric snapshot of the reactive property, so later updates of `state.count` are not reflected in that binding.

```vue
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0 });
const count = state.count;
</script>
```

<span id="type-no-reactivity-loss-good"></span>

**Good**

`toRef(state, "count")` keeps `count` linked to the original reactive property rather than copying its current primitive value.

```vue
<script setup lang="ts">
import { reactive, toRef } from "vue";
const state = reactive({ count: 0 });
const count = toRef(state, "count");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) · [All rules](all.md)

<span id="type-no-unsafe-template-binding"></span>

### `type/no-unsafe-template-binding`

Disallow template bindings that resolve to unsafe types

[Bad](#type-no-unsafe-template-binding-bad) · [Good](#type-no-unsafe-template-binding-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-unsafe-template-binding": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-unsafe-template-binding-bad"></span>

**Bad**

The interpolated `value` is explicitly typed as `any`, so the checker cannot give the template binding a safe concrete type.

```vue
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

<span id="type-no-unsafe-template-binding-good"></span>

**Good**

Changing the annotation to `string` gives the same interpolation a concrete, checkable type without changing the rendered value.

```vue
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) · [All rules](all.md)

<span id="type-require-typed-emits"></span>

### `type/require-typed-emits`

Require type definition for defineEmits

[Bad](#type-require-typed-emits-bad) · [Good](#type-require-typed-emits-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-emits-bad"></span>

**Bad**

The array-only `defineEmits(["save"])` declares the event name without a typed payload contract.

```vue
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

<span id="type-require-typed-emits-good"></span>

**Good**

`defineEmits<{ save: [] }>()` declares the typed `save` event with an empty payload tuple, explicitly stating that it takes no payload arguments.

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) · [All rules](all.md)

<span id="type-require-typed-props"></span>

### `type/require-typed-props`

Require type definition for defineProps

[Bad](#type-require-typed-props-bad) · [Good](#type-require-typed-props-good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-props": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-props-bad"></span>

**Bad**

The array-only `defineProps(["title"])` declares `title` by name without giving it a type.

```vue
<script setup lang="ts">
defineProps(["title"]);
</script>
```

<span id="type-require-typed-props-good"></span>

**Good**

`defineProps<{ title: string }>()` gives `title` an explicit string type instead of a name-only runtime declaration.

```vue
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) · [All rules](all.md)

<span id="type-strict-boolean-expressions"></span>

### `type/strict-boolean-expressions`

Require safe boolean expressions in script and template conditions

[Bad](#type-strict-boolean-expressions-bad) · [Good](#type-strict-boolean-expressions-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: See [typed options and defaults](options.md).

Enable typeAware and this rule explicitly. The default disallows nullable numbers, while non-null numbers are allowed.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/strict-boolean-expressions": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-strict-boolean-expressions-bad"></span>

**Bad**

`if (count)` relies on the truthiness of a nullable numeric binding instead of an explicit boolean test; it also conflates zero with absence.

```vue
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

<span id="type-strict-boolean-expressions-good"></span>

**Good**

`count !== undefined && count > 0` separately tests presence and positivity, producing an explicit boolean condition after narrowing the optional value.

```vue
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [All rules](all.md)

<span id="script-no-get-current-instance"></span>

### `script/no-get-current-instance`

Disallow getCurrentInstance() in Vapor mode (returns null)

[Bad](#script-no-get-current-instance-bad) · [Good](#script-no-get-current-instance-good)

Default severity: `error`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**Bad**

The Vapor-marked setup imports and calls `getCurrentInstance`, relying on an instance API this rule disallows for Vapor-oriented components.

```vue
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**Good**

`inject("app-config")` obtains the explicitly provided configuration without importing or calling `getCurrentInstance`.

```vue
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [All rules](all.md)

<span id="script-no-next-tick"></span>

### `script/no-next-tick`

Disallow nextTick() usage in Vapor-oriented components

[Bad](#script-no-next-tick-bad) · [Good](#script-no-next-tick-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**Bad**

The Vapor-oriented component imports and awaits `nextTick`, introducing the DOM-flush scheduling dependency that this migration rule rejects.

```vue
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**Good**

The input is obtained through `useTemplateRef` and focused at `onMounted`. The explicit mount boundary replaces the example’s `nextTick` dependency.

```vue
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [All rules](all.md)

<span id="script-no-options-api"></span>

### `script/no-options-api`

Disallow Options API patterns in Vapor mode

[Bad](#script-no-options-api-bad) · [Good](#script-no-options-api-good)

Default severity: `error`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**Bad**

The default-export object declares Options API `data()`, a component option form prohibited by this rule.

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**Good**

The component state becomes a Composition API `ref` in Vapor `<script setup>`, removing the Options API object and its `data` option.

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [All rules](all.md)

<span id="vapor-no-inline-template"></span>

### `vapor/no-inline-template`

Disallow deprecated inline-template attribute

[Bad](#vapor-no-inline-template-bad) · [Good](#vapor-no-inline-template-good)

Default severity: `error`  
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
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-inline-template-bad"></span>

**Bad**

LegacyCard uses the inline-template attribute for its child markup.

```vue
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

<span id="vapor-no-inline-template-good"></span>

**Good**

The markup is passed through the default slot instead of an inline template.

```vue
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) · [All rules](all.md)

<span id="vapor-no-vue-lifecycle-events"></span>

### `vapor/no-vue-lifecycle-events`

Disallow @vue:xxx per-element lifecycle events (not supported in Vapor)

[Bad](#vapor-no-vue-lifecycle-events-bad) · [Good](#vapor-no-vue-lifecycle-events-good)

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
        "vapor/no-vue-lifecycle-events": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-vue-lifecycle-events-bad"></span>

**Bad**

The input uses the @vue:mounted template lifecycle event.

```vue
<template>
  <input @vue:mounted="focusInput" />
</template>
```

<span id="vapor-no-vue-lifecycle-events-good"></span>

**Good**

onMounted accesses the named template reference and focuses the input through the supported script lifecycle hook.

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) · [All rules](all.md)

<span id="vapor-prefer-static-class"></span>

### `vapor/prefer-static-class`

Prefer static class over dynamic class binding for string literals

[Bad](#vapor-prefer-static-class-bad) · [Good](#vapor-prefer-static-class-good)

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
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-prefer-static-class-bad"></span>

**Bad**

The class binding evaluates a constant string even though the class does not change.

```vue
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

<span id="vapor-prefer-static-class-good"></span>

**Good**

A static class attribute expresses the same panel classes without a binding.

```vue
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) · [All rules](all.md)

<span id="vapor-require-vapor-attribute"></span>

### `vapor/require-vapor-attribute`

Suggest adding vapor attribute to script setup

[Bad](#vapor-require-vapor-attribute-bad) · [Good](#vapor-require-vapor-attribute-good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Not implemented for SFC lint  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Current support: `no-sfc-finding`

This rule is a placeholder with an empty callback. Adding vapor selects Vapor compilation; the current linter does not report this catalog ID for its absence.

**Configured ID (currently no SFC finding)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-require-vapor-attribute-bad"></span>

**Bad**

The script setup block lacks the Vapor compilation attribute. This is an intended convention: the current empty rule callback does not diagnose it.

```vue
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

<span id="vapor-require-vapor-attribute-good"></span>

**Good**

Adding vapor selects Vapor compilation. It demonstrates the intended repair and does not imply that the current linter emits this catalog rule.

```vue
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [All rules](all.md)

<span id="ecosystem-nuxt-prefer-nuxt-link"></span>

### `ecosystem/nuxt-prefer-nuxt-link`

Prefer NuxtLink for internal application links

[Bad](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Good](#ecosystem-nuxt-prefer-nuxt-link-good)

Default severity: `warning`  
Presets: `nuxt`  
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
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

**Bad**

The internal settings destination uses a plain anchor in a Nuxt application.

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

**Good**

NuxtLink handles the same internal destination through the Nuxt router.

```vue
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [All rules](all.md)

<span id="ecosystem-pinia-prefer-store-to-refs"></span>

### `ecosystem/pinia-prefer-store-to-refs`

Prefer storeToRefs() when destructuring Pinia stores

[Bad](#ecosystem-pinia-prefer-store-to-refs-bad) · [Good](#ecosystem-pinia-prefer-store-to-refs-good)

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

**Bad**

Destructuring `name` directly from the store separates the value from its reactive store access.

```vue
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

**Good**

The store remains intact and storeToRefs creates a reactive reference for name.

```vue
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) · [All rules](all.md)

<span id="ecosystem-router-link-require-to"></span>

### `ecosystem/router-link-require-to`

Require a `to` target on RouterLink and NuxtLink components

[Bad](#ecosystem-router-link-require-to-bad) · [Good](#ecosystem-router-link-require-to-good)

Default severity: `error`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

A single SFC root link may inherit its target from parent attributes. This example uses a nested link, whose target must be explicit.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/router-link-require-to": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-router-link-require-to-bad"></span>

**Bad**

The nested RouterLink has no `to` destination; it cannot rely on root attribute fallthrough.

```vue
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

<span id="ecosystem-router-link-require-to-good"></span>

**Good**

`to="/settings"` explicitly supplies the nested link destination.

```vue
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [All rules](all.md)

<span id="ecosystem-void-link-require-href"></span>

### `ecosystem/void-link-require-href`

Require `href` on Void Vue Link components

[Bad](#ecosystem-void-link-require-href-bad) · [Good](#ecosystem-void-link-require-href-good)

Default severity: `error`  
Presets: `ecosystem`  
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
        "ecosystem/void-link-require-href": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-require-href-bad"></span>

**Bad**

The Link imported from @void/vue omits its href destination.

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

<span id="ecosystem-void-link-require-href-good"></span>

**Good**

The same imported Link receives the settings destination through href.

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) · [All rules](all.md)

<span id="ecosystem-void-link-valid-method"></span>

### `ecosystem/void-link-valid-method`

Validate static Void Vue Link method props

[Bad](#ecosystem-void-link-valid-method-bad) · [Good](#ecosystem-void-link-valid-method-good)

Default severity: `warning`  
Presets: `ecosystem`  
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
        "ecosystem/void-link-valid-method": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-valid-method-bad"></span>

**Bad**

The DELETE action requests prefetching, although prefetch is intended for navigation requests.

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

<span id="ecosystem-void-link-valid-method-good"></span>

**Good**

Removing prefetch keeps the DELETE action without prefetching that non-GET request.

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) · [All rules](all.md)

<span id="ecosystem-vue-i18n-no-missing-key"></span>

### `ecosystem/vue-i18n-no-missing-key`

Report static vue-i18n keys that are absent from local SFC messages

[Bad](#ecosystem-vue-i18n-no-missing-key-bad) · [Good](#ecosystem-vue-i18n-no-missing-key-good)

Default severity: `warning`  
Presets: `ecosystem`  
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
        "ecosystem/vue-i18n-no-missing-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

**Bad**

The template requests auth.missing, but the local English messages declare only auth.login.

```vue
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

**Good**

The template requests the auth.login key that exists in the local messages.

```vue
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) · [All rules](all.md)

<span id="ecosystem-vue-router-prefer-named-link"></span>

### `ecosystem/vue-router-prefer-named-link`

Prefer named route objects over static path strings in RouterLink

[Bad](#ecosystem-vue-router-prefer-named-link-bad) · [Good](#ecosystem-vue-router-prefer-named-link-good)

Default severity: `warning`  
Presets: `ecosystem`  
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
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

**Bad**

The RouterLink destination is a literal path rather than a named route.

```vue
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

**Good**

The bound route object identifies the destination by its settings route name.

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) · [All rules](all.md)

<span id="ecosystem-vue-router-prefer-named-push"></span>

### `ecosystem/vue-router-prefer-named-push`

Prefer named route objects for Vue Router programmatic navigation

[Bad](#ecosystem-vue-router-prefer-named-push-bad) · [Good](#ecosystem-vue-router-prefer-named-push-good)

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

**Bad**

router.push receives a path string that is tied to the current URL spelling.

```vue
<script setup lang="ts">
router.push("/settings");
</script>
```

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

**Good**

router.push receives a route object with the stable settings name.

```vue
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [All rules](all.md)

<span id="ecosystem-vue-test-utils-no-html-snapshot"></span>

### `ecosystem/vue-test-utils-no-html-snapshot`

Avoid snapshotting wrapper.html() in Vue Test Utils tests

[Bad](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Good](#ecosystem-vue-test-utils-no-html-snapshot-good)

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

**Bad**

The assertion snapshots the complete wrapper HTML instead of checking the expected behavior.

```vue
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

**Good**

The assertion checks that the rendered text contains Saved.

```vue
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [All rules](all.md)

<span id="css-no-display-none"></span>

### `css/no-display-none`

Suggest using v-show instead of display: none

[Bad](#css-no-display-none-bad) · [Good](#css-no-display-none-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-display-none": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-display-none-bad"></span>

**Bad**

The `.message` declaration hides the local paragraph through CSS rather than a template visibility condition.

```vue
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

<span id="css-no-display-none-good"></span>

**Good**

`v-show="isSaved"` makes the visibility condition explicit on the local paragraph and removes `display: none`.

```vue
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) · [All rules](all.md)

<span id="css-no-hardcoded-values"></span>

### `css/no-hardcoded-values`

Suggest using CSS variables instead of hardcoded values

[Bad](#css-no-hardcoded-values-bad) · [Good](#css-no-hardcoded-values-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-hardcoded-values": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-hardcoded-values-bad"></span>

**Bad**

The button embeds spacing numbers and a hexadecimal color directly in the declarations.

```vue
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

<span id="css-no-hardcoded-values-good"></span>

**Good**

The declarations refer to named spacing and color custom properties, so these values can be maintained as tokens.

```vue
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) · [All rules](all.md)

<span id="css-no-id-selectors"></span>

### `css/no-id-selectors`

Discourage use of ID selectors in CSS

[Bad](#css-no-id-selectors-bad) · [Good](#css-no-id-selectors-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-id-selectors-bad"></span>

**Bad**

`#submit` ties the style rule to an ID selector.

```vue
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

<span id="css-no-id-selectors-good"></span>

**Good**

The `.submit` class expresses the reusable styling hook without an ID selector.

```vue
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) · [All rules](all.md)

<span id="css-no-important"></span>

### `css/no-important`

Discourage use of !important in CSS

[Bad](#css-no-important-bad) · [Good](#css-no-important-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-important": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-important-bad"></span>

**Bad**

The color declaration overrides normal cascade priority with `!important`.

```vue
<style scoped>
.button {
  color: red !important;
}
</style>
```

<span id="css-no-important-good"></span>

**Good**

The color comes from a custom property without an important declaration.

```vue
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) · [All rules](all.md)

<span id="css-no-utility-classes"></span>

### `css/no-utility-classes`

Warn against implementing utility classes in component styles

[Bad](#css-no-utility-classes-bad) · [Good](#css-no-utility-classes-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-utility-classes-bad"></span>

**Bad**

The authored selectors use utility-shaped names such as `.flex`, `.mt-4`, and `.text-center`.

```vue
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

<span id="css-no-utility-classes-good"></span>

**Good**

A component-specific `.my-component` selector groups the component styling under one semantic name.

```vue
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) · [All rules](all.md)

<span id="css-no-v-bind-performance"></span>

### `css/no-v-bind-performance`

Warn about performance cost of CSS v-bind()

[Bad](#css-no-v-bind-performance-bad) · [Good](#css-no-v-bind-performance-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-v-bind-performance-bad"></span>

**Bad**

The stylesheet reads the changing `offset` through the SFC CSS `v-bind()` mechanism.

```vue
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

<span id="css-no-v-bind-performance-good"></span>

**Good**

The element receives the changing transform directly through its style binding.

```vue
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) · [All rules](all.md)

<span id="css-prefer-logical-properties"></span>

### `css/prefer-logical-properties`

Recommend CSS logical properties for better i18n support

[Bad](#css-prefer-logical-properties-bad) · [Good](#css-prefer-logical-properties-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-logical-properties-bad"></span>

**Bad**

`margin-left` fixes the margin to a physical side regardless of writing direction.

```vue
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

<span id="css-prefer-logical-properties-good"></span>

**Good**

`margin-inline-start` follows the start of the inline direction instead.

```vue
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [All rules](all.md)

<span id="css-prefer-nested-selectors"></span>

### `css/prefer-nested-selectors`

Recommend using CSS nesting for descendant selectors

[Bad](#css-prefer-nested-selectors-bad) · [Good](#css-prefer-nested-selectors-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-nested-selectors-bad"></span>

**Bad**

The `.card .title` descendant selector repeats the parent selector in a flat rule.

```vue
<style scoped>
.card .title { color: red; }
</style>
```

<span id="css-prefer-nested-selectors-good"></span>

**Good**

The `.title` rule is nested inside `.card`, keeping the parent-child styling relationship together.

```vue
<style scoped>
.card { .title { color: red; } }
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [All rules](all.md)

<span id="css-prefer-slotted"></span>

### `css/prefer-slotted`

Recommend ::v-slotted() for styling slot content

[Bad](#css-prefer-slotted-bad) · [Good](#css-prefer-slotted-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-slotted-bad"></span>

**Bad**

The scoped stylesheet targets the `slot` outlet rather than the elements supplied through the slot.

```vue
<style scoped>
slot { color: red; }
</style>
```

<span id="css-prefer-slotted-good"></span>

**Good**

`:slotted(.label)` targets the supplied label element through the scoped slot selector.

```vue
<style scoped>
:slotted(.label) { color: red; }
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [All rules](all.md)

<span id="css-require-font-display"></span>

### `css/require-font-display`

Require font-display in @font-face rules

[Bad](#css-require-font-display-bad) · [Good](#css-require-font-display-good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/require-font-display": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-require-font-display-bad"></span>

**Bad**

The font-face declaration defines the font source but omits its font-display policy.

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

<span id="css-require-font-display-good"></span>

**Good**

`font-display: swap` explicitly selects the fallback-to-font display policy.

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) · [All rules](all.md)

<span id="musea-no-empty-variant"></span>

### `musea/no-empty-variant`

Disallow empty &lt;variant&gt; blocks

[Bad](#musea-no-empty-variant-bad) · [Good](#musea-no-empty-variant-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/no-empty-variant": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-no-empty-variant-bad"></span>

**Bad**

The named primary variant is empty, so it provides no preview content.

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-no-empty-variant-good"></span>

**Good**

The variant renders a primary Button with its Save content.

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) · [All rules](all.md)

<span id="musea-prefer-design-tokens"></span>

### `musea/prefer-design-tokens`

Prefer design token CSS variables over hardcoded primitive values

[Bad](#musea-prefer-design-tokens-bad) · [Good](#musea-prefer-design-tokens-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: See [typed options and defaults](options.md).

Requires an .art.vue file and the token inventory shown below. It does not infer a token from an arbitrary color.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
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

<span id="musea-prefer-design-tokens-bad"></span>

**Bad**

The art example uses the literal blue color instead of the configured primary design token.

`Button.art.vue`

```vue
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

<span id="musea-prefer-design-tokens-good"></span>

**Good**

The style refers to --color-primary, the token configured for this example.

`Button.art.vue`

```vue
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) · [All rules](all.md)

<span id="musea-require-component"></span>

### `musea/require-component`

Require component attribute in &lt;art&gt; block

[Bad](#musea-require-component-bad) · [Good](#musea-require-component-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-component": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-component-bad"></span>

**Bad**

The art block supplies a title but does not identify the component being previewed.

```vue
<art title="Button">
  <variant name="primary" />
</art>
```

<span id="musea-require-component-good"></span>

**Good**

defineArt supplies ./Button.vue as the component for the art block.

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [All rules](all.md)

<span id="musea-require-title"></span>

### `musea/require-title`

Require title attribute in &lt;art&gt; block

[Bad](#musea-require-title-bad) · [Good](#musea-require-title-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-title-bad"></span>

**Bad**

The art block identifies Button.vue but supplies no title.

```vue
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-require-title-good"></span>

**Good**

The defineArt options supply the Button title for the art block.

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) · [All rules](all.md)

<span id="musea-unique-variant-names"></span>

### `musea/unique-variant-names`

Require unique variant names

[Bad](#musea-unique-variant-names-bad) · [Good](#musea-unique-variant-names-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/unique-variant-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-unique-variant-names-bad"></span>

**Bad**

Two variants in the same art block both use the primary name.

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

<span id="musea-unique-variant-names-good"></span>

**Good**

The variants have distinct primary and secondary names.

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) · [All rules](all.md)

<span id="musea-valid-variant"></span>

### `musea/valid-variant`

Require name attribute in &lt;variant&gt; blocks

[Bad](#musea-valid-variant-bad) · [Good](#musea-valid-variant-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Musea .art.vue art, variant, and style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-valid-variant-bad"></span>

**Bad**

The variant omits the name needed to identify the preview.

```vue
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

<span id="musea-valid-variant-good"></span>

**Good**

The primary name identifies that variant.

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [All rules](all.md)

<span id="script-component-options-name-casing"></span>

### `script/component-options-name-casing`

Enforce PascalCase for the component `name` option

[Bad](#script-component-options-name-casing-bad) · [Good](#script-component-options-name-casing-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-component-options-name-casing-bad"></span>

**Bad**

The component option `name: 'my-component'` is kebab-case, whereas this rule requires a literal component name in PascalCase.

```vue
<script lang="ts">
export default {
name: 'my-component' // kebab-case
}
</script>
```

<span id="script-component-options-name-casing-good"></span>

**Good**

`MyComponent` begins with an uppercase letter and contains only alphanumeric characters, satisfying the name check.

```vue
<script lang="ts">
export default {
name: 'MyComponent'
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) · [All rules](all.md)

<span id="script-custom-event-name-casing"></span>

### `script/custom-event-name-casing`

Enforce camelCase for emitted custom event names

[Bad](#script-custom-event-name-casing-bad) · [Good](#script-custom-event-name-casing-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](options.md).

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/custom-event-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-custom-event-name-casing-bad"></span>

**Bad**

The emitted string `my-event` contains a hyphen and violates the default camelCase event naming policy.

```vue
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

<span id="script-custom-event-name-casing-good"></span>

**Good**

Both the declaration and call use `myEvent`, preserving agreement between the event name and its emission while satisfying the default casing policy. A configured kebab-case policy has a different expectation.

```vue
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) · [All rules](all.md)

<span id="script-define-emits-declaration"></span>

### `script/define-emits-declaration`

Enforce the type-based defineEmits&lt;{}&gt;() form over the runtime/array form

[Bad](#script-define-emits-declaration-bad) · [Good](#script-define-emits-declaration-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-emits-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-emits-declaration-bad"></span>

**Bad**

`defineEmits(["change"])` uses a runtime array declaration; this style rule prefers a type-based declaration.

```vue
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

<span id="script-define-emits-declaration-good"></span>

**Good**

`defineEmits<{ change: [id: number] }>()` moves the event declaration into a type argument and explicitly describes the numeric payload used by `emit("change", 1)`.

```vue
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [All rules](all.md)

<span id="script-define-macros-order"></span>

### `script/define-macros-order`

Enforce a consistent order of the Vue compiler macros in &lt;script setup&gt;

[Bad](#script-define-macros-order-bad) · [Good](#script-define-macros-order-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-macros-order-bad"></span>

**Bad**

`defineProps` appears before `defineModel`, although `defineModel` has the earlier rank in the canonical macro order.

```vue
<script setup lang="ts">
// defineProps before defineModel (out of canonical order)
const props = defineProps<{ count: number }>()
const model = defineModel<string>()
</script>
```

<span id="script-define-macros-order-good"></span>

**Good**

The declarations follow the exact sequence `defineOptions`, `defineModel`, `defineProps`, `defineEmits`, `defineSlots`, before unrelated runtime statements.

```vue
<script setup lang="ts">
defineOptions({ name: 'MyComponent' })
const model = defineModel<string>()
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
defineSlots<{ default(props: {}): any }>()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L46) · [All rules](all.md)

<span id="script-define-props-declaration"></span>

### `script/define-props-declaration`

Enforce type-based defineProps&lt;{ ... }&gt;() over the runtime/object form

[Bad](#script-define-props-declaration-bad) · [Good](#script-define-props-declaration-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-declaration-bad"></span>

**Bad**

`defineProps({ title: String })` supplies a runtime object, which conflicts with this rule’s preference for type-based props.

```vue
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

<span id="script-define-props-declaration-good"></span>

**Good**

`defineProps<{ title: string }>()` declares `title` in the type argument and retains the `props.title` access without a runtime declaration argument.

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [All rules](all.md)

<span id="script-define-props-destructuring"></span>

### `script/define-props-destructuring`

Enforce consistent style for defineProps destructuring in &lt;script setup&gt;

[Bad](#script-define-props-destructuring-bad) · [Good](#script-define-props-destructuring-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](options.md).

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-destructuring-bad"></span>

**Bad**

`defineProps` is assigned to the single `props` binding rather than destructured, contrary to the default destructuring preference.

```vue
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

<span id="script-define-props-destructuring-good"></span>

**Good**

The object pattern binds `foo` and `bar` directly and gives the optional `bar` a default. This relies on Vue 3.5+ reactive props destructuring; the configurable `never` mode prefers the opposite form.

```vue
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [All rules](all.md)

<span id="script-no-arrow-functions-in-watch"></span>

### `script/no-arrow-functions-in-watch`

Disallow arrow functions as Options API watch handlers

[Bad](#script-no-arrow-functions-in-watch-bad) · [Good](#script-no-arrow-functions-in-watch-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-arrow-functions-in-watch": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-arrow-functions-in-watch-bad"></span>

**Bad**

The Options API watcher `value` and the nested `other.handler` are arrow functions. An arrow captures its surrounding `this` rather than receiving the component instance.

```vue
<script lang="ts">
export default {
watch: {
// `this` is not the component instance inside an arrow function.
value: () => {
this.doSomething()
},
other: {
handler: () => {}
}
}
}
</script>
```

<span id="script-no-arrow-functions-in-watch-good"></span>

**Good**

Both handlers become ordinary methods, allowing Vue to bind `this` to the component. The `deep: true` watcher option remains compatible with the object form.

```vue
<script lang="ts">
export default {
watch: {
value(newValue, oldValue) {
this.doSomething()
},
other: {
handler(newValue) {},
deep: true
}
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) · [All rules](all.md)

<span id="script-no-async-in-computed"></span>

### `script/no-async-in-computed`

Disallow async functions in computed properties

[Bad](#script-no-async-in-computed-bad) · [Good](#script-no-async-in-computed-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-async-in-computed": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-async-in-computed-bad"></span>

**Bad**

The `computed` getter is `async`, so the fetch produces a Promise instead of a synchronously derived computed value.

```vue
<script setup lang="ts">
import { computed } from "vue";
const data = computed(async () => {
  const response = await fetch("/api/data");
  return response.json();
});
</script>
```

<span id="script-no-async-in-computed-good"></span>

**Good**

The asynchronous fetch moves into `watch` and stores its result in `data.value`. Cleanup aborts the old request and prevents an inactive callback from writing a stale result; no async computed getter remains.

```vue
<script setup lang="ts">
import { ref, watch } from "vue";
const query = ref("");
const data = ref<unknown>(null);
watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;
  onCleanup(() => { active = false; controller.abort(); });
  const response = await fetch(`/api/data?q=${encodeURIComponent(value)}`, { signal: controller.signal });
  const next: unknown = await response.json();
  if (active) data.value = next;
});
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) · [All rules](all.md)

<span id="script-no-boolean-default"></span>

### `script/no-boolean-default`

Disallow a default on a Boolean prop

[Bad](#script-no-boolean-default-bad) · [Good](#script-no-boolean-default-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-boolean-default": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-boolean-default-bad"></span>

**Bad**

Both `disabled` and `checked` declare a `default` on a prop whose sole constructor is `Boolean`; the rule rejects even an explicit `false` default.

```vue
<script lang="ts">
export default {
props: {
// Boolean props already default to false; an explicit default is confusing.
disabled: { type: Boolean, default: true },
checked: { type: Boolean, default: false }
}
}
</script>
```

<span id="script-no-boolean-default-good"></span>

**Good**

The Boolean-only props omit `default`, using Vue’s implicit false value. The `[Boolean, String]` union and the Number prop illustrate that this check is limited to the sole `Boolean` constructor.

```vue
<script lang="ts">
export default {
props: {
// No explicit default: defaults to false.
disabled: { type: Boolean },
disabled2: Boolean,
// Union type may legitimately need a default.
value: { type: [Boolean, String], default: '' },
// Non-Boolean prop.
count: { type: Number, default: 0 }
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) · [All rules](all.md)

<span id="script-no-deep-destructure-in-props"></span>

### `script/no-deep-destructure-in-props`

Disallow deeply nested destructuring in defineProps

[Bad](#script-no-deep-destructure-in-props-bad) · [Good](#script-no-deep-destructure-in-props-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deep-destructure-in-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deep-destructure-in-props-bad"></span>

**Bad**

The binding pattern descends through `user` to destructure `name`, exceeding the default shallow props-destructuring depth.

```vue
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

<span id="script-no-deep-destructure-in-props-good"></span>

**Good**

The props object remains intact, and a computed getter reads `props.user.name`. The nested access stays explicit without a deeply nested binding pattern.

```vue
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [All rules](all.md)

<span id="script-no-deprecated-data-object-declaration"></span>

### `script/no-deprecated-data-object-declaration`

Disallow an object literal as the component data option (Vue 3 requires a function)

[Bad](#script-no-deprecated-data-object-declaration-bad) · [Good](#script-no-deprecated-data-object-declaration-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-data-object-declaration": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-data-object-declaration-bad"></span>

**Bad**

The Options API `data` option is an object literal, a Vue 2 form that Vue 3 no longer accepts.

```vue
<script lang="ts">
export default {
// `data` must be a function in Vue 3, not an object literal.
data: {
count: 0
}
}
</script>
```

<span id="script-no-deprecated-data-object-declaration-good"></span>

**Good**

`data()` returns a new `{ count: 0 }` object, providing the function-based data declaration required by Vue 3.

```vue
<script lang="ts">
export default {
data() {
return { count: 0 }
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) · [All rules](all.md)

<span id="script-no-deprecated-destroyed-lifecycle"></span>

### `script/no-deprecated-destroyed-lifecycle`

Disallow deprecated destroyed and beforeDestroy lifecycle hooks

[Bad](#script-no-deprecated-destroyed-lifecycle-bad) · [Good](#script-no-deprecated-destroyed-lifecycle-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: Available for supported findings  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-destroyed-lifecycle": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

**Bad**

`beforeDestroy` is the removed Vue 2 lifecycle option used for the timer cleanup.

```vue
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

**Good**

Renaming the hook to `beforeUnmount` preserves the cleanup body under its Vue 3 lifecycle name.

```vue
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) · [All rules](all.md)

<span id="script-no-deprecated-dollar-listeners-api"></span>

### `script/no-deprecated-dollar-listeners-api`

Disallow the $listeners instance property removed in Vue 3 (merged into $attrs)

[Bad](#script-no-deprecated-dollar-listeners-api-bad) · [Good](#script-no-deprecated-dollar-listeners-api-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

**Bad**

The member reads and the bare argument reference all use `$listeners`, which Vue 3 removed after merging listeners into attributes.

```vue
<script setup lang="ts">
const handlers = this.$listeners
const forwarded = ctx.$listeners
emit('input', $listeners)
</script>
```

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

**Good**

The reads move to `this.$attrs` and setup-context `ctx.attrs`. These replace the removed listener surface; the illustrated receivers must exist in the surrounding component context.

```vue
<script setup lang="ts">
const handlers = this.$attrs
const forwarded = ctx.attrs
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) · [All rules](all.md)

<span id="script-no-deprecated-dollar-scopedslots-api"></span>

### `script/no-deprecated-dollar-scopedslots-api`

Disallow the $scopedSlots instance property removed in Vue 3 (use $slots)

[Bad](#script-no-deprecated-dollar-scopedslots-api-bad) · [Good](#script-no-deprecated-dollar-scopedslots-api-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-scopedslots-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

**Bad**

`this.$scopedSlots`, `ctx.$scopedSlots`, and the bare `$scopedSlots` reference use the Vue 2 scoped-slot API removed in Vue 3.

```vue
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

**Good**

Replacing `$scopedSlots` with `$slots` uses the unified slot surface. The example removes the deprecated spelling rather than establishing a setup context for the receivers.

```vue
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) · [All rules](all.md)

<span id="script-no-deprecated-events-api"></span>

### `script/no-deprecated-events-api`

Disallow the removed Vue 2 events API ($on / $off / $once)

[Bad](#script-no-deprecated-events-api-bad) · [Good](#script-no-deprecated-events-api-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-events-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-events-api-bad"></span>

**Bad**

The `$on`, `$once`, and `$off` calls use the instance event-bus methods removed in Vue 3.

```vue
<script setup lang="ts">
this.$on('event', handler)
this.$once('event', handler)
this.$off('event', handler)
emitter.$off('event')
</script>
```

<span id="script-no-deprecated-events-api-good"></span>

**Good**

`$emit` remains valid, while event-bus subscription moves to the external emitter’s `on` method. The repair separates parent-directed emission from an external event bus.

```vue
<script setup lang="ts">
// $emit is still valid in Vue 3
this.$emit('event', payload)

// Use an external emitter instead
import mitt from 'mitt'
const emitter = mitt()
emitter.on('event', handler)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) · [All rules](all.md)

<span id="script-no-deprecated-props-default-this"></span>

### `script/no-deprecated-props-default-this`

Disallow `this` inside a prop default/validator function (removed in Vue 3)

[Bad](#script-no-deprecated-props-default-this-bad) · [Good](#script-no-deprecated-props-default-this-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-props-default-this": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-props-default-this-bad"></span>

**Bad**

The prop default and validator read `this`, but those functions cannot rely on the component instance in Vue 3.

```vue
<script lang="ts">
export default {
props: {
size: {
type: Number,
// `this` is not the component instance in Vue 3.
default() {
return this.defaultSize
}
},
value: {
type: Number,
validator() {
return this.value > 0
}
}
}
}
</script>
```

<span id="script-no-deprecated-props-default-this-good"></span>

**Good**

The default reads `props.baseSize` from its argument, and the validator tests its `value` argument. Both stop depending on an unavailable instance receiver.

```vue
<script lang="ts">
export default {
props: {
size: {
type: Number,
// Vue 3 passes the raw props as the first argument instead.
default(props) {
return props.baseSize
}
},
value: {
type: Number,
validator(value) {
return value > 0
}
}
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) · [All rules](all.md)

<span id="script-no-dupe-keys"></span>

### `script/no-dupe-keys`

Disallow duplicate keys across Options API props/data/computed/methods/setup/inject

[Bad](#script-no-dupe-keys-bad) · [Good](#script-no-dupe-keys-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-dupe-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-dupe-keys-bad"></span>

**Bad**

`foo` is declared by both props and data, and `bar` by both computed and methods. Those declarations compete for the same component instance keys.

```vue
<script lang="ts">
export default {
props: ['foo'],
data() {
return { foo: 1 } // duplicate of prop `foo`
},
computed: {
bar() { return 2 }
},
methods: {
bar() {} // duplicate of computed `bar`
}
}
</script>
```

<span id="script-no-dupe-keys-good"></span>

**Good**

The prop, data, and computed declarations use distinct names (`foo`, `bar`, and `baz`), eliminating both cross-option collisions.

```vue
<script lang="ts">
export default {
props: ['foo'],
data() {
return { bar: 1 }
},
computed: {
baz() { return 2 }
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L52) · [All rules](all.md)

<span id="script-no-duplicate-attr-inheritance"></span>

### `script/no-duplicate-attr-inheritance`

Flag a component that applies its fallthrough attributes twice

[Bad](#script-no-duplicate-attr-inheritance-bad) · [Good](#script-no-duplicate-attr-inheritance-good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-duplicate-attr-inheritance-bad"></span>

**Bad**

The explicit `inheritAttrs: true` values restate Vue’s default. This rule reports that redundant literal even when no root `$attrs` spread is shown.

```vue
<script lang="ts">
defineOptions({ inheritAttrs: true })
export default { inheritAttrs: true }
</script>
```

<span id="script-no-duplicate-attr-inheritance-good"></span>

**Good**

`inheritAttrs: false` expresses a real opt-out, while the empty options object leaves default inheritance implicit. Neither restates the redundant `true` value.

```vue
<script lang="ts">
defineOptions({ inheritAttrs: false }) // intentional opt-out
export default {}                      // default inheritance, unstated
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) · [All rules](all.md)

<span id="script-no-export-in-script-setup"></span>

### `script/no-export-in-script-setup`

Disallow export statements inside &lt;script setup&gt;

[Bad](#script-no-export-in-script-setup-bad) · [Good](#script-no-export-in-script-setup-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-export-in-script-setup-bad"></span>

**Bad**

`export const count` attempts to expose a module export from `<script setup>`, where runtime exports are prohibited.

```vue
<script setup lang="ts">
export const count = 1;
</script>
```

<span id="script-no-export-in-script-setup-good"></span>

**Good**

Removing `export` keeps `count` as a setup binding rather than a module export.

```vue
<script setup lang="ts">
const count = 1;
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) · [All rules](all.md)

<span id="script-no-import-compiler-macros"></span>

### `script/no-import-compiler-macros`

Disallow importing Vue compiler macros that are auto-imported

[Bad](#script-no-import-compiler-macros-bad) · [Good](#script-no-import-compiler-macros-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-import-compiler-macros": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-import-compiler-macros-bad"></span>

**Bad**

The `vue` import includes `defineProps` and `defineEmits`, although these are compiler macros available directly in `<script setup>`.

```vue
<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

<span id="script-no-import-compiler-macros-good"></span>

**Good**

Removing the macro imports leaves both typed macro calls intact; no runtime import is needed for either declaration.

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) · [All rules](all.md)

<span id="script-no-internal-imports"></span>

### `script/no-internal-imports`

Disallow importing from Vue internal modules

[Bad](#script-no-internal-imports-bad) · [Good](#script-no-internal-imports-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-internal-imports": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-internal-imports-bad"></span>

**Bad**

Both imports address internal `dist` files rather than Vue’s public package entry point, coupling the component to build-file paths.

```vue
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

<span id="script-no-internal-imports-good"></span>

**Good**

Importing the required helpers from `vue` removes the dependency on internal distribution file locations.

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) · [All rules](all.md)

<span id="script-no-multiple-slot-args"></span>

### `script/no-multiple-slot-args`

Disallow passing more than one argument to a scoped-slot function call

[Bad](#script-no-multiple-slot-args-bad) · [Good](#script-no-multiple-slot-args-good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-multiple-slot-args": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-multiple-slot-args-bad"></span>

**Bad**

The slot calls pass multiple positional arguments or spread an unknown argument list. Vue slots receive one props object, not a positional parameter list.

```vue
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

<span id="script-no-multiple-slot-args-good"></span>

**Good**

`{ foo, bar }` combines the data into one argument; `slotProps` and the argument-free call also stay within the supported slot-call shape.

```vue
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) · [All rules](all.md)

<span id="script-no-potential-component-option-typo"></span>

### `script/no-potential-component-option-typo`

Flag likely typos in Options API component option names

[Bad](#script-no-potential-component-option-typo-bad) · [Good](#script-no-potential-component-option-typo-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-potential-component-option-typo-bad"></span>

**Bad**

The option is spelled `method`, one edit away from the recognized `methods` option; Vue would not treat it as the intended methods declaration.

```vue
<script lang="ts">
export default { method: { save() {} } };
</script>
```

<span id="script-no-potential-component-option-typo-good"></span>

**Good**

Changing the key to `methods` places `save()` under the recognized component option.

```vue
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) · [All rules](all.md)

<span id="script-no-reactive-destructure"></span>

### `script/no-reactive-destructure`

Disallow destructuring reactive objects which loses reactivity

[Bad](#script-no-reactive-destructure-bad) · [Good](#script-no-reactive-destructure-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reactive-destructure": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reactive-destructure-bad"></span>

**Bad**

`const { count, name } = state` copies primitive properties out of the `reactive` object, losing their connection to subsequent property changes.

```vue
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = state;
</script>
```

<span id="script-no-reactive-destructure-good"></span>

**Good**

Destructuring `toRefs(state)` creates refs for `count` and `name`, keeping each binding linked to the original reactive property.

```vue
<script setup lang="ts">
import { reactive, toRefs } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = toRefs(state);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) · [All rules](all.md)

<span id="script-no-ref-as-operand"></span>

### `script/no-ref-as-operand`

Require ref-bound variables to be accessed via `.value` when used as an operand

[Bad](#script-no-ref-as-operand-bad) · [Good](#script-no-ref-as-operand-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-ref-as-operand": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-ref-as-operand-bad"></span>

**Bad**

`count + 1` uses the ref object itself as the arithmetic operand instead of the number it wraps.

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

<span id="script-no-ref-as-operand-good"></span>

**Good**

`count.value + 1` reads the wrapped number before adding one; script arithmetic requires this explicit ref access.

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) · [All rules](all.md)

<span id="script-no-required-prop-with-default"></span>

### `script/no-required-prop-with-default`

Disallow a prop that is both required: true and has a default

[Bad](#script-no-required-prop-with-default-bad) · [Good](#script-no-required-prop-with-default-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-required-prop-with-default": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-required-prop-with-default-bad"></span>

**Bad**

`title` is both required and given the fallback `"Untitled"`, combining a required-input contract with a default intended for missing input.

```vue
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

<span id="script-no-required-prop-with-default-good"></span>

**Good**

Removing `required: true` makes `title` optional and leaves `"Untitled"` as its coherent fallback.

```vue
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) · [All rules](all.md)

<span id="script-no-reserved-identifiers"></span>

### `script/no-reserved-identifiers`

Disallow using Vue compiler reserved identifiers

[Bad](#script-no-reserved-identifiers-bad) · [Good](#script-no-reserved-identifiers-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-identifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-identifiers-bad"></span>

**Bad**

The bindings `__props`, `__emit`, and `__sfc__` use identifiers reserved for generated Vue compiler code.

```vue
<script setup lang="ts">
const __props = { name: "Ada" };
const __emit = () => {};
const __sfc__ = {};
</script>
```

<span id="script-no-reserved-identifiers-good"></span>

**Good**

The ordinary names `props`, `emit`, and `componentData` avoid those generated identifiers while retaining props and emits declarations.

```vue
<script setup lang="ts">
const props = defineProps<{ name: string }>();
const emit = defineEmits<{ save: [] }>();
const componentData = {};
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) · [All rules](all.md)

<span id="script-no-reserved-keys"></span>

### `script/no-reserved-keys`

Disallow Vue-reserved names as Options API props/data/computed/methods/setup/inject keys

[Bad](#script-no-reserved-keys-bad) · [Good](#script-no-reserved-keys-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-keys-bad"></span>

**Bad**

The returned data key `$el` collides with Vue’s built-in component-instance property and also uses a reserved `$` prefix.

```vue
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

<span id="script-no-reserved-keys-good"></span>

**Good**

Renaming the application data to `elementLabel` avoids the built-in instance surface and reserved prefix.

```vue
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) · [All rules](all.md)

<span id="script-no-reserved-props"></span>

### `script/no-reserved-props`

Disallow reserved names in a component's props declaration

[Bad](#script-no-reserved-props-bad) · [Good](#script-no-reserved-props-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-props-bad"></span>

**Bad**

The object-form `ref` and `$foo`, plus the array-form `key`, are reserved prop names. `ref` and `key` are framework controls, and `$`-prefixed names are rejected.

```vue
<script lang="ts">
export default {
props: {
ref: String,   // reserved
$foo: Number    // `$`-prefixed names are reserved
}
}

export default {
props: ['key']    // reserved (array form)
}
</script>
```

<span id="script-no-reserved-props-good"></span>

**Good**

The ordinary prop names `name` and `refValue` avoid the reserved names in both spelling and prefix.

```vue
<script lang="ts">
export default {
props: {
name: String,
refValue: Number
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) · [All rules](all.md)

<span id="script-no-restricted-globals"></span>

### `script/no-restricted-globals`

Disallow references to runtime-environment globals that must go through a typed wrapper

[Bad](#script-no-restricted-globals-bad) · [Good](#script-no-restricted-globals-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](options.md).

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-globals": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-restricted-globals-bad"></span>

**Bad**

The example reads the default restricted globals `process`, `localStorage`, and `sessionStorage` directly, bypassing the project’s explicit config and storage helpers.

```vue
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

<span id="script-no-restricted-globals-good"></span>

**Good**

`useFeatureFlag`, `authStorage.read`, and `viewStorage.write` remove those direct restricted-global references. The remaining `window.scrollY` is not a default restriction of this rule; SSR safety is a separate concern.

```vue
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) · [All rules](all.md)

<span id="script-no-restricted-members"></span>

### `script/no-restricted-members`

Disallow project-configured object.property member accesses

[Bad](#script-no-restricted-members-bad) · [Good](#script-no-restricted-members-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](options.md).

This example configures window.localStorage. The rule has no default deny list; enabling it alone does not report a member.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
            }
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

<span id="script-no-restricted-members-bad"></span>

**Bad**

With `{ object: "window", property: "localStorage" }` configured in `ruleOptions`, `window.localStorage` accesses the forbidden object/member pair. This rule has no default forbidden members.

```vue
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

<span id="script-no-restricted-members-good"></span>

**Good**

`authStorage.read("token")` delegates the read to the application’s storage helper and no longer accesses the configured `window.localStorage` member.

```vue
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) · [All rules](all.md)

<span id="script-no-side-effects-in-computed-properties"></span>

### `script/no-side-effects-in-computed-properties`

Disallow side effects in Options API computed getters

[Bad](#script-no-side-effects-in-computed-properties-bad) · [Good](#script-no-side-effects-in-computed-properties-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-side-effects-in-computed-properties": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-side-effects-in-computed-properties-bad"></span>

**Bad**

`doubled` assigns to `this.count`, and `reversed` mutates `this.items` through `reverse()`. Both getters modify the state they are supposed to derive from.

```vue
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
this.count = this.count * 2 // side effect: assigns to data
return this.count
},
reversed() {
return this.items.reverse() // side effect: mutates the array
}
}
}
</script>
```

<span id="script-no-side-effects-in-computed-properties-good"></span>

**Good**

`doubled` returns the multiplication without assignment. `reversed` copies the array before reversing it, so the original component state is unchanged by the getter.

```vue
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
return this.count * 2
},
reversed() {
return [...this.items].reverse() // operate on a copy
}
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) · [All rules](all.md)

<span id="script-no-top-level-ref-in-script"></span>

### `script/no-top-level-ref-in-script`

Disallow top-level ref/reactive to prevent Cross-Request State Pollution

[Bad](#script-no-top-level-ref-in-script-bad) · [Good](#script-no-top-level-ref-in-script-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-top-level-ref-in-script": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-top-level-ref-in-script-bad"></span>

**Bad**

The ordinary `<script>` initializes `count` and `user` at module scope. During SSR, these state objects can be shared across component instances and requests.

```vue
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
setup() {
return { count, user }
}
}
</script>
```

<span id="script-no-top-level-ref-in-script-good"></span>

**Good**

The setup ref is initialized per component instance; the ordinary script keeps only a constant, a state-producing function, and a ref created inside `setup()`. None creates reactive state at ordinary module scope.

```vue
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
return reactive({ count: 0 })
}

export default {
setup() {
// Create state inside setup
const count = ref(0)
return { count }
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) · [All rules](all.md)

<span id="script-no-unstable-nested-components"></span>

### `script/no-unstable-nested-components`

Disallow component definitions inside setup or render functions

[Bad](#script-no-unstable-nested-components-bad) · [Good](#script-no-unstable-nested-components-good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unstable-nested-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unstable-nested-components-bad"></span>

**Bad**

`defineComponent` runs inside the parent’s `setup()`, creating a new `Child` component definition whenever that setup executes.

```vue
<script lang="ts">
import { defineComponent } from "vue";
export default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };
</script>
```

<span id="script-no-unstable-nested-components-good"></span>

**Good**

The `Child` definition moves to module scope, and `setup()` returns that existing definition instead of recreating it.

```vue
<script lang="ts">
import { defineComponent } from "vue";
const Child = defineComponent({ render() { return null; } });
export default { setup() { return { Child }; } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) · [All rules](all.md)

<span id="script-no-unused-emit-declarations"></span>

### `script/no-unused-emit-declarations`

Flag declared events that are never emitted

[Bad](#script-no-unused-emit-declarations-bad) · [Good](#script-no-unused-emit-declarations-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unused-emit-declarations": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unused-emit-declarations-bad"></span>

**Bad**

`defineEmits` declares both `change` and `unused`, but the captured `emit` function only emits the literal event `change`.

```vue
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

<span id="script-no-unused-emit-declarations-good"></span>

**Good**

Removing `unused` makes the declared event list match the observed emission. The example uses a captured, unescaped emit binding so this local usage conclusion is available.

```vue
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [All rules](all.md)

<span id="script-no-use-computed-property-like-method"></span>

### `script/no-use-computed-property-like-method`

Disallow calling an Options API computed property like a method

[Bad](#script-no-use-computed-property-like-method-bad) · [Good](#script-no-use-computed-property-like-method-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-use-computed-property-like-method": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-use-computed-property-like-method-bad"></span>

**Bad**

`this.total()` calls the value exposed by the computed getter; the getter returns `3`, which is not callable.

```vue
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };
</script>
```

<span id="script-no-use-computed-property-like-method-good"></span>

**Good**

`this.total` reads the computed value without call parentheses, so `log` prints the derived number.

```vue
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) · [All rules](all.md)

<span id="script-no-with-defaults"></span>

### `script/no-with-defaults`

Discourage withDefaults in favor of destructuring defaults (Vue 3.5+)

[Bad](#script-no-with-defaults-bad) · [Good](#script-no-with-defaults-good)

Default severity: `warning`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-with-defaults": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-with-defaults-bad"></span>

**Bad**

`withDefaults` wraps the typed props declaration solely to supply `count` and `name` defaults, instead of the Vue 3.5+ destructuring-default style preferred here.

```vue
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

<span id="script-no-with-defaults-good"></span>

**Good**

The destructuring pattern puts `count = 0` and `name = "Ada"` beside their bindings and removes the `withDefaults` wrapper.

```vue
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [All rules](all.md)

<span id="script-prefer-computed"></span>

### `script/prefer-computed`

Prefer computed() for derived reactive state

[Bad](#script-prefer-computed-bad) · [Good](#script-prefer-computed-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

The watcher must only derive the destination. Editable copies and callbacks with other side effects are allowed.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-computed": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-computed-bad"></span>

**Bad**

The watcher only copies a derivation of `count` into a second ref, `doubled`, so the derived state is maintained through manual synchronization.

```vue
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

<span id="script-prefer-computed-good"></span>

**Good**

`computed(() => count.value * 2)` expresses the derivation directly and removes both the extra writable ref and its synchronization watcher.

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) · [All rules](all.md)

<span id="script-prefer-define-options"></span>

### `script/prefer-define-options`

Prefer defineOptions() over a plain &lt;script&gt; that only sets name/inheritAttrs

[Bad](#script-prefer-define-options-bad) · [Good](#script-prefer-define-options-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-define-options-bad"></span>

**Bad**

The plain script’s only meaningful statement exports an object containing just `name` and `inheritAttrs`; these options can be expressed by `defineOptions`.

```vue
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

<span id="script-prefer-define-options-good"></span>

**Good**

The shown `data()` method makes the script carry real Options API logic, so it falls outside this rule’s conservative options-only suggestion. This Good demonstrates an allowed exception; the direct migration would put `defineOptions({ name: 'MyComponent', inheritAttrs: false })` in `<script setup>`.

```vue
<script lang="ts">
// Real options logic — keep the plain script.
export default {
name: 'MyComponent',
data() { return { count: 0 } },
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) · [All rules](all.md)

<span id="script-prefer-import-from-vue"></span>

### `script/prefer-import-from-vue`

Prefer importing from 'vue' instead of internal packages

[Bad](#script-prefer-import-from-vue-bad) · [Good](#script-prefer-import-from-vue-good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-import-from-vue": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-import-from-vue-bad"></span>

**Bad**

`ref` and `h` are imported from the internal `@vue/runtime-core` and `@vue/runtime-dom` packages rather than the public `vue` package.

```vue
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

<span id="script-prefer-import-from-vue-good"></span>

**Good**

Both helpers are imported together from `vue`, using the public package entry point instead of either internal package.

```vue
<script setup lang="ts">
import { ref, h } from 'vue'
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) · [All rules](all.md)

<span id="script-prefer-ref-over-reactive"></span>

### `script/prefer-ref-over-reactive`

Recommend using ref() over reactive() for state management

[Bad](#script-prefer-ref-over-reactive-bad) · [Good](#script-prefer-ref-over-reactive-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-ref-over-reactive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-ref-over-reactive-bad"></span>

**Bad**

The state is created with `reactive`, contrary to this opinionated rule’s preference for refs. The example illustrates a style preference, not an inherently invalid reactive object.

```vue
<script setup lang="ts">
// reactive requires careful handling to avoid losing reactivity
const state = reactive({
count: 0,
name: 'foo'
})
</script>
```

<span id="script-prefer-ref-over-reactive-good"></span>

**Good**

The examples create both scalar and object state with `ref`; related fields may also be split into separate refs. This satisfies the preferred state-construction form.

```vue
<script setup lang="ts">
// ref is more explicit and safer
const count = ref(0)
const name = ref('foo')

// For objects, ref still works
const user = ref({ name: 'foo', age: 20 })

// Or use multiple refs for related data
const userName = ref('foo')
const userAge = ref(20)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) · [All rules](all.md)

<span id="script-prefer-use-attrs"></span>

### `script/prefer-use-attrs`

Recommend using useAttrs() over context.attrs

[Bad](#script-prefer-use-attrs-bad) · [Good](#script-prefer-use-attrs-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-attrs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-attrs-bad"></span>

**Bad**

`setup` obtains `attrs` by destructuring its context parameter, which this rule asks to replace with the Composition API helper.

```vue
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

<span id="script-prefer-use-attrs-good"></span>

**Good**

`useAttrs()` supplies `attrs` inside setup, retaining the `attrs.class` read without depending on the second setup parameter.

```vue
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) · [All rules](all.md)

<span id="script-prefer-use-id"></span>

### `script/prefer-use-id`

Recommend using useId() for generating unique IDs (Vue 3.5+)

[Bad](#script-prefer-use-id-bad) · [Good](#script-prefer-use-id-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-id-bad"></span>

**Bad**

`id` contains `Math.random()`, so the generated input/label identifier can differ between server and client rendering. Its ID-named binding is the rule’s recognized generation context.

```vue
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

<span id="script-prefer-use-id-good"></span>

**Good**

Vue 3.5+ `useId()` generates the identifier, and both `:for` and `:id` continue reading the same binding instead of independently generating random values.

```vue
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) · [All rules](all.md)

<span id="script-prefer-use-slots"></span>

### `script/prefer-use-slots`

Recommend using useSlots() over context.slots

[Bad](#script-prefer-use-slots-bad) · [Good](#script-prefer-use-slots-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-slots-bad"></span>

**Bad**

`setup` destructures `slots` from its context argument, the access form this rule prefers to replace.

```vue
<script lang="ts">
import { defineComponent, h } from "vue";
export default defineComponent({
  setup(_props, { slots }) { return () => h("div", slots.default?.()); },
});
</script>
```

<span id="script-prefer-use-slots-good"></span>

**Good**

`useSlots()` retrieves the slots inside setup, preserving the render function and its optional default-slot call without a context parameter.

```vue
<script lang="ts">
import { defineComponent, h, useSlots } from "vue";
export default defineComponent({
  setup() {
    const slots = useSlots();
    return () => h("div", slots.default?.());
  },
});
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) · [All rules](all.md)

<span id="script-prefer-use-template-ref"></span>

### `script/prefer-use-template-ref`

Recommend useTemplateRef over ref for template references (Vue 3.5+)

[Bad](#script-prefer-use-template-ref-bad) · [Good](#script-prefer-use-template-ref-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-template-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-template-ref-bad"></span>

**Bad**

The nullable `input` ref is paired with the template’s literal `ref="input"`, identifying it as an element reference rather than ordinary nullable data.

```vue
<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>
<template>
<input ref="input" />
</template>
```

<span id="script-prefer-use-template-ref-good"></span>

**Good**

Vue 3.5+ `useTemplateRef<HTMLInputElement>('input')` makes that template reference explicit. The unpaired `error = ref(null)` remains ordinary data and is intentionally outside this rule.

```vue
<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
// Paired with the template ref below.
const input = useTemplateRef<HTMLInputElement>('input')
// A nullable data ref the template never binds as a ref.
const error = ref(null)
</script>
<template>
<input ref="input" />
<p>{{ error }}</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) · [All rules](all.md)

<span id="script-require-default-prop"></span>

### `script/require-default-prop`

Require a default value for every optional, non-Boolean prop

[Bad](#script-require-default-prop-bad) · [Good](#script-require-default-prop-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-default-prop-bad"></span>

**Bad**

`name` and `age` are optional non-Boolean runtime props without defaults, leaving their omitted-input values unspecified.

```vue
<script lang="ts">
export default {
props: {
// optional, non-Boolean, no default
name: String,
age: { type: Number },
}
}
</script>
```

<span id="script-require-default-prop-good"></span>

**Good**

`name` receives `default: ''`. `enabled` uses Boolean’s implicit false default, and required `id` needs no fallback, illustrating both exemptions.

```vue
<script lang="ts">
export default {
props: {
name: { type: String, default: '' },
enabled: Boolean,                 // Boolean defaults to false
id: { type: Number, required: true },
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) · [All rules](all.md)

<span id="script-require-explicit-emits"></span>

### `script/require-explicit-emits`

Require emitted events to be declared in defineEmits or the emits option

[Bad](#script-require-explicit-emits-bad) · [Good](#script-require-explicit-emits-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-emits": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-emits-bad"></span>

**Bad**

The captured emit function emits `save`, but `defineEmits([])` declares no such event.

```vue
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

<span id="script-require-explicit-emits-good"></span>

**Good**

Adding `"save"` to the declaration makes the emitted literal event part of the component’s explicit event contract.

```vue
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) · [All rules](all.md)

<span id="script-require-explicit-slots"></span>

### `script/require-explicit-slots`

Require slots consumed via useSlots() to be explicitly typed with defineSlots&lt;...&gt;()

[Bad](#script-require-explicit-slots-bad) · [Good](#script-require-explicit-slots-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-slots-bad"></span>

**Bad**

The typed `defineProps<{ id: number }>()` establishes TypeScript syntax, but setup uses `useSlots()` without a `defineSlots` declaration. The rule therefore finds consumed slots without an explicit slot contract.

```vue
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

<span id="script-require-explicit-slots-good"></span>

**Good**

`defineSlots` declares a `default` slot whose props include `msg: string`; `useSlots()` now appears alongside an explicit typed slot contract.

```vue
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) · [All rules](all.md)

<span id="script-require-function-return-type"></span>

### `script/require-function-return-type`

Require return type annotations on functions

[Bad](#script-require-function-return-type-bad) · [Good](#script-require-function-return-type-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-function-return-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-function-return-type-bad"></span>

**Bad**

Both `add` and `greet` annotate their parameters but omit a return-type annotation; inferred returns do not satisfy this explicit-annotation policy.

```vue
<script setup lang="ts">
const add = (a: number, b: number) => {
return a + b
}

function greet(name: string) {
return `Hello, ${name}`
}
</script>
```

<span id="script-require-function-return-type-good"></span>

**Good**

`add` declares `: number`, and `greet` declares `: string`, making the return contracts explicit without changing either body.

```vue
<script setup lang="ts">
const add = (a: number, b: number): number => {
return a + b
}

function greet(name: string): string {
return `Hello, ${name}`
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) · [All rules](all.md)

<span id="script-require-prop-type-constructor"></span>

### `script/require-prop-type-constructor`

Require prop `type` values to be constructors rather than string literals

[Bad](#script-require-prop-type-constructor-bad) · [Good](#script-require-prop-type-constructor-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-type-constructor": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-type-constructor-bad"></span>

**Bad**

The prop declarations use strings `"String"` and `"Number"` as runtime types, including inside the constructor array. Those strings are not constructor functions.

```vue
<script lang="ts">
export default {
props: {
// The type should be the `String` constructor, not the string "String".
name: "String",
age: { type: "Number" },
id: { type: ["String", "Number"] }
}
}
</script>
```

<span id="script-require-prop-type-constructor-good"></span>

**Good**

The declarations use the actual `String` and `Number` identifiers, including the union array `[String, Number]`.

```vue
<script lang="ts">
export default {
props: {
name: String,
age: { type: Number },
id: { type: [String, Number] }
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L59) · [All rules](all.md)

<span id="script-require-prop-types"></span>

### `script/require-prop-types`

Require every prop to declare a type

[Bad](#script-require-prop-types-bad) · [Good](#script-require-prop-types-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-types": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-types-bad"></span>

**Bad**

The array entry declares only the name `status`; the `null` value and empty descriptor declare no runtime prop type either.

```vue
<script lang="ts">
export default {
props: ['status']            // array form: no types
}

export default {
props: {
status: null,              // no type
other: {}                  // empty descriptor: no type
}
}
</script>
```

<span id="script-require-prop-types-good"></span>

**Good**

`status: String` supplies a shorthand constructor, and `other` supplies `type: Number` inside its descriptor. Both props now carry type declarations.

```vue
<script lang="ts">
export default {
props: {
status: String,
other: { type: Number, default: 0 }
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) · [All rules](all.md)

<span id="script-require-symbol-provide"></span>

### `script/require-symbol-provide`

Recommend using Symbol as injection key for provide/inject

[Bad](#script-require-symbol-provide-bad) · [Good](#script-require-symbol-provide-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-symbol-provide-bad"></span>

**Bad**

`provide` and `inject` use literal string keys such as `'user'` and `'theme'`, which can collide with another provider using the same spelling.

```vue
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

<span id="script-require-symbol-provide-good"></span>

**Good**

The shared `UserKey` is created with `Symbol` and annotated as `InjectionKey<User>`; both calls pass that key instead of a literal string.

```vue
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) · [All rules](all.md)

<span id="script-require-typed-object-prop"></span>

### `script/require-typed-object-prop`

Require an explicit type on a prop whose runtime type is `Object` or `Array`

[Bad](#script-require-typed-object-prop-bad) · [Good](#script-require-typed-object-prop-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-object-prop-bad"></span>

**Bad**

Bare `Object` and `Array` constructors describe only broad runtime categories, so neither `user` nor the `items` element shape has an explicit static type.

```vue
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

<span id="script-require-typed-object-prop-good"></span>

**Good**

`PropType<User>` and `PropType<User[]>` add the object and element types while retaining the same runtime constructors.

```vue
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [All rules](all.md)

<span id="script-require-typed-ref"></span>

### `script/require-typed-ref`

Require an explicit type argument on a ref() initialized with no value, null, or undefined

[Bad](#script-require-typed-ref-bad) · [Good](#script-require-typed-ref-good)

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-ref-bad"></span>

**Bad**

The imported `ref` calls have neither a type argument nor a useful initial value: no argument, `null`, and `undefined` cannot infer the intended future value type.

```vue
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

<span id="script-require-typed-ref-good"></span>

**Good**

Explicit type arguments describe the string and nullable User refs. `ref(0)` already has a concrete numeric initializer and can rely on inference.

```vue
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) · [All rules](all.md)

<span id="script-require-valid-default-prop"></span>

### `script/require-valid-default-prop`

Require a prop's default value to be valid for its declared type

[Bad](#script-require-valid-default-prop-bad) · [Good](#script-require-valid-default-prop-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-valid-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-valid-default-prop-bad"></span>

**Bad**

The Number and Boolean props receive mismatched scalar defaults, and the Array and Object props use shared literal values instead of factories.

```vue
<script lang="ts">
export default {
props: {
count: { type: Number, default: '0' },     // string default for Number
enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
items: { type: Array, default: [] },        // literal must be a factory
config: { type: Object, default: {} }       // literal must be a factory
}
}
</script>
```

<span id="script-require-valid-default-prop-good"></span>

**Good**

The scalar defaults become `0` and `false`; the array and object defaults become functions returning fresh values. The `[String, Number]` example accepts its string default because it matches one declared type.

```vue
<script lang="ts">
export default {
props: {
count: { type: Number, default: 0 },
enabled: { type: Boolean, default: false },
items: { type: Array, default: () => [] },
config: { type: Object, default: () => ({}) },
label: { type: [String, Number], default: '' }
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) · [All rules](all.md)

<span id="script-return-in-computed-property"></span>

### `script/return-in-computed-property`

Require a return value in every computed getter

[Bad](#script-return-in-computed-property-bad) · [Good](#script-return-in-computed-property-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-computed-property": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-computed-property-bad"></span>

**Bad**

The block-bodied computed getter evaluates `1 + 2` but never returns it, leaving the computed value undefined.

```vue
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { 1 + 2; });
</script>
```

<span id="script-return-in-computed-property-good"></span>

**Good**

`return 1 + 2` turns the expression into the getter’s returned value. The rule looks for a value-returning return in the getter itself, not merely an expression statement.

```vue
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { return 1 + 2; });
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) · [All rules](all.md)

<span id="script-return-in-emits-validator"></span>

### `script/return-in-emits-validator`

Require a return value in every Options API emits validator

[Bad](#script-return-in-emits-validator-bad) · [Good](#script-return-in-emits-validator-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

Use a block-body arrow for the currently supported SFC filter. The underlying validator also handles method shorthand, but the current SFC prefilter does not reliably dispatch that shape.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-emits-validator": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-emits-validator-bad"></span>

**Bad**

The `submit` validator logs the payload but does not return a validation result, so its block body yields undefined.

```vue
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

<span id="script-return-in-emits-validator-good"></span>

**Good**

`return payload != null` supplies a boolean validation result for the submitted payload instead of ending without a returned value.

```vue
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) · [All rules](all.md)

<span id="script-valid-define-emits"></span>

### `script/valid-define-emits`

Enforce valid defineEmits() usage (no type+runtime args, no local references, single call)

[Bad](#script-valid-define-emits-bad) · [Good](#script-valid-define-emits-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-emits": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-emits-bad"></span>

**Bad**

The same `defineEmits` call supplies both a type argument and the runtime array `["save"]`, mixing two mutually exclusive declarations.

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

<span id="script-valid-define-emits-good"></span>

**Good**

Removing the runtime argument leaves a single type-based event declaration for `save`.

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) · [All rules](all.md)

<span id="script-valid-define-options"></span>

### `script/valid-define-options`

Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots)

[Bad](#script-valid-define-options-bad) · [Good](#script-valid-define-options-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-options": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-options-bad"></span>

**Bad**

The first call puts the dedicated `props` declaration inside `defineOptions`; the later calls also repeat the macro and include a non-object argument. These illustrate the forbidden shape and repeated-call constraints.

```vue
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

<span id="script-valid-define-options-good"></span>

**Good**

One `defineOptions` call receives an object containing only the supported ordinary options `name` and `inheritAttrs`.

```vue
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) · [All rules](all.md)

<span id="script-valid-define-props"></span>

### `script/valid-define-props`

Enforce valid defineProps() usage (single call, not both type and runtime args, no local references)

[Bad](#script-valid-define-props-bad) · [Good](#script-valid-define-props-good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-props-bad"></span>

**Bad**

The same `defineProps` call supplies both `{ title: string }` as a type argument and `{ title: String }` as a runtime argument, which the compiler does not permit together.

```vue
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

<span id="script-valid-define-props-good"></span>

**Good**

Removing the runtime object leaves one type-based declaration for `title` instead of combining both declaration forms.

```vue
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) · [All rules](all.md)

<span id="script-valid-next-tick"></span>

### `script/valid-next-tick`

Require the result of a nextTick() call to be awaited, chained, or given a callback

[Bad](#script-valid-next-tick-bad) · [Good](#script-valid-next-tick-good)

Default severity: `warning`  
Presets: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-next-tick": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-next-tick-bad"></span>

**Bad**

The imported `nextTick()` is a bare expression with no callback, so its returned Promise is ignored and no work waits for the DOM flush.

```vue
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

<span id="script-valid-next-tick-good"></span>

**Good**

`await nextTick()` consumes the Promise and explicitly waits for the next DOM update before subsequent setup code continues.

```vue
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) · [All rules](all.md)

<span id="nuxt-no-nuxt-config-test-key"></span>

### `nuxt/no-nuxt-config-test-key`

Disallow setting `test` key in Nuxt config

[Bad](#nuxt-no-nuxt-config-test-key-bad) · [Good](#nuxt-no-nuxt-config-test-key-good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: Nuxt configuration files (nuxt.config.ts)  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

**Bad**

The exported Nuxt config sets the identifier key `test` to the boolean `true`, the obsolete config shape this rule rejects.

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ test: true });
```

<span id="nuxt-no-nuxt-config-test-key-good"></span>

**Good**

The empty config removes that boolean `test` property. This example does not forbid a test configuration object.

`nuxt.config.ts`

```ts
export default defineNuxtConfig({});
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [All rules](all.md)

<span id="nuxt-no-page-meta-runtime-values"></span>

### `nuxt/no-page-meta-runtime-values`

Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup

[Bad](#nuxt-no-page-meta-runtime-values-bad) · [Good](#nuxt-no-page-meta-runtime-values-good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

**Bad**

`useRoute()` is evaluated immediately while building the `definePageMeta` object, although the macro hoists that metadata outside the setup runtime context.

```vue
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

<span id="nuxt-no-page-meta-runtime-values-good"></span>

**Good**

`validate` receives a callback, so its `useRoute().params.id` access is deferred until the callback runs. The rule distinguishes deferred function bodies from eager metadata values.

```vue
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [All rules](all.md)

<span id="nuxt-nuxt-config-keys-order"></span>

### `nuxt/nuxt-config-keys-order`

Prefer recommended order of Nuxt config properties

[Bad](#nuxt-nuxt-config-keys-order-bad) · [Good](#nuxt-nuxt-config-keys-order-good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: Available for supported findings  
Applies to: Nuxt configuration files (nuxt.config.ts)  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-nuxt-config-keys-order-bad"></span>

**Bad**

The config places `ssr` before `modules`, reversing their order in the rule’s recommended Nuxt config key sequence.

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ ssr: true, modules: [] });
```

<span id="nuxt-nuxt-config-keys-order-good"></span>

**Good**

Putting `modules` before `ssr` preserves both values while satisfying the prescribed order; the repair changes layout rather than either option’s meaning.

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ modules: [], ssr: true });
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [All rules](all.md)

<span id="nuxt-prefer-import-meta"></span>

### `nuxt/prefer-import-meta`

Prefer using `import.meta.*` over `process.*`

[Bad](#nuxt-prefer-import-meta-bad) · [Good](#nuxt-prefer-import-meta-good)

Default severity: `error`  
Presets: `nuxt`  
Automatic fix: Available for supported findings  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-prefer-import-meta-bad"></span>

**Bad**

`process.client` uses a legacy Nuxt environment flag that the rule asks to migrate to `import.meta`.

```vue
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

<span id="nuxt-prefer-import-meta-good"></span>

**Good**

`import.meta.client` keeps the browser-only branch explicit using the replacement environment flag.

```vue
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [All rules](all.md)


## Project examples

<span id="ecosystem-vue-router-unknown-route"></span>

### `ecosystem/vue-router-unknown-route`

The name is absent from the complete installed router.

Default severity: error  
Applies to: Reachable project declarations and imported components  
Options: crossFile; rule severity (off/warn/error)  
Automatic fix: None

The complete installed router must be reachable from the application's createApp(...).use(router). Unknown/dynamic route tables do not prove unknown-name findings. Missing params are warnings because navigation may inherit a value from the current route.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-unknown-route": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-unknown-route-bad"></span>

**Bad**

The reachable installed router declares `user-post`, but navigation uses the misspelled `user-posts` name.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-posts", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-unknown-route-good"></span>

**Good**

Use the registered `user-post` name while retaining both declared path parameters.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Cross-file index](cross-file.md)

<span id="ecosystem-vue-router-extra-param"></span>

### `ecosystem/vue-router-extra-param`

The route does not declare tab; Vue Router discards it.

Default severity: error  
Applies to: Reachable project declarations and imported components  
Options: crossFile; rule severity (off/warn/error)  
Automatic fix: None

The complete installed router must be reachable from the application's createApp(...).use(router). Unknown/dynamic route tables do not prove unknown-name findings. Missing params are warnings because navigation may inherit a value from the current route.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-extra-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-extra-param-bad"></span>

**Bad**

The `user-post` path declares `userId` and `postId`, but the navigation also supplies undeclared `tab` as a path parameter.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2", tab: "a" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-extra-param-good"></span>

**Good**

Remove `tab` from params and retain only the keys present in the route path. Use query separately if the application needs a tab selection.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Cross-file index](cross-file.md)

<span id="ecosystem-vue-router-param-type"></span>

### `ecosystem/vue-router-param-type`

postId is not repeatable, so an array is invalid.

Default severity: error  
Applies to: Reachable project declarations and imported components  
Options: crossFile; rule severity (off/warn/error)  
Automatic fix: None

The complete installed router must be reachable from the application's createApp(...).use(router). Unknown/dynamic route tables do not prove unknown-name findings. Missing params are warnings because navigation may inherit a value from the current route.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-param-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-param-type-bad"></span>

**Bad**

`postId` is a scalar path parameter, but the navigation gives it the array `["2"]`.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: ["2"] } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-param-type-good"></span>

**Good**

Pass the scalar `"2"` for the non-repeatable `postId` segment.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Cross-file index](cross-file.md)

<span id="ecosystem-vue-router-missing-param"></span>

### `ecosystem/vue-router-missing-param`

Required postId is missing; relying on the current route is fragile.

Default severity: warning  
Applies to: Reachable project declarations and imported components  
Options: crossFile; rule severity (off/warn/error)  
Automatic fix: None

The complete installed router must be reachable from the application's createApp(...).use(router). Unknown/dynamic route tables do not prove unknown-name findings. Missing params are warnings because navigation may inherit a value from the current route.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-missing-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-missing-param-bad"></span>

**Bad**

The navigation omits required `postId` from the `user-post` path. This is a warning because Vue Router may inherit a value from the current route.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-missing-param-good"></span>

**Good**

Pass both `userId` and `postId` explicitly so navigation does not depend on the current route's parameter state.

`src/UserPost.vue`

```vue
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Cross-file index](cross-file.md)

<span id="html-cross-component-nesting"></span>

### `html/cross-component-nesting`

Check actual HTML nesting after imported components are composed.

Default severity: warning  
Applies to: Reachable project declarations and imported components  
Options: crossFile; rule severity (off/warn/error)  
Automatic fix: None

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "html/cross-component-nesting": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="html-cross-component-nesting-bad"></span>

**Bad**

The parent's `<p>` contains a resolved child whose root is `<div>`, producing invalid paragraph/block nesting after composition.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><p><Child /></p></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

<span id="html-cross-component-nesting-good"></span>

**Good**

Use a `<section>` container that can contain the child's block element; the child stays unchanged.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><section><Child /></section></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Cross-file index](cross-file.md)

<span id="vue-cross-file-attrs-fallthrough"></span>

### `vue/cross-file-attrs-fallthrough`

A parent passes attributes to a resolved child whose root cannot inherit them and does not explicitly use $attrs.

Default severity: warning  
Applies to: Reachable project declarations and imported components  
Options: crossFile; rule severity (off/warn/error)  
Automatic fix: None

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "vue/cross-file-attrs-fallthrough": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vue-cross-file-attrs-fallthrough-bad"></span>

**Bad**

The parent passes `class="notice"` to a resolved fragment child that has no automatic attribute target and never reads `$attrs`.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vue-cross-file-attrs-fallthrough-good"></span>

**Good**

The child chooses `<main>` as its target by binding `$attrs` there; its sibling `<aside>` remains separate.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-array-mutation"></span>

### `vize:croquis/cf/array-mutation`

An array is mutated by index, which a reactive array does not track.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

Historical Vue 2.7 only: use matching Vue 2.7 and SFC compiler dependencies for this scenario. Vue 3 proxies track array index assignment, so `items[0] = next` is reactive in Vue 3 and is not a Vue 3 defect. This published code has no current producer.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import Vue from 'vue';
import App from './App.vue';
new Vue({ render: h => h(App) }).$mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script lang="ts">
import Vue from 'vue';
import { replaceFirst } from './replace-first';
export default Vue.extend({
  data() { return { items: ['Before'] }; },
  methods: { replace() { replaceFirst(this.items, 'After'); } },
});
</script>
<template><section><p>{{ items[0] }}</p><button @click="replace">Replace</button></section></template>

```

<span id="vize-croquis-cf-array-mutation-bad"></span>

**Bad**

In this historical Vue 2.7 project, `items[0] = next` changes the array without notifying Vue 2’s array observer, so the displayed first item need not update.

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items[0] = next;
}

```

<span id="vize-croquis-cf-array-mutation-good"></span>

**Good**

`splice(0, 1, next)` uses the array mutation method observed by Vue 2, allowing the same replacement to update the view.

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items.splice(0, 1, next);
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-async-boundary"></span>

### `vize:croquis/cf/async-boundary`

Reactive state crosses an async boundary and can be observed stale.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/async-boundary": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./SearchPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export interface Result { items: string[]; }
export async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {
  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);
  return response.json();
}
```

<span id="vize-croquis-cf-async-boundary-bad"></span>

**Bad**

A slower old query can finish after a newer query and overwrite `result`, because the watcher has no invalidation cleanup.

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value) => {
    result.value = await load(value);
  },
);
</script>
```

<span id="vize-croquis-cf-async-boundary-good"></span>

**Good**

Register cleanup before awaiting: abort the old request and invalidate its `active` flag, then assign only a still-active response.

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-async-no-suspense"></span>

### `vize:croquis/cf/async-no-suspense`

An async component is rendered without a Suspense boundary.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

Current support: `no-source-async-fact`

The boundary producer reads macros.is_async(), but source parsing currently records top-level await on the script-setup scope instead. The complete Bad/Good source pair below therefore produces no async-no-suspense finding through the current CLI. It explains the Suspense convention; supplying the missing macro fact is implementation follow-up work.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-async-no-suspense-bad"></span>

**Bad**

The child has top-level await but its parent supplies no `<Suspense>` boundary. Current source parsing does not supply the macro fact required to emit this code.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

<span id="vize-croquis-cf-async-no-suspense-good"></span>

**Good**

The parent wraps the same async child in `<Suspense>` with a loading fallback. This demonstrates the convention; both source alternatives remain non-emitted by the current pass.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Suspense><Child /><template #fallback><p>Loading</p></template></Suspense></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-browser-api-ssr"></span>

### `vize:croquis/cf/browser-api-ssr`

A browser-only API is used where the component can render on the server.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/browser-api-ssr": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-browser-api-ssr-bad"></span>

**Bad**

`window.innerWidth` runs during setup, where an SSR environment has no browser `window`.

`App.vue`

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-browser-api-ssr-good"></span>

**Good**

Initialize a ref to a server-safe value and read `window` inside `onMounted`, which runs after client mounting.

`App.vue`

```vue
<script setup lang="ts">
import { onMounted, ref } from "vue";
const width = ref(0);
onMounted(() => { width.value = window.innerWidth; });
</script>
<template><p>Content</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-circular-dep"></span>

### `vize:croquis/cf/circular-dep`

Components import each other in a cycle.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

This illustrates a concrete eager-initialization cycle. A recursive Vue component or every circular import is not automatically erroneous. No current producer emits this contract code.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { aLabel } from './a';
</script>

<template>
<p>{{ aLabel }}</p>
</template>

```

`labels.ts`

```ts
export const aPrefix = 'A';
export const bPrefix = 'B';

```

<span id="vize-croquis-cf-circular-dep-bad"></span>

**Bad**

`a.ts` imports `b.ts`, which imports `a.ts` back. Both eagerly initialize a constant from the other module’s still-uninitialized constant, creating a temporal-dead-zone failure.

`a.ts`

```ts
import { bLabel } from './b';
export const aLabel = 'A' + bLabel;

```

`b.ts`

```ts
import { aLabel } from './a';
export const bLabel = 'B' + aLabel;

```

<span id="vize-croquis-cf-circular-dep-good"></span>

**Good**

Both modules read initialized prefixes from the independent `labels.ts` module, removing the cycle and the eager cross-read.

`a.ts`

```ts
import { bPrefix } from './labels';
export const aLabel = 'A' + bPrefix;

```

`b.ts`

```ts
import { aPrefix } from './labels';
export const bLabel = 'B' + aPrefix;

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-circular-reactive-dependency"></span>

### `vize:croquis/cf/circular-reactive-dependency`

Reactive computations depend on each other in a cycle.

Default severity: context-dependent  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/circular-reactive-dependency": "warn" },
    },
  },
});
```

```sh
vp run lint
```

Example qualification: `illustrative-source-pair`

The complete Vue project below illustrates update feedback and its repair. It is not a qualified CLI finding witness: the diagnostic producer requires retained reactive-flow reference identities and edges, as shown by the accompanying graph. These sources do not establish that the current source path will emit this exact code. Dedicated tracked-ID graph finding controls remain separate from source grammar checks.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`count-key.ts`

```ts
import type { InjectionKey, Ref } from 'vue';
export const countKey: InjectionKey<Ref<number>> = Symbol('count');
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from 'vue';
import { countKey } from './count-key';
import CycleView from './CycleView.vue';
const count = ref(1); // A: the provider-owned source.
provide(countKey, count);
</script>
<template>
  <button @click="count++">Increment</button>
  <CycleView />
</template>
```

<span id="vize-croquis-cf-circular-reactive-dependency-bad"></span>

**Bad**

App owns and provides count (A). CycleView derives nextCount (B), then immediately writes each derived value back into the same injected count. Every write changes the input to the computation again, creating update feedback A → B → A. The identities in the retained graph below represent these two references, not unrelated bindings with matching names.

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject, watch } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
watch(nextCount, value => { count.value = value; }, { immediate: true });
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B; B -> A
```

<span id="vize-croquis-cf-circular-reactive-dependency-good"></span>

**Good**

Remove the watcher that writes B back into A. App keeps ownership of count and changes it only through its explicit Increment action; CycleView reads the derived nextCount without feeding the result back. The same references retain only the A → B dependency.

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-closure-captures-reactive"></span>

### `vize:croquis/cf/closure-captures-reactive`

A closure captures a reactive value and will not see later updates.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { computed, ref } from 'vue';
import { makeReader } from './reader';
const count = ref(0);
const read = makeReader(count);
const shown = computed(read);
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ shown }}</p>
</template>

```

<span id="vize-croquis-cf-closure-captures-reactive-bad"></span>

**Bad**

`makeReader` copies `count.value` before creating the closure. The computed reader then returns that initial number without reading a reactive dependency.

`reader.ts`

```ts
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  const captured = count.value;
  return () => captured;
}

```

<span id="vize-croquis-cf-closure-captures-reactive-good"></span>

**Good**

The closure reads `count.value` when invoked, so the computed getter can track the ref and update `shown` after increments.

`reader.ts`

```ts
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  return () => count.value;
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-composable-outside-setup"></span>

### `vize:croquis/cf/composable-outside-setup`

A composable is called outside `setup`.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

The concern is this lifecycle-dependent composable, not a blanket ban on ordinary utility functions or all Composition API calls outside setup. This contract has no current producer.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useTitle } from './use-title';
const title = useTitle();
</script>

<template>
<h1>{{ title }}</h1>
</template>

```

<span id="vize-croquis-cf-composable-outside-setup-bad"></span>

**Bad**

Importing `use-title.ts` registers `onMounted` before a component setup is active. Calling its exported function later only returns that module-level ref; it cannot repair the missed lifecycle ownership.

`use-title.ts`

```ts
import { onMounted, ref } from 'vue';
const title = ref('Before mount');
onMounted(() => { title.value = 'Mounted'; });
export function useTitle() { return title; }

```

<span id="vize-croquis-cf-composable-outside-setup-good"></span>

**Good**

Both state creation and hook registration move into `useTitle`, which App calls synchronously inside setup. The mounted hook now belongs to that App instance.

`use-title.ts`

```ts
import { onMounted, ref } from 'vue';
export function useTitle() {
  const title = ref('Before mount');
  onMounted(() => { title.value = 'Mounted'; });
  return title;
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-computed-side-effects"></span>

### `vize:croquis/cf/computed-side-effects`

A computed getter writes state or performs another side effect.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useDouble } from './use-double';
const { count, doubled, lastCalculated } = useDouble();
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ doubled }} / {{ lastCalculated }}</p>
</template>

```

<span id="vize-croquis-cf-computed-side-effects-bad"></span>

**Bad**

Evaluating `doubled` writes `lastCalculated`, so reading a computed value also mutates separate state. That couples the side effect to when the lazy getter is read.

`use-double.ts`

```ts
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const lastCalculated = ref(0);
  const doubled = computed(() => {
    const next = count.value * 2;
    lastCalculated.value = next;
    return next;
  });
  return { count, doubled, lastCalculated };
}

```

<span id="vize-croquis-cf-computed-side-effects-good"></span>

**Good**

The getter only returns the derived number. A separate watcher owns the write to `lastCalculated` when `count` changes, including its initial value.

`use-double.ts`

```ts
import { computed, ref, watch } from 'vue';
export function useDouble() {
  const count = ref(0);
  const lastCalculated = ref(0);
  const doubled = computed(() => count.value * 2);
  watch(count, next => { lastCalculated.value = next * 2; }, { immediate: true });
  return { count, doubled, lastCalculated };
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-deep-import"></span>

### `vize:croquis/cf/deep-import`

An import chain is deeper than the project allows.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

This is an explicitly chosen project layout policy; it does not invent a supported depth threshold or option. There is no current diagnostic producer for this contract.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { label } from './entry';
</script>

<template>
<p>{{ label }}</p>
</template>

```

`value.ts`

```ts
export const label = 'Notice';

```

`level-one.ts`

```ts
export { label } from './level-two';

```

`level-two.ts`

```ts
export { label } from './level-three';

```

`level-three.ts`

```ts
export { label } from './value';

```

`public-api.ts`

```ts
export { label } from './value';

```

<span id="vize-croquis-cf-deep-import-bad"></span>

**Bad**

The entry routes a simple value through `level-one`, `level-two`, and `level-three`, creating an unnecessarily deep import chain for a project that wants a shallow public boundary.

`entry.ts`

```ts
export { label } from './level-one';

```

<span id="vize-croquis-cf-deep-import-good"></span>

**Good**

The entry uses `public-api.ts`, which re-exports the value directly. The consumer keeps the same imported name while the chain becomes shorter.

`entry.ts`

```ts
export { label } from './public-api';

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-destructuring-breaks-reactivity"></span>

### `vize:croquis/cf/destructuring-breaks-reactivity`

Destructuring a reactive object copies the fields and drops tracking.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/destructuring-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-destructuring-breaks-reactivity-bad"></span>

**Bad**

Ordinary destructuring of the `props` object copies its current `item` value; this is separate from direct Vue 3.5 `defineProps()` destructuring.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const { item } = props;
</script>
```

<span id="vize-croquis-cf-destructuring-breaks-reactivity-good"></span>

**Good**

`toRef(props, "item")` retains the connection to the property on `props`.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-di-outside-setup"></span>

### `vize:croquis/cf/di-outside-setup`

`provide` or `inject` is called outside `setup`.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

This example uses component provide/inject. `app.provide` and supported `app.runWithContext` injection are different valid ownership surfaces, not prohibited by this scenario. No current producer emits this contract code.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`theme.ts`

```ts
import { inject, provide } from 'vue';
import type { InjectionKey } from 'vue';
export const ThemeKey: InjectionKey<string> = Symbol('theme');
export function provideTheme() { provide(ThemeKey, 'dark'); }
export function useTheme() { return inject(ThemeKey, 'light'); }

```

`ThemedText.vue`

```vue
<script setup lang="ts">
import { useTheme } from './theme';
const theme = useTheme();
</script>

<template>
<p>{{ theme }}</p>
</template>

```

<span id="vize-croquis-cf-di-outside-setup-bad"></span>

**Bad**

`main.ts` calls component `provide` with no active component instance. The child’s `inject` therefore cannot receive this intended ancestor value and uses `light`.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
import { provideTheme } from './theme';
provideTheme();
createApp(App).mount('#app');

```

`App.vue`

```vue
<script setup lang="ts">
import ThemedText from './ThemedText.vue';
</script>

<template>
<ThemedText />
</template>

```

<span id="vize-croquis-cf-di-outside-setup-good"></span>

**Good**

App calls the provider from its setup before rendering the child. The child now inherits the `dark` value from its component ancestor.

`App.vue`

```vue
<script setup lang="ts">
import ThemedText from './ThemedText.vue';
import { provideTheme } from './theme';
provideTheme();
</script>

<template>
<ThemedText />
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-dom-access-without-next-tick"></span>

### `vize:croquis/cf/dom-access-without-next-tick`

The DOM is read before Vue has flushed the update.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`read-label.ts`

```ts
export function readLabel(node: HTMLElement | null): string {
  return node?.textContent ?? '';
}

```

<span id="vize-croquis-cf-dom-access-without-next-tick-bad"></span>

**Bad**

The click handler increments `count` and immediately reads the rendered paragraph, before Vue flushes the scheduled DOM update. `sampled` can contain the previous count.

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { readLabel } from './read-label';
const count = ref(0);
const label = ref<HTMLElement | null>(null);
const sampled = ref('');
function increment() {
  count.value++;
  sampled.value = readLabel(label.value);
}
</script>

<template>
<button @click="increment">Increment</button><p ref="label">{{ count }}</p><p>DOM sample: {{ sampled }}</p>
</template>

```

<span id="vize-croquis-cf-dom-access-without-next-tick-good"></span>

**Good**

Awaiting `nextTick()` after the state write lets Vue update the paragraph before `readLabel` samples its text.

`App.vue`

```vue
<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { readLabel } from './read-label';
const count = ref(0);
const label = ref<HTMLElement | null>(null);
const sampled = ref('');
async function increment() {
  count.value++;
  await nextTick();
  sampled.value = readLabel(label.value);
}
</script>

<template>
<button @click="increment">Increment</button><p ref="label">{{ count }}</p><p>DOM sample: {{ sampled }}</p>
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-duplicate-id"></span>

### `vize:croquis/cf/duplicate-id`

The same element id is used in more than one component.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/duplicate-id": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./CheckoutForm.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-duplicate-id-bad"></span>

**Bad**

The reachable shipping and billing components both render `id="postal-code"`, so their labels share an ambiguous document target.

`CheckoutForm.vue`

```vue
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

`ShippingAddress.vue`

```vue
<template>
  <label for="postal-code">Shipping postal code</label>
  <input id="postal-code" />
</template>
```

`BillingAddress.vue`

```vue
<template>
  <label for="postal-code">Billing postal code</label>
  <input id="postal-code" />
</template>
```

<span id="vize-croquis-cf-duplicate-id-good"></span>

**Good**

Each component calls `useId()` and binds its own value to both label and input, preserving the association without a repeated literal ID.

`CheckoutForm.vue`

```vue
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

`ShippingAddress.vue`

```vue
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Shipping postal code</label>
  <input :id="postalCodeId" />
</template>
```

`BillingAddress.vue`

```vue
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Billing postal code</label>
  <input :id="postalCodeId" />
</template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-event-listener-leak"></span>

### `vize:croquis/cf/event-listener-leak`

An event listener is registered and never removed.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useWidth } from './use-width';
const width = useWidth();
</script>

<template>
<p>{{ width }}</p>
</template>

```

<span id="vize-croquis-cf-event-listener-leak-bad"></span>

**Bad**

Mounting adds a window resize listener that captures the component’s width ref, but unmounting never removes it. Repeated mounts can retain unused listeners and state.

`use-width.ts`

```ts
import { onMounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  return width;
}

```

<span id="vize-croquis-cf-event-listener-leak-good"></span>

**Good**

`onUnmounted` removes the exact same `resize` function registered at mount, ending that instance’s external listener lifetime.

`use-width.ts`

```ts
import { onMounted, onUnmounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  onUnmounted(() => { window.removeEventListener('resize', resize); });
  return width;
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-event-modifier"></span>

### `vize:croquis/cf/event-modifier`

An event listener uses a modifier the emit does not support.

Default severity: info  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-event-modifier-bad"></span>

**Bad**

`.stop` assumes a native event's propagation method on the child's custom `save` event, whose payload need not be a DOM Event.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save.stop="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-event-modifier-good"></span>

**Good**

Remove `.stop` from the custom-event listener; handle native propagation at the actual DOM listener when needed.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-hydration-risk"></span>

### `vize:croquis/cf/hydration-risk`

This code groups several reactivity findings, including a prop copied into a ref. It does not imply that every Date.now() expression is detected by the cross-file pass.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/hydration-risk": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-hydration-risk-bad"></span>

**Bad**

The child initializes `ref(props.count)` once, so its local count no longer follows later parent prop changes. This is the current prop-to-ref producer, not a general nondeterministic-SSR example.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :count="0" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
const props = defineProps<{ count: number }>();
const count = ref(props.count);
</script>
<template><p>{{ count }}</p></template>
```

<span id="vize-croquis-cf-hydration-risk-good"></span>

**Good**

`toRef(props, "count")` points to the prop instead of copying its initial value into independent state.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :count="0" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";
const props = defineProps<{ count: number }>();
const count = toRef(props, "count");
</script>
<template><p>{{ count }}</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-inherit-attrs-unused"></span>

### `vize:croquis/cf/inherit-attrs-unused`

`inheritAttrs: false` is set and the component never reads the attributes.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-inherit-attrs-unused-bad"></span>

**Bad**

The child sets `inheritAttrs: false` but never forwards the parent's `class="notice"` attribute.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template><main>Content</main></template>
```

<span id="vize-croquis-cf-inherit-attrs-unused-good"></span>

**Good**

Keep explicit inheritance control and bind `$attrs` to the intended `<main>` target.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template><main v-bind="$attrs">Content</main></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-inject-without-symbol"></span>

### `vize:croquis/cf/inject-without-symbol`

`inject` uses a plain key instead of an `InjectionKey` symbol.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/inject-without-symbol": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";
export interface Theme { color: string; }
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

<span id="vize-croquis-cf-inject-without-symbol-bad"></span>

**Bad**

The consumer injects the untyped string key `"theme"`, which offers no symbol identity shared with the provider.

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

<span id="vize-croquis-cf-inject-without-symbol-good"></span>

**Good**

The consumer and provider import the same `ThemeKey` instead of duplicating string names.

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-injected-async-mutation-race"></span>

### `vize:croquis/cf/injected-async-mutation-race`

An injected value is mutated from an async task that can race.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/injected-async-mutation-race": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./StoreProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export async function loadCount(query: string, options?: { signal?: AbortSignal }): Promise<number> {
  const response = await fetch(`/count?q=${encodeURIComponent(query)}`, options);
  return Number(await response.text());
}
```

`CountSummary.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { StoreKey } from "./keys/store";
const store = inject(StoreKey)!;
</script>
<template><p>{{ store.count }}</p></template>
```

<span id="vize-croquis-cf-injected-async-mutation-race-bad"></span>

**Bad**

`CountLoader.vue` writes an awaited result directly into the injected store shared with `CountSummary.vue`, letting stale work affect both consumers.

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);
</script>

<template>
  <CountLoader />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue
<script setup lang="ts">
import { loadCount } from "./api";
import { inject, ref, watch } from "vue";
import { StoreKey } from "./keys/store";

const store = inject(StoreKey)!;
const query = ref("");

watch(query, async (value) => {
  store.count = await loadCount(value);
});
</script>
```

<span id="vize-croquis-cf-injected-async-mutation-race-good"></span>

**Good**

The loader cancels invalidated work and emits only an active result. The provider owns the store mutation through `applyLoadedCount`.

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);

function applyLoadedCount(count: number) {
  store.count = count;
}
</script>

<template>
  <CountLoader @loaded="applyLoadedCount" />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue
<script setup lang="ts">
import { loadCount } from "./api";
import { ref, watch } from "vue";

const emit = defineEmits<{ loaded: [count: number] }>();
const query = ref("");

watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;

  onCleanup(() => {
    active = false;
    controller.abort();
  });

  const count = await loadCount(value, { signal: controller.signal });
  if (active) emit("loaded", count);
});
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-lifecycle-outside-setup"></span>

### `vize:croquis/cf/lifecycle-outside-setup`

A lifecycle hook is registered outside `setup`.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`install-title.ts`

```ts
import { onMounted } from 'vue';
export function installTitle() {
  onMounted(() => { document.title = 'Mounted application'; });
}

```

<span id="vize-croquis-cf-lifecycle-outside-setup-bad"></span>

**Bad**

The entry calls `installTitle()` before mounting an app, so `onMounted` is registered without an active component setup context.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
import { installTitle } from './install-title';
installTitle();
createApp(App).mount('#app');

```

`App.vue`

```vue
<script setup lang="ts">

</script>

<template>
<p>Application</p>
</template>

```

<span id="vize-croquis-cf-lifecycle-outside-setup-good"></span>

**Good**

Calling the same helper synchronously from App’s setup attaches the lifecycle callback to that instance’s mount.

`App.vue`

```vue
<script setup lang="ts">
import { installTitle } from './install-title';
installTitle();
</script>

<template>
<p>Application</p>
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-lifecycle-without-cleanup"></span>

### `vize:croquis/cf/lifecycle-without-cleanup`

A lifecycle hook starts work and never cleans it up.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-lifecycle-without-cleanup-bad"></span>

**Bad**

Mounting registers a window resize listener, but unmounting never removes the same callback.

`App.vue`

```vue
<script setup lang="ts">
import { onMounted } from "vue";
const resize = () => {};
onMounted(() => { window.addEventListener("resize", resize); });
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-lifecycle-without-cleanup-good"></span>

**Good**

`onUnmounted` removes the listener with the same event name and function identity used by `addEventListener`.

`App.vue`

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
const resize = () => {};
onMounted(() => { window.addEventListener("resize", resize); });
onUnmounted(() => { window.removeEventListener("resize", resize); });
</script>
<template><p>Content</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-missing-required-prop"></span>

### `vize:croquis/cf/missing-required-prop`

A required prop is not passed.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-missing-required-prop-bad"></span>

**Bad**

The parent renders `<Child />` without the child's required `title: string` prop.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-missing-required-prop-good"></span>

**Good**

`title="Hello"` supplies the required prop declared by the resolved child.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-missing-suspense"></span>

### `vize:croquis/cf/missing-suspense`

An async dependency is used outside a Suspense boundary.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`AsyncCard.vue`

```vue
<script setup lang="ts">
const message = await Promise.resolve('Ready');
</script>

<template>
<p>{{ message }}</p>
</template>

```

<span id="vize-croquis-cf-missing-suspense-bad"></span>

**Bad**

`AsyncCard` has top-level await, making its setup asynchronous, but App renders it without a Suspense boundary to coordinate that dependency.

`App.vue`

```vue
<script setup lang="ts">
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<AsyncCard />
</template>

```

<span id="vize-croquis-cf-missing-suspense-good"></span>

**Good**

App wraps the async child in `Suspense` and supplies a loading fallback until the child setup resolves.

`App.vue`

```vue
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-module-scope-reactive"></span>

### `vize:croquis/cf/module-scope-reactive`

Reactive state is created at module scope and shared by every caller.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

Module-scope reactive state is legal for intentional application stores. This example assumes component/request isolation; the published contract currently has no producer.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { createCounter } from './counter';
const { count } = createCounter();
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

<span id="vize-croquis-cf-module-scope-reactive-bad"></span>

**Bad**

The module initializes `count` once, and both Counter instances receive the same ref. Clicking one changes both counters even though this example intends independent instance state.

`counter.ts`

```ts
import { ref } from 'vue';
const count = ref(0);
export function createCounter() { return { count }; }

```

<span id="vize-croquis-cf-module-scope-reactive-good"></span>

**Good**

Creating the ref inside `createCounter` gives each synchronous setup call a separate state object, so each button owns its counter.

`counter.ts`

```ts
import { ref } from 'vue';
export function createCounter() {
  const count = ref(0);
  return { count };
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-multi-root-attrs"></span>

### `vize:croquis/cf/multi-root-attrs`

A multi-root component receives attributes and has nowhere to put them.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-multi-root-attrs-bad"></span>

**Bad**

The child has `<main>` and `<aside>` roots, so Vue has no single root that can automatically receive the parent's class.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vize-croquis-cf-multi-root-attrs-good"></span>

**Good**

Forward `$attrs` explicitly to `<main>` while keeping the second root.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-mutated-after-escape"></span>

### `vize:croquis/cf/mutated-after-escape`

A reactive object is mutated after it has escaped its owner.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

This is an explicit immutable-history ownership policy, not a general prohibition on passing or later mutating reactive objects. No current producer emits this contract.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`archive.ts`

```ts
export interface Profile { name: string }
const records: Readonly<Profile>[] = [];
export function publish(profile: Readonly<Profile>): void { records.push(profile); }
export function latestName(): string { return records.at(-1)?.name ?? ''; }

```

`App.vue`

```vue
<script setup lang="ts">
import { publishProfile } from './profile';
import { latestName } from './archive';
publishProfile();
const archivedName = latestName();
</script>

<template>
<p>Archived name: {{ archivedName }}</p>
</template>

```

<span id="vize-croquis-cf-mutated-after-escape-bad"></span>

**Bad**

The archive retains the same object passed to `publish`. The owner then changes its name, retroactively changing the supposedly historical record to Grace. TypeScript’s Readonly parameter does not copy the object.

`profile.ts`

```ts
import { reactive } from 'vue';
import { publish } from './archive';
export function publishProfile(): void {
  const profile = reactive({ name: 'Ada' });
  publish(profile);
  profile.name = 'Grace';
}

```

<span id="vize-croquis-cf-mutated-after-escape-good"></span>

**Good**

Publishing a plain copy separates the archived Ada record from later edits of the reactive profile. The archive’s snapshot policy is now maintained.

`profile.ts`

```ts
import { reactive } from 'vue';
import { publish } from './archive';
export function publishProfile(): void {
  const profile = reactive({ name: 'Ada' });
  publish({ ...profile });
  profile.name = 'Grace';
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-non-reactive-provide"></span>

### `vize:croquis/cf/non-reactive-provide`

A provided value is not reactive, so descendants will not see updates.

Default severity: context-dependent  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/non-reactive-provide": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-non-reactive-provide-bad"></span>

**Bad**

`ThemeProvider.vue` provides a plain object. Mutating that object's fields does not give the injected consumer a Vue reactive dependency.

`keys/theme.ts`

```ts
export const ThemeKey = Symbol("theme");
```

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = { color: "blue" };
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

<span id="vize-croquis-cf-non-reactive-provide-good"></span>

**Good**

The provider wraps the theme in `ref`; the same injected reference can track later changes.

`keys/theme.ts`

```ts
export const ThemeKey = Symbol("theme");
```

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-non-unique-id"></span>

### `vize:croquis/cf/non-unique-id`

An element id inside a loop is not unique per item.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/non-unique-id": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ResultsList.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-non-unique-id-bad"></span>

**Bad**

Every `v-for` iteration renders the same literal `result-title` ID; the loop's key does not make DOM IDs unique.

`ResultsList.vue`

```vue
<script setup lang="ts">
const results = [{ id: "first", title: "First result" }, { id: "second", title: "Second result" }];
</script>
<template>
  <article v-for="result in results" :key="result.id">
    <h2 id="result-title">{{ result.title }}</h2>
  </article>
</template>
```

<span id="vize-croquis-cf-non-unique-id-good"></span>

**Good**

The heading ID includes the result's stable ID, producing a distinct document identifier for each item.

`ResultsList.vue`

```vue
<script setup lang="ts">
const results = [{ id: "first", title: "First result" }, { id: "second", title: "Second result" }];
</script>
<template>
  <article v-for="result in results" :key="result.id">
    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>
  </article>
</template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-object-identity-comparison"></span>

### `vize:croquis/cf/object-identity-comparison`

A reactive object is compared by identity, which changes across unwraps.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

The example assumes IDs uniquely identify records. Comparing two references to the same reactive proxy remains valid; this contract has no current producer.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`user.ts`

```ts
import { reactive } from 'vue';
export function makeUser() {
  const raw = { id: 7, name: 'Ada' };
  return { raw, proxy: reactive(raw) };
}

```

<span id="vize-croquis-cf-object-identity-comparison-bad"></span>

**Bad**

`proxy === raw` compares wrapper identity, so it is false even though both represent the same user record. The application intended record identity, not object-wrapper identity.

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy === raw;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

<span id="vize-croquis-cf-object-identity-comparison-good"></span>

**Good**

Comparing the stable record `id` answers the intended question without depending on whether the object is raw or proxied.

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy.id === raw.id;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-pinia-getter"></span>

### `vize:croquis/cf/pinia-getter`

A Pinia getter is read without `storeToRefs`, so it will not stay reactive.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

Pinia must be installed, and main.ts installs its plugin before mounting. Reading `store.doubled` directly inside a tracked computation or template is valid; the defect here is taking a plain snapshot. This contract currently has no producer.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
createApp(App).use(createPinia()).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`counter-store.ts`

```ts
import { defineStore } from 'pinia';
export const useCounterStore = defineStore('counter', {
  state: () => ({ count: 0 }),
  getters: { doubled: state => state.count * 2 },
});

```

<span id="vize-croquis-cf-pinia-getter-bad"></span>

**Bad**

`const doubled = store.doubled` copies the getter’s current number during setup. The copied number does not follow later `store.count` updates.

`App.vue`

```vue
<script setup lang="ts">
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const doubled = store.doubled;
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-pinia-getter-good"></span>

**Good**

`storeToRefs(store)` supplies a reactive getter ref that can be destructured and unwrapped by the template while staying connected to the store.

`App.vue`

```vue
<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const { doubled } = storeToRefs(store);
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-prop-type-mismatch"></span>

### `vize:croquis/cf/prop-type-mismatch`

A passed prop value does not match the declared type.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-prop-type-mismatch-bad"></span>

**Bad**

The parent passes the numeric expression `42` to the resolved child's `title: string` prop.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :title="42" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-prop-type-mismatch-good"></span>

**Good**

The literal `title="Hello"` supplies a string matching the child's declaration.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-provide-inject-type"></span>

### `vize:croquis/cf/provide-inject-type`

A provided value and its inject do not have the same type.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/provide-inject-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

This check compares explicit provider/consumer type annotations, not inferred literal value types. Keep the provider's `as string` annotation in this example.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-provide-inject-type-bad"></span>

**Bad**

The provider explicitly annotates `title` as `string`, while the descendant requests `inject<number>` for the same key.

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
const title = inject<number>("title");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-provide-inject-type-good"></span>

**Good**

The consumer's explicit `inject<string>` agrees with the provider annotation. Keep `as string`: this producer compares explicit annotations, not inferred literal types.

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
const title = inject<string>("title");
</script>
<template><p>Content</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-provide-without-symbol"></span>

### `vize:croquis/cf/provide-without-symbol`

`provide` uses a plain key instead of an `InjectionKey` symbol.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/provide-without-symbol": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-provide-without-symbol-bad"></span>

**Bad**

Both components use the string `"theme"`; unrelated features can accidentally reuse that key.

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

<span id="vize-croquis-cf-provide-without-symbol-good"></span>

**Good**

Export one typed `ThemeKey` symbol and import that same value at both provide and inject sites. Creating separate symbols with the same description would not connect them.

`ThemeProvider.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-reactive-export"></span>

### `vize:croquis/cf/reactive-export`

Reactive state is exported from the module.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

Intentional shared application stores may export reactive state. This scenario requires isolated state and does not claim every reactive export is invalid. No current producer emits this contract.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

<span id="vize-croquis-cf-reactive-export-bad"></span>

**Bad**

The module exports one initialized reactive object, so every importer receives the same count. In an SSR module shared between requests, this defeats the example’s per-instance/request state isolation.

`state.ts`

```ts
import { reactive } from 'vue';
export const state = reactive({ count: 0 });

```

`App.vue`

```vue
<script setup lang="ts">
import { state } from './state';
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

<span id="vize-croquis-cf-reactive-export-good"></span>

**Good**

The module exports a factory, and App invokes it inside setup. Each instance obtains a fresh reactive count rather than the exported singleton.

`state.ts`

```ts
import { reactive } from 'vue';
export function createState() { return reactive({ count: 0 }); }

```

`App.vue`

```vue
<script setup lang="ts">
import { createState } from './state';
const state = createState();
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-reactivity-outside-setup"></span>

### `vize:croquis/cf/reactivity-outside-setup`

A reactive API is called outside `setup`.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

Vue permits ref/reactive/computed outside component setup. The risk here is unwanted ownership/sharing under an explicit instance-isolation policy, not API illegality. This contract has no current producer.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { useCounter } from './use-counter';
const { count, doubled } = useCounter();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-reactivity-outside-setup-bad"></span>

**Bad**

Both reactive APIs run while the module loads. The two Counter instances therefore share one ref and computed value despite the intended independent counters.

`use-counter.ts`

```ts
import { computed, ref } from 'vue';
const count = ref(0);
const doubled = computed(() => count.value * 2);
export function useCounter() { return { count, doubled }; }

```

<span id="vize-croquis-cf-reactivity-outside-setup-good"></span>

**Good**

`useCounter` creates the ref and computed synchronously inside each component setup call, giving each widget its own state and tracked derivation.

`use-counter.ts`

```ts
import { computed, ref } from 'vue';
export function useCounter() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-reassignment-breaks-reactivity"></span>

### `vize:croquis/cf/reassignment-breaks-reactivity`

Reassigning a reactive binding replaces it with a plain value.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/reassignment-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-reassignment-breaks-reactivity-bad"></span>

**Bad**

The child creates a prop ref and then overwrites the variable with `props.user`, discarding that ref connection.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
let user = toRef(props, "user");

user = props.user;
</script>
```

<span id="vize-croquis-cf-reassignment-breaks-reactivity-good"></span>

**Good**

Keep the `toRef` in a `const` binding and remove the reassignment that replaces it.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-reference-escapes-scope"></span>

### `vize:croquis/cf/reference-escapes-scope`

A reactive reference escapes the scope that owns its lifetime.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

Refs may legitimately be returned from composables or shared across scopes. This example explicitly requires a snapshot cache; it does not claim that unmount invalidates a ref. No current producer emits this contract.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`saved.ts`

```ts
import type { Ref } from 'vue';
let saved: Ref<number> | number | undefined;
export function remember(value: Ref<number> | number): void { saved = value; }
export function remembered(): Ref<number> | number | undefined { return saved; }

```

<span id="vize-croquis-cf-reference-escapes-scope-bad"></span>

**Bad**

The process-level cache retains the component’s live count ref. It can keep that instance state reachable after unmount and can observe later edits, although this cache is intended to store a snapshot.

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

<span id="vize-croquis-cf-reference-escapes-scope-good"></span>

**Good**

The cache receives the current plain number, so it keeps a snapshot without retaining the component-owned ref.

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count.value);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-setup-context-violation"></span>

### `vize:croquis/cf/setup-context-violation`

Setup context is used in a way Vue does not allow.

Default severity: context-dependent  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-setup-context-violation-bad"></span>

**Bad**

`ref(0)` is created at normal script module scope, outside the per-instance setup context represented by this analyzer scenario.

`App.vue`

```vue
<script lang="ts">
import { ref } from "vue";
const count = ref(0);
export default {};
</script>
<template><p>Count</p></template>
```

<span id="vize-croquis-cf-setup-context-violation-good"></span>

**Good**

Move the binding into script setup, where each component instance owns its count and the template can read it.

`App.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>
<template><p>{{ count }}</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-shallow-deep-access"></span>

### `vize:croquis/cf/shallow-deep-access`

A deep property of a `shallowReactive` or `shallowRef` value is read as if it were tracked.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { makeProfile } from './profile';
const profile = makeProfile();
</script>

<template>
<p>{{ profile.user.name }}</p><button @click="profile.user.name = 'Grace'">Rename</button>
</template>

```

<span id="vize-croquis-cf-shallow-deep-access-bad"></span>

**Bad**

`shallowReactive` tracks the root `user` property but leaves the nested object raw. Changing `profile.user.name` does not notify the template as a tracked deep mutation.

`profile.ts`

```ts
import { shallowReactive } from 'vue';
export function makeProfile() { return shallowReactive({ user: { name: 'Ada' } }); }

```

<span id="vize-croquis-cf-shallow-deep-access-good"></span>

**Good**

Deep `reactive` wraps the nested user object, so the same name assignment can trigger the displayed name’s update.

`profile.ts`

```ts
import { reactive } from 'vue';
export function makeProfile() { return reactive({ user: { name: 'Ada' } }); }

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-spread-breaks-reactivity"></span>

### `vize:croquis/cf/spread-breaks-reactivity`

Spreading a reactive object copies its values and drops the tracking.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/spread-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-spread-breaks-reactivity-bad"></span>

**Bad**

`UserSummary.vue` spreads `props.user` into a new object, taking a snapshot of the incoming reactive data.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ user: { name: string; role: string } }>();
const copiedUser = { ...props.user };
</script>
```

<span id="vize-croquis-cf-spread-breaks-reactivity-good"></span>

**Good**

`toRef(props, "user")` keeps a reference to the incoming prop instead of copying its fields.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string; role: string } }>();
const user = toRef(props, "user");
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-suspense-no-fallback"></span>

### `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` has no fallback content.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

Suspense without a fallback is valid Vue syntax. This is a chosen loading-UI convention, not a compiler error; the published contract has no current producer.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`AsyncCard.vue`

```vue
<script setup lang="ts">
const message = await Promise.resolve('Ready');
</script>

<template>
<p>{{ message }}</p>
</template>

```

<span id="vize-croquis-cf-suspense-no-fallback-bad"></span>

**Bad**

The Suspense boundary has an async child but no fallback content, leaving no loading content for this example’s pending state.

`App.vue`

```vue
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /></Suspense>
</template>

```

<span id="vize-croquis-cf-suspense-no-fallback-good"></span>

**Good**

The `#fallback` slot supplies an explicit loading paragraph until the async child resolves.

`App.vue`

```vue
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-template-ref-timing"></span>

### `vize:croquis/cf/template-ref-timing`

A template ref is read before the component is mounted.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`focus-input.ts`

```ts
export function focusInput(input: HTMLInputElement | null): void { input?.focus(); }

```

<span id="vize-croquis-cf-template-ref-timing-bad"></span>

**Bad**

Setup reads the template ref before mounting, when its value is still null. The optional focus call therefore performs no focus action.

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { focusInput } from './focus-input';
const input = ref<HTMLInputElement | null>(null);
focusInput(input.value);
</script>

<template>
<input ref="input" aria-label="Name" />
</template>

```

<span id="vize-croquis-cf-template-ref-timing-good"></span>

**Good**

`onMounted` defers the read until Vue has assigned the input element to the template ref, allowing the focus helper to act on it.

`App.vue`

```vue
<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { focusInput } from './focus-input';
const input = ref<HTMLInputElement | null>(null);
onMounted(() => { focusInput(input.value); });
</script>

<template>
<input ref="input" aria-label="Name" />
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-toraw-mutation"></span>

### `vize:croquis/cf/toraw-mutation`

`toRaw` is used and the raw object is then mutated.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { makeProfile, rename } from './profile';
const profile = makeProfile();
</script>

<template>
<p>{{ profile.name }}</p><button @click="rename(profile)">Rename</button>
</template>

```

<span id="vize-croquis-cf-toraw-mutation-bad"></span>

**Bad**

`rename` obtains the raw target and writes `raw.name`, bypassing the proxy setter that would notify the displayed reactive name.

`profile.ts`

```ts
import { reactive, toRaw } from 'vue';
export function makeProfile() { return reactive({ name: 'Ada' }); }
export function rename(profile: { name: string }): void {
  const raw = toRaw(profile);
  raw.name = 'Grace';
}

```

<span id="vize-croquis-cf-toraw-mutation-good"></span>

**Good**

Writing `profile.name` through the passed reactive proxy preserves the same rename while notifying its dependents.

`profile.ts`

```ts
import { reactive } from 'vue';
export function makeProfile() { return reactive({ name: 'Ada' }); }
export function rename(profile: { name: string }): void {
  profile.name = 'Grace';
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-uncaught-error"></span>

### `vize:croquis/cf/uncaught-error`

A component can throw and no error boundary catches it.

Default severity: info  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/uncaught-error": "warn" },
    },
  },
});
```

```sh
vp run lint
```

The current producer scans template expressions such as JSON.parse(input). It does not report a throw statement that exists only in the script block.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-uncaught-error-bad"></span>

**Bad**

The child's template calls `JSON.parse` on malformed input, and the reachable parent has no error-capture boundary.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

<span id="vize-croquis-cf-uncaught-error-good"></span>

**Good**

The parent registers `onErrorCaptured` around that child. Returning `false` stops propagation; a production boundary should also present useful recovery UI.

`App.vue`

```vue
<script setup lang="ts">
import { onErrorCaptured } from "vue";
import Child from "./Child.vue";
onErrorCaptured(() => false);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-undeclared-emit"></span>

### `vize:croquis/cf/undeclared-emit`

The component emits an event that is not declared.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-undeclared-emit-bad"></span>

**Bad**

The child calls `emit("save")` but its `defineEmits` contract declares only `cancel`.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ cancel: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-undeclared-emit-good"></span>

**Good**

Declare `save` with its empty argument tuple so the emitted event agrees with the component contract.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-undeclared-prop"></span>

### `vize:croquis/cf/undeclared-prop`

A parent passes a prop the child does not declare.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-undeclared-prop-bad"></span>

**Bad**

The parent passes `typo` even though the resolved child declares only `title`. This analyzer convention is separate from Vue's general fallthrough-attribute behavior.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" :typo="true" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-undeclared-prop-good"></span>

**Good**

Remove the unintended `typo` binding and retain the declared `title` prop.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-undefined-slot"></span>

### `vize:croquis/cf/undefined-slot`

A parent fills a slot the child does not expose.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`Card.vue`

```vue
<script setup lang="ts">
defineSlots<{ header(): unknown }>();
</script>

<template>
<article><header><slot name="header" /></header></article>
</template>

```

<span id="vize-croquis-cf-undefined-slot-bad"></span>

**Bad**

App supplies a `footer` slot, but Card declares and renders only `header`. The supplied Notice content has no matching slot outlet in this child.

`App.vue`

```vue
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #footer>Notice</template></Card>
</template>

```

<span id="vize-croquis-cf-undefined-slot-good"></span>

**Good**

App supplies `header`, matching both the child’s typed slot declaration and its rendered outlet, so Notice appears there.

`App.vue`

```vue
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #header>Notice</template></Card>
</template>

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-unhandled-event"></span>

### `vize:croquis/cf/unhandled-event`

A child emits an event that no parent handles.

Default severity: info  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unhandled-event-bad"></span>

**Bad**

`Child.vue` emits `save`, but its immediate wrapper does not listen for it; component events do not automatically bubble through wrappers.

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unhandled-event-good"></span>

**Good**

`Wrapper.vue` attaches a `save` listener to its direct child. The empty callback demonstrates handling for this rule, not a complete save implementation.

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-unmatched-inject"></span>

### `vize:croquis/cf/unmatched-inject`

`inject` names a key that no ancestor provides.

Default severity: error / warning (with default)  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/unmatched-inject": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unmatched-inject-bad"></span>

**Bad**

`ThemeLabel.vue` injects `ThemeKey`, but its reachable `App.vue` ancestor never provides that key.

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

`App.vue`

```vue
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

<span id="vize-croquis-cf-unmatched-inject-good"></span>

**Good**

`App.vue` provides a reactive theme using the same exported `ThemeKey`, before rendering the descendant that injects it.

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-unmatched-listener"></span>

### `vize:croquis/cf/unmatched-listener`

A parent listens for an event the child does not emit.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unmatched-listener-bad"></span>

**Bad**

The parent listens for `save`, while the resolved child declares only `cancel`.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ cancel: [] }>();
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unmatched-listener-good"></span>

**Good**

The child declares and emits `save`, matching the parent's listener name.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-unregistered-component"></span>

### `vize:croquis/cf/unregistered-component`

A template uses a component that is not registered or imported.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unregistered-component-bad"></span>

**Bad**

A `Child.vue` file exists, but the parent neither imports nor otherwise registers `Child` for its template.

`App.vue`

```vue
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

<span id="vize-croquis-cf-unregistered-component-good"></span>

**Good**

Import `Child` in the parent's script setup so the template resolves the component binding.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-unresolved-import"></span>

### `vize:croquis/cf/unresolved-import`

An import does not resolve to a module.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unresolved-import-bad"></span>

**Bad**

The parent imports `./Missing.vue`, but the project contains `Child.vue` rather than that path.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Missing.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

<span id="vize-croquis-cf-unresolved-import-good"></span>

**Good**

Point the import at the existing `./Child.vue` file, retaining the same template binding.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-unused-attrs"></span>

### `vize:croquis/cf/unused-attrs`

Fallthrough attributes are passed to a multi-root component that does not use them.

Default severity: info  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unused-attrs-bad"></span>

**Bad**

The parent's `tracking-code` is neither consumed as a prop nor forwarded by the multi-root child.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child tracking-code="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vize-croquis-cf-unused-attrs-good"></span>

**Good**

Binding `$attrs` on `<main>` gives that fallthrough attribute an explicit destination.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child tracking-code="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-unused-emit"></span>

### `vize:croquis/cf/unused-emit`

A declared emit is never used.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unused-emit-bad"></span>

**Bad**

The child declares `save` but never calls the emitted-event function with that name.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unused-emit-good"></span>

**Good**

The example calls `emit("save")`, making the declared event used. Real interactions should emit it when the corresponding action occurs.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-unused-provide"></span>

### `vize:croquis/cf/unused-provide`

A provided key is never injected.

Default severity: warning  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/unused-provide": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";
export interface Theme { color: string; }
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

<span id="vize-croquis-cf-unused-provide-bad"></span>

**Bad**

`App.vue` provides `ThemeKey`, but its rendered `Dashboard.vue` subtree has no consumer of that key.

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

`Dashboard.vue`

```vue
<template>
  <h1>Dashboard</h1>
</template>
```

<span id="vize-croquis-cf-unused-provide-good"></span>

**Good**

The dashboard now renders `ThemeLabel.vue`, which injects the ancestor's exact `ThemeKey` identity.

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

`Dashboard.vue`

```vue
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-value-extraction-breaks-reactivity"></span>

### `vize:croquis/cf/value-extraction-breaks-reactivity`

Reading a reactive value out into a local drops later updates.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/value-extraction-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-bad"></span>

**Bad**

Vue 3.5's reactive destructured `item` is read into `itemSnapshot` once; later prop replacement does not update that snapshot.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
const { item } = defineProps<{ item: { name: string } }>();
const itemSnapshot = item;
</script>
```

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-good"></span>

**Good**

Read `item` inside `computed`, so Vue's reactive props-destructuring transform can track each evaluation.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { computed } from "vue";

const { item } = defineProps<{ item: { name: string } }>();
const itemView = computed(() => item);
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-watch-can-be-computed"></span>

### `vize:croquis/cf/watch-can-be-computed`

A watcher only copies a value into state and can be a computed.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

This illustrates the published preference for purely derived state. Watchers remain appropriate for external effects or independently writable state; no current producer emits this contract.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useDouble } from './use-double';
const { count, doubled } = useDouble();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-watch-can-be-computed-bad"></span>

**Bad**

The watcher performs no external effect; it only keeps a second writable ref synchronized with twice `count`. This example has no independent writes to that derived value.

`use-double.ts`

```ts
import { ref, watch } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = ref(0);
  watch(count, next => { doubled.value = next * 2; }, { immediate: true });
  return { count, doubled };
}

```

<span id="vize-croquis-cf-watch-can-be-computed-good"></span>

**Good**

A computed getter expresses the same derivation directly and removes the manual synchronization and extra writable state.

`use-double.ts`

```ts
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-watcheffect-async"></span>

### `vize:croquis/cf/watcheffect-async`

`watchEffect` starts an async task and cannot clean up the previous run.

Default severity: error  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/watcheffect-async": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./SearchPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export interface Result { items: string[]; }
export async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {
  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);
  return response.json();
}
```

<span id="vize-croquis-cf-watcheffect-async-bad"></span>

**Bad**

The async `watchEffect` mixes implicit dependency collection with an awaited request and no invalidation guard.

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watchEffect } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watchEffect(async () => {
  result.value = await load(props.query);
});
</script>
```

<span id="vize-croquis-cf-watcheffect-async-good"></span>

**Good**

An explicit `watch(() => props.query, ...)` declares the source, registers request cleanup, and refuses a stale response after invalidation.

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Cross-file index](cross-file.md)

<span id="vize-croquis-cf-watcher-outside-setup"></span>

### `vize:croquis/cf/watcher-outside-setup`

`watch` or `watchEffect` is called outside `setup`.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

Module-scope watchers are valid when their owner keeps and calls a stop handle or intentionally gives them application lifetime. This example requires component-owned lifetimes; the contract has no current producer.

**Shared project files**

Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Observer from './Observer.vue';
</script>

<template>
<Observer /><Observer />
</template>

```

`Observer.vue`

```vue
<script setup lang="ts">
import { useObserver } from './use-observer';
const { count, observed } = useObserver();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ observed }}</p>
</template>

```

<span id="vize-croquis-cf-watcher-outside-setup-bad"></span>

**Bad**

The watcher is created at module load, outside either Observer’s setup, and both instances share its refs. It is not automatically stopped when a particular Observer unmounts.

`use-observer.ts`

```ts
import { ref, watch } from 'vue';
const count = ref(0);
const observed = ref(0);
watch(count, next => { observed.value = next; });
export function useObserver() { return { count, observed }; }

```

<span id="vize-croquis-cf-watcher-outside-setup-good"></span>

**Good**

Each synchronous setup call creates its own refs and watcher inside `useObserver`. Vue associates that watcher with the calling component’s lifetime.

`use-observer.ts`

```ts
import { ref, watch } from 'vue';
export function useObserver() {
  const count = ref(0);
  const observed = ref(0);
  watch(count, next => { observed.value = next; });
  return { count, observed };
}

```

The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](cross-file.md)
