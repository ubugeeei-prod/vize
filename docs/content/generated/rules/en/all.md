---
title: All lint rules
---

# All lint rules

Use this index to find a rule's purpose, default settings, and owning group. The group pages contain the explanations, configuration, complete Bad/Good examples, and current support limits. Existing rule and example links remain usable here.

With Vite+, import `defineConfig` from `@vizejs/vite-plugin/vite-plus`, configure `vue.linter.rules`, and run `vp run lint` for Vize and Oxlint diagnostics.

Severity is the implementation default; override it with `off`, `warn`, or `error`. See [Rule Options](./options.md) for rule-option support and [Cross-file rules](./cross-file.md) for project-graph findings.

`_none_` means explicit enablement or host configuration is required. `general-recommended` is displayed as `happy-path`.

See the [ESLint migration map](./migration.md) for rule IDs, differences, and unsupported mappings.

## Single-file rules (255)

| Rule | Examples | Severity | Presets | Fixable | Options | Implementation | Description | Category |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [Bad](#petite-vue-no-unsupported-directive-bad) · [Good](#petite-vue-no-unsupported-directive-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) | Disallow directives that petite-vue does not support | Essential |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [Bad](#petite-vue-valid-v-effect-bad) · [Good](#petite-vue-valid-v-effect-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) | Require v-effect to have a non-empty expression | Essential |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [Bad](#petite-vue-valid-v-scope-bad) · [Good](#petite-vue-valid-v-scope-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) | Require v-scope to bind an object literal | Essential |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [Bad](#vue-multi-word-component-names-bad) · [Good](#vue-multi-word-component-names-good) | `error` | `essential`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) | Require component names to be multi-word | Essential |
| [`vue/no-child-content`](#vue-no-child-content) | [Bad](#vue-no-child-content-bad) · [Good](#vue-no-child-content-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) | Disallow child content when using v-html or v-text | Essential |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [Bad](#vue-no-deprecated-filter-bad) · [Good](#vue-no-deprecated-filter-good) | `error` | `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L56) | Disallow deprecated Vue 2 filter syntax using the pipe operator | Essential |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [Bad](#vue-no-deprecated-functional-template-bad) · [Good](#vue-no-deprecated-functional-template-good) | `error` | `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) | Disallow the `functional` attribute on the SFC `&lt;template&gt;` | Essential |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [Bad](#vue-no-deprecated-html-element-is-bad) · [Good](#vue-no-deprecated-html-element-is-good) | `error` | `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) | Disallow the `is` attribute on native HTML elements | Essential |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [Bad](#vue-no-deprecated-inline-template-bad) · [Good](#vue-no-deprecated-inline-template-good) | `error` | `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) | Disallow the deprecated `inline-template` attribute | Essential |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [Bad](#vue-no-deprecated-router-link-tag-prop-bad) · [Good](#vue-no-deprecated-router-link-tag-prop-good) | `error` | `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) | Disallow the `tag` prop on &lt;router-link&gt; | Essential |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [Bad](#vue-no-deprecated-scope-attribute-bad) · [Good](#vue-no-deprecated-scope-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) | Disallow the deprecated `scope` attribute on &lt;template&gt; | Essential |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [Bad](#vue-no-deprecated-slot-attribute-bad) · [Good](#vue-no-deprecated-slot-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) | Disallow the deprecated `slot` attribute | Essential |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [Bad](#vue-no-deprecated-slot-scope-attribute-bad) · [Good](#vue-no-deprecated-slot-scope-attribute-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) | Disallow the deprecated `slot-scope` attribute | Essential |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [Bad](#vue-no-deprecated-v-bind-sync-bad) · [Good](#vue-no-deprecated-v-bind-sync-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) | Disallow the deprecated `.sync` modifier on `v-bind` | Essential |
| [`vue/no-deprecated-v-is`](#vue-no-deprecated-v-is) | [Bad](#vue-no-deprecated-v-is-bad) · [Good](#vue-no-deprecated-v-is-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_is.rs#L27) | Disallow the deprecated `v-is` directive | Essential |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [Bad](#vue-no-deprecated-v-on-native-modifier-bad) · [Good](#vue-no-deprecated-v-on-native-modifier-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) | Disallow the deprecated `.native` modifier on `v-on` | Essential |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [Bad](#vue-no-deprecated-v-on-number-modifiers-bad) · [Good](#vue-no-deprecated-v-on-number-modifiers-good) | `error` | `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) | Disallow deprecated numeric `keyCode` modifiers on `v-on` | Essential |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [Bad](#vue-no-dupe-v-else-if-bad) · [Good](#vue-no-dupe-v-else-if-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) | Disallow duplicate conditions in `v-if` / `v-else-if` chains | Essential |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [Bad](#vue-no-duplicate-attributes-bad) · [Good](#vue-no-duplicate-attributes-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) | Disallow duplicate attributes on the same element | Essential |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [Bad](#vue-no-multiple-template-root-bad) · [Good](#vue-no-multiple-template-root-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) | Disallow multiple root nodes in a template | Essential |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [Bad](#vue-no-mutating-props-bad) · [Good](#vue-no-mutating-props-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L36) | Disallow mutating component props | Essential |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [Bad](#vue-no-reserved-component-names-bad) · [Good](#vue-no-reserved-component-names-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) | Disallow the use of reserved names as component names | Essential |
| [`vue/no-template-key`](#vue-no-template-key) | [Bad](#vue-no-template-key-bad) · [Good](#vue-no-template-key-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) | Disallow `key` attribute on `&lt;template&gt;` | Essential |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [Bad](#vue-no-textarea-mustache-bad) · [Good](#vue-no-textarea-mustache-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) | Disallow mustache interpolation in `&lt;textarea&gt;` | Essential |
| [`vue/no-unused-components`](#vue-no-unused-components) | [Bad](#vue-no-unused-components-bad) · [Good](#vue-no-unused-components-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L47) | Disallow registering components that are not used inside templates | Essential |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [Bad](#vue-no-unused-vars-bad) · [Good](#vue-no-unused-vars-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) | Disallow unused variable definitions in v-for and v-slot directives | Essential |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [Bad](#vue-no-use-v-if-with-v-for-bad) · [Good](#vue-no-use-v-if-with-v-for-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) | Disallow using `v-if` on the same element as `v-for` | Essential |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [Bad](#vue-no-useless-template-attributes-bad) · [Good](#vue-no-useless-template-attributes-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) | Disallow useless attributes on `&lt;template&gt;` elements | Essential |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [Bad](#vue-no-v-for-template-key-on-child-bad) · [Good](#vue-no-v-for-template-key-on-child-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) | Disallow `key` on the child of a `&lt;template v-for&gt;` | Essential |
| [`vue/no-v-html`](#vue-no-v-html) | [Bad](#vue-no-v-html-bad) · [Good](#vue-no-v-html-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) | Warn against v-html to prevent XSS vulnerabilities | Essential |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [Bad](#vue-no-v-text-v-html-on-component-bad) · [Good](#vue-no-v-text-v-html-on-component-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) | Disallow v-text / v-html on component elements | Essential |
| [`vue/permitted-contents`](#vue-permitted-contents) | [Bad](#vue-permitted-contents-bad) · [Good](#vue-permitted-contents-good) | `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) | Enforce HTML content model rules | Essential |
| [`vue/require-component-is`](#vue-require-component-is) | [Bad](#vue-require-component-is-bad) · [Good](#vue-require-component-is-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) | Require `v-bind:is` on `&lt;component&gt;` elements | Essential |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [Bad](#vue-require-toggle-inside-transition-bad) · [Good](#vue-require-toggle-inside-transition-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L49) | Require a toggle on the element wrapped by `&lt;transition&gt;` | Essential |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [Bad](#vue-require-v-for-key-bad) · [Good](#vue-require-v-for-key-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L46) | Require `v-bind:key` with `v-for` directives | Essential |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [Bad](#vue-use-v-on-exact-bad) · [Good](#vue-use-v-on-exact-good) | `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L31) | Enforce `.exact` modifier on `v-on` when there are modifier-based handlers | Essential |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [Bad](#vue-valid-attribute-name-bad) · [Good](#vue-valid-attribute-name-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) | Require valid attribute names | Essential |
| [`vue/valid-template-root`](#vue-valid-template-root) | [Bad](#vue-valid-template-root-bad) · [Good](#vue-valid-template-root-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) | Enforce a valid `&lt;template&gt;` root for Vue 3 fragment semantics | Essential |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [Bad](#vue-valid-v-bind-bad) · [Good](#vue-valid-v-bind-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) | Enforce valid `v-bind` directives | Essential |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [Bad](#vue-valid-v-cloak-bad) · [Good](#vue-valid-v-cloak-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) | Enforce valid `v-cloak` directives | Essential |
| [`vue/valid-v-else`](#vue-valid-v-else) | [Bad](#vue-valid-v-else-bad) · [Good](#vue-valid-v-else-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L31) | Enforce valid `v-else` directives | Essential |
| [`vue/valid-v-else-if`](#vue-valid-v-else-if) | [Bad](#vue-valid-v-else-if-bad) · [Good](#vue-valid-v-else-if-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else_if.rs#L34) | Enforce valid `v-else-if` directives | Essential |
| [`vue/valid-v-for`](#vue-valid-v-for) | [Bad](#vue-valid-v-for-bad) · [Good](#vue-valid-v-for-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L39) | Enforce valid `v-for` directives | Essential |
| [`vue/valid-v-html`](#vue-valid-v-html) | [Bad](#vue-valid-v-html-bad) · [Good](#vue-valid-v-html-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) | Enforce valid `v-html` directives | Essential |
| [`vue/valid-v-if`](#vue-valid-v-if) | [Bad](#vue-valid-v-if-bad) · [Good](#vue-valid-v-if-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) | Enforce valid `v-if` directives | Essential |
| [`vue/valid-v-is`](#vue-valid-v-is) | [Bad](#vue-valid-v-is-bad) · [Good](#vue-valid-v-is-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_is.rs#L30) | Enforce valid `v-is` directives | Essential |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [Bad](#vue-valid-v-memo-bad) · [Good](#vue-valid-v-memo-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) | Enforce valid `v-memo` directives | Essential |
| [`vue/valid-v-model`](#vue-valid-v-model) | [Bad](#vue-valid-v-model-bad) · [Good](#vue-valid-v-model-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) | Enforce valid `v-model` directives | Essential |
| [`vue/valid-v-on`](#vue-valid-v-on) | [Bad](#vue-valid-v-on-bad) · [Good](#vue-valid-v-on-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) | Enforce valid `v-on` directives | Essential |
| [`vue/valid-v-once`](#vue-valid-v-once) | [Bad](#vue-valid-v-once-bad) · [Good](#vue-valid-v-once-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) | Enforce valid `v-once` directives | Essential |
| [`vue/valid-v-pre`](#vue-valid-v-pre) | [Bad](#vue-valid-v-pre-bad) · [Good](#vue-valid-v-pre-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_pre.rs#L30) | Enforce valid `v-pre` directives | Essential |
| [`vue/valid-v-show`](#vue-valid-v-show) | [Bad](#vue-valid-v-show-bad) · [Good](#vue-valid-v-show-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) | Enforce valid `v-show` directives | Essential |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [Bad](#vue-valid-v-slot-bad) · [Good](#vue-valid-v-slot-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) | Enforce valid `v-slot` directives | Essential |
| [`vue/valid-v-text`](#vue-valid-v-text) | [Bad](#vue-valid-v-text-bad) · [Good](#vue-valid-v-text-good) | `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) | Enforce valid `v-text` directives | Essential |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [Bad](#vue-attribute-hyphenation-bad) · [Good](#vue-attribute-hyphenation-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) | Enforce attribute naming style on custom components | Strongly Recommended |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [Bad](#vue-component-definition-name-casing-bad) · [Good](#vue-component-definition-name-casing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) | Enforce PascalCase or kebab-case for component definition names | Strongly Recommended |
| [`vue/html-quotes`](#vue-html-quotes) | [Bad](#vue-html-quotes-bad) · [Good](#vue-html-quotes-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) | Enforce quotes style of HTML attributes | Strongly Recommended |
| [`vue/html-self-closing`](#vue-html-self-closing) | [Bad](#vue-html-self-closing-bad) · [Good](#vue-html-self-closing-good) | `warning` | `nuxt`, `opinionated` | Yes | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) | Enforce self-closing style | Strongly Recommended |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [Bad](#vue-mustache-interpolation-spacing-bad) · [Good](#vue-mustache-interpolation-spacing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) | Enforce consistent spacing inside mustache interpolations | Strongly Recommended |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [Bad](#vue-no-multi-spaces-bad) · [Good](#vue-no-multi-spaces-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) | Disallow multiple consecutive spaces | Strongly Recommended |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [Bad](#vue-no-template-shadow-bad) · [Good](#vue-no-template-shadow-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) | Disallow variable names that shadow variables in outer scope | Strongly Recommended |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [Bad](#vue-no-unused-properties-bad) · [Good](#vue-no-unused-properties-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) | Disallow unused properties defined in defineProps | Strongly Recommended |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [Bad](#vue-prop-name-casing-bad) · [Good](#vue-prop-name-casing-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) | Enforce a casing for declared prop names | Strongly Recommended |
| [`vue/v-bind-style`](#vue-v-bind-style) | [Bad](#vue-v-bind-style-bad) · [Good](#vue-v-bind-style-good) | `warning` | `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L28) | Enforce `v-bind` directive style | Strongly Recommended |
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
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [Bad](#vue-no-non-component-keep-alive-child-bad) · [Good](#vue-no-non-component-keep-alive-child-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L15) | Disallow plain element wrappers directly below `&lt;KeepAlive&gt;` | Recommended |
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
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [Bad](#vue-no-use-v-else-with-v-for-bad) · [Good](#vue-no-use-v-else-with-v-for-good) | `warning` | `ecosystem`, `happy-path`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) | Disallow using `v-else-if` or `v-else` on the same element as `v-for` | Recommended |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [Bad](#vue-no-useless-mustaches-bad) · [Good](#vue-no-useless-mustaches-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) | Disallow a mustache interpolation whose expression is a constant string literal | Recommended |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [Bad](#vue-no-useless-v-bind-bad) · [Good](#vue-no-useless-v-bind-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) | Disallow a v-bind whose value is a plain string literal | Recommended |
| [`vue/no-v-text`](#vue-no-v-text) | [Bad](#vue-no-v-text-bad) · [Good](#vue-no-v-text-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) | Disallow the v-text directive; prefer mustache interpolation | Recommended |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [Bad](#vue-prefer-props-shorthand-bad) · [Good](#vue-prefer-props-shorthand-good) | `warning` | `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) | Recommend shorthand syntax for props (Vue 3.4+) | Recommended |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [Bad](#vue-prefer-true-attribute-shorthand-bad) · [Good](#vue-prefer-true-attribute-shorthand-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) | Prefer the shorthand for a boolean attribute bound to `true` | Recommended |
| [`vue/require-component-registration`](#vue-require-component-registration) | [Bad](#vue-require-component-registration-bad) · [Good](#vue-require-component-registration-good) | `warning` | `opinionated` | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L57) | Require explicit import or registration for components | Recommended |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [Bad](#vue-require-scoped-style-bad) · [Good](#vue-require-scoped-style-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) | Require scoped attribute on style tags | Recommended |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [Bad](#vue-scoped-event-names-bad) · [Good](#vue-scoped-event-names-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) | Recommend scoped event names using context:event format | Recommended |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [Bad](#vue-sfc-element-order-bad) · [Good](#vue-sfc-element-order-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) | Enforce consistent order of SFC top-level elements | Recommended |
| [`vue/single-style-block`](#vue-single-style-block) | [Bad](#vue-single-style-block-bad) · [Good](#vue-single-style-block-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) | Recommend having a single style block | Recommended |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [Bad](#vue-slot-name-casing-bad) · [Good](#vue-slot-name-casing-good) | `warning` | `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) | Enforce kebab-case for named slots used via v-slot | Recommended |
| [`vue/this-in-template`](#vue-this-in-template) | [Bad](#vue-this-in-template-bad) · [Good](#vue-this-in-template-good) | `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L37) | Disallow `this.` in template expressions | Recommended |
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
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [Bad](#script-no-deep-destructure-in-props-bad) · [Good](#script-no-deep-destructure-in-props-good) | `warning` | `opinionated`, `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L39) | Disallow deeply nested destructuring in defineProps | Script |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [Bad](#script-no-deprecated-data-object-declaration-bad) · [Good](#script-no-deprecated-data-object-declaration-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) | Disallow an object literal as the component data option (Vue 3 requires a function) | Script |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [Bad](#script-no-deprecated-destroyed-lifecycle-bad) · [Good](#script-no-deprecated-destroyed-lifecycle-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) | Disallow deprecated destroyed and beforeDestroy lifecycle hooks | Script |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [Bad](#script-no-deprecated-dollar-listeners-api-bad) · [Good](#script-no-deprecated-dollar-listeners-api-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L54) | Disallow the $listeners instance property removed in Vue 3 (merged into $attrs) | Script |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [Bad](#script-no-deprecated-dollar-scopedslots-api-bad) · [Good](#script-no-deprecated-dollar-scopedslots-api-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L45) | Disallow the $scopedSlots instance property removed in Vue 3 (use $slots) | Script |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [Bad](#script-no-deprecated-events-api-bad) · [Good](#script-no-deprecated-events-api-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L51) | Disallow the removed Vue 2 events API ($on / $off / $once) | Script |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [Bad](#script-no-deprecated-props-default-this-bad) · [Good](#script-no-deprecated-props-default-this-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) | Disallow `this` inside a prop default/validator function (removed in Vue 3) | Script |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [Bad](#script-no-dupe-keys-bad) · [Good](#script-no-dupe-keys-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L55) | Disallow duplicate keys across Options API props/data/computed/methods/setup/inject | Script |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [Bad](#script-no-duplicate-attr-inheritance-bad) · [Good](#script-no-duplicate-attr-inheritance-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L35) | Flag a component that applies its fallthrough attributes twice | Script |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [Bad](#script-no-export-in-script-setup-bad) · [Good](#script-no-export-in-script-setup-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) | Disallow export statements inside &lt;script setup&gt; | Script |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [Bad](#script-no-import-compiler-macros-bad) · [Good](#script-no-import-compiler-macros-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) | Disallow importing Vue compiler macros that are auto-imported | Script |
| [`script/no-internal-imports`](#script-no-internal-imports) | [Bad](#script-no-internal-imports-bad) · [Good](#script-no-internal-imports-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) | Disallow importing from Vue internal modules | Script |
| [`script/no-multiple-slot-args`](#script-no-multiple-slot-args) | [Bad](#script-no-multiple-slot-args-bad) · [Good](#script-no-multiple-slot-args-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) | Disallow passing more than one argument to a scoped-slot function call | Script |
| [`script/no-potential-component-option-typo`](#script-no-potential-component-option-typo) | [Bad](#script-no-potential-component-option-typo-bad) · [Good](#script-no-potential-component-option-typo-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) | Flag likely typos in Options API component option names | Script |
| [`script/no-reactive-destructure`](#script-no-reactive-destructure) | [Bad](#script-no-reactive-destructure-bad) · [Good](#script-no-reactive-destructure-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) | Disallow destructuring reactive objects which loses reactivity | Script |
| [`script/no-ref-as-operand`](#script-no-ref-as-operand) | [Bad](#script-no-ref-as-operand-bad) · [Good](#script-no-ref-as-operand-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) | Require ref-bound variables to be accessed via `.value` when used as an operand | Script |
| [`script/no-required-prop-with-default`](#script-no-required-prop-with-default) | [Bad](#script-no-required-prop-with-default-bad) · [Good](#script-no-required-prop-with-default-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L34) | Disallow a prop that is both required: true and has a default | Script |
| [`script/no-reserved-identifiers`](#script-no-reserved-identifiers) | [Bad](#script-no-reserved-identifiers-bad) · [Good](#script-no-reserved-identifiers-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) | Disallow using Vue compiler reserved identifiers | Script |
| [`script/no-reserved-keys`](#script-no-reserved-keys) | [Bad](#script-no-reserved-keys-bad) · [Good](#script-no-reserved-keys-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L30) | Disallow Vue-reserved names as Options API props/data/computed/methods/setup keys | Script |
| [`script/no-reserved-props`](#script-no-reserved-props) | [Bad](#script-no-reserved-props-bad) · [Good](#script-no-reserved-props-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) | Disallow reserved names in a component's props declaration | Script |
| [`script/no-restricted-globals`](#script-no-restricted-globals) | [Bad](#script-no-restricted-globals-bad) · [Good](#script-no-restricted-globals-good) | `error` | _none_ | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) | Disallow references to runtime-environment globals that must go through a typed wrapper | Script |
| [`script/no-restricted-members`](#script-no-restricted-members) | [Bad](#script-no-restricted-members-bad) · [Good](#script-no-restricted-members-good) | `error` | _none_ | No | [`ruleOptions`](./options.md) | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) | Disallow project-configured object.property member accesses | Script |
| [`script/no-side-effects-in-computed-properties`](#script-no-side-effects-in-computed-properties) | [Bad](#script-no-side-effects-in-computed-properties-bad) · [Good](#script-no-side-effects-in-computed-properties-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L77) | Disallow side effects in Options API computed getters | Script |
| [`script/no-top-level-ref-in-script`](#script-no-top-level-ref-in-script) | [Bad](#script-no-top-level-ref-in-script-bad) · [Good](#script-no-top-level-ref-in-script-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) | Disallow top-level ref/reactive to prevent Cross-Request State Pollution | Script |
| [`script/no-unstable-nested-components`](#script-no-unstable-nested-components) | [Bad](#script-no-unstable-nested-components-bad) · [Good](#script-no-unstable-nested-components-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) | Disallow component definitions inside setup or render functions | Script |
| [`script/no-unused-emit-declarations`](#script-no-unused-emit-declarations) | [Bad](#script-no-unused-emit-declarations-bad) · [Good](#script-no-unused-emit-declarations-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) | Flag declared events that are never emitted | Script |
| [`script/no-use-computed-property-like-method`](#script-no-use-computed-property-like-method) | [Bad](#script-no-use-computed-property-like-method-bad) · [Good](#script-no-use-computed-property-like-method-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) | Disallow calling an Options API computed property like a method | Script |
| [`script/no-with-defaults`](#script-no-with-defaults) | [Bad](#script-no-with-defaults-bad) · [Good](#script-no-with-defaults-good) | `warning` | `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) | Discourage withDefaults in favor of destructuring defaults (Vue 3.5+) | Script |
| [`script/prefer-computed`](#script-prefer-computed) | [Bad](#script-prefer-computed-bad) · [Good](#script-prefer-computed-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) | Prefer computed() for derived reactive state | Script |
| [`script/prefer-define-options`](#script-prefer-define-options) | [Bad](#script-prefer-define-options-bad) · [Good](#script-prefer-define-options-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) | Prefer defineOptions() over a plain &lt;script&gt; that only sets name/inheritAttrs | Script |
| [`script/prefer-import-from-vue`](#script-prefer-import-from-vue) | [Bad](#script-prefer-import-from-vue-bad) · [Good](#script-prefer-import-from-vue-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L36) | Prefer importing from 'vue' instead of internal packages | Script |
| [`script/prefer-ref-over-reactive`](#script-prefer-ref-over-reactive) | [Bad](#script-prefer-ref-over-reactive-bad) · [Good](#script-prefer-ref-over-reactive-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) | Recommend using ref() over reactive() for state management | Script |
| [`script/prefer-use-attrs`](#script-prefer-use-attrs) | [Bad](#script-prefer-use-attrs-bad) · [Good](#script-prefer-use-attrs-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) | Recommend using useAttrs() over context.attrs | Script |
| [`script/prefer-use-id`](#script-prefer-use-id) | [Bad](#script-prefer-use-id-bad) · [Good](#script-prefer-use-id-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) | Recommend using useId() for generating unique IDs (Vue 3.5+) | Script |
| [`script/prefer-use-slots`](#script-prefer-use-slots) | [Bad](#script-prefer-use-slots-bad) · [Good](#script-prefer-use-slots-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) | Recommend using useSlots() over context.slots | Script |
| [`script/prefer-use-template-ref`](#script-prefer-use-template-ref) | [Bad](#script-prefer-use-template-ref-bad) · [Good](#script-prefer-use-template-ref-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) | Recommend useTemplateRef over ref for template references (Vue 3.5+) | Script |
| [`script/require-default-prop`](#script-require-default-prop) | [Bad](#script-require-default-prop-bad) · [Good](#script-require-default-prop-good) | `error` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) | Require a default value for every optional, non-Boolean prop | Script |
| [`script/require-explicit-emits`](#script-require-explicit-emits) | [Bad](#script-require-explicit-emits-bad) · [Good](#script-require-explicit-emits-good) | `warning` | `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) | Require emitted events to be declared in defineEmits or the emits option | Script |
| [`script/require-explicit-slots`](#script-require-explicit-slots) | [Bad](#script-require-explicit-slots-bad) · [Good](#script-require-explicit-slots-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) | Require slots consumed via useSlots() to be explicitly typed with defineSlots&lt;...&gt;() | Script |
| [`script/require-function-return-type`](#script-require-function-return-type) | [Bad](#script-require-function-return-type-bad) · [Good](#script-require-function-return-type-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) | Require return type annotations on functions | Script |
| [`script/require-prop-type-constructor`](#script-require-prop-type-constructor) | [Bad](#script-require-prop-type-constructor-bad) · [Good](#script-require-prop-type-constructor-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L65) | Require prop `type` values to be constructors rather than string literals | Script |
| [`script/require-prop-types`](#script-require-prop-types) | [Bad](#script-require-prop-types-bad) · [Good](#script-require-prop-types-good) | `error` | `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) | Require every prop to declare a type | Script |
| [`script/require-symbol-provide`](#script-require-symbol-provide) | [Bad](#script-require-symbol-provide-bad) · [Good](#script-require-symbol-provide-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) | Recommend using Symbol as injection key for provide/inject | Script |
| [`script/require-typed-object-prop`](#script-require-typed-object-prop) | [Bad](#script-require-typed-object-prop-bad) · [Good](#script-require-typed-object-prop-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) | Require an explicit type on a prop whose runtime type is `Object` or `Array` | Script |
| [`script/require-typed-ref`](#script-require-typed-ref) | [Bad](#script-require-typed-ref-bad) · [Good](#script-require-typed-ref-good) | `warning` | _none_ | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) | Require an explicit type argument on a ref() initialized with no value, null, or undefined | Script |
| [`script/require-valid-default-prop`](#script-require-valid-default-prop) | [Bad](#script-require-valid-default-prop-bad) · [Good](#script-require-valid-default-prop-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) | Require a prop's default value to be valid for its declared type | Script |
| [`script/return-in-computed-property`](#script-return-in-computed-property) | [Bad](#script-return-in-computed-property-bad) · [Good](#script-return-in-computed-property-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) | Require a return value in every computed getter | Script |
| [`script/return-in-emits-validator`](#script-return-in-emits-validator) | [Bad](#script-return-in-emits-validator-bad) · [Good](#script-return-in-emits-validator-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L61) | Require a return value in every Vue emits validator | Script |
| [`script/valid-define-emits`](#script-valid-define-emits) | [Bad](#script-valid-define-emits-bad) · [Good](#script-valid-define-emits-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) | Enforce valid defineEmits() usage (no type+runtime args, no local references, single call) | Script |
| [`script/valid-define-options`](#script-valid-define-options) | [Bad](#script-valid-define-options-bad) · [Good](#script-valid-define-options-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) | Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots) | Script |
| [`script/valid-define-props`](#script-valid-define-props) | [Bad](#script-valid-define-props-bad) · [Good](#script-valid-define-props-good) | `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) | Enforce valid defineProps() usage (single call, not both type and runtime args, no local references) | Script |
| [`script/valid-next-tick`](#script-valid-next-tick) | [Bad](#script-valid-next-tick-bad) · [Good](#script-valid-next-tick-good) | `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) | Require the result of a nextTick() call to be awaited, chained, or given a callback | Script |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [Bad](#nuxt-no-nuxt-config-test-key-bad) · [Good](#nuxt-no-nuxt-config-test-key-good) | `error` | `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) | Disallow setting `test` key in Nuxt config | Nuxt |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [Bad](#nuxt-no-page-meta-runtime-values-bad) · [Good](#nuxt-no-page-meta-runtime-values-good) | `error` | `nuxt` | No | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) | Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup | Nuxt |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [Bad](#nuxt-nuxt-config-keys-order-bad) · [Good](#nuxt-nuxt-config-keys-order-good) | `error` | `nuxt` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) | Prefer recommended order of Nuxt config properties | Nuxt |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [Bad](#nuxt-prefer-import-meta-bad) · [Good](#nuxt-prefer-import-meta-good) | `error` | `nuxt` | Yes | No | [source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) | Prefer using `import.meta.*` over `process.*` | Nuxt |

## Project rules and analyzer contracts (66)

These project entries supplement the 255 single-file catalog entries above. Each example includes its component/project context or an explicit tracked-graph scenario. CLI, experimental library producers, and contracts without a producer have different support boundaries; see the [cross-file overview](./cross-file.md).

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

### `petite-vue/no-unsupported-directive`

Disallow directives that petite-vue does not support

<span id="petite-vue-no-unsupported-directive-bad"></span>

[Bad](./petite-vue.md#petite-vue-no-unsupported-directive-bad)

<span id="petite-vue-no-unsupported-directive-good"></span>

[Good](./petite-vue.md#petite-vue-no-unsupported-directive-good)

[Group examples](./petite-vue.md#petite-vue-no-unsupported-directive) · [Rule reference](./reference/petite-vue-no-unsupported-directive.md)

### `petite-vue/valid-v-effect`

Require v-effect to have a non-empty expression

<span id="petite-vue-valid-v-effect-bad"></span>

[Bad](./petite-vue.md#petite-vue-valid-v-effect-bad)

<span id="petite-vue-valid-v-effect-good"></span>

[Good](./petite-vue.md#petite-vue-valid-v-effect-good)

[Group examples](./petite-vue.md#petite-vue-valid-v-effect) · [Rule reference](./reference/petite-vue-valid-v-effect.md)

### `petite-vue/valid-v-scope`

Require v-scope to bind an object literal

<span id="petite-vue-valid-v-scope-bad"></span>

[Bad](./petite-vue.md#petite-vue-valid-v-scope-bad)

<span id="petite-vue-valid-v-scope-good"></span>

[Good](./petite-vue.md#petite-vue-valid-v-scope-good)

[Group examples](./petite-vue.md#petite-vue-valid-v-scope) · [Rule reference](./reference/petite-vue-valid-v-scope.md)

### `vue/multi-word-component-names`

Require component names to be multi-word

<span id="vue-multi-word-component-names-bad"></span>

[Bad](./vue.md#vue-multi-word-component-names-bad)

<span id="vue-multi-word-component-names-good"></span>

[Good](./vue.md#vue-multi-word-component-names-good)

[Group examples](./vue.md#vue-multi-word-component-names) · [Rule reference](./reference/vue-multi-word-component-names.md)

### `vue/no-child-content`

Disallow child content when using v-html or v-text

<span id="vue-no-child-content-bad"></span>

[Bad](./vue.md#vue-no-child-content-bad)

<span id="vue-no-child-content-good"></span>

[Good](./vue.md#vue-no-child-content-good)

[Group examples](./vue.md#vue-no-child-content) · [Rule reference](./reference/vue-no-child-content.md)

### `vue/no-deprecated-filter`

Disallow deprecated Vue 2 filter syntax using the pipe operator

<span id="vue-no-deprecated-filter-bad"></span>

[Bad](./vue.md#vue-no-deprecated-filter-bad)

<span id="vue-no-deprecated-filter-good"></span>

[Good](./vue.md#vue-no-deprecated-filter-good)

[Group examples](./vue.md#vue-no-deprecated-filter) · [Rule reference](./reference/vue-no-deprecated-filter.md)

### `vue/no-deprecated-functional-template`

Disallow the `functional` attribute on the SFC `<template>`

<span id="vue-no-deprecated-functional-template-bad"></span>

[Bad](./vue.md#vue-no-deprecated-functional-template-bad)

<span id="vue-no-deprecated-functional-template-good"></span>

[Good](./vue.md#vue-no-deprecated-functional-template-good)

[Group examples](./vue.md#vue-no-deprecated-functional-template) · [Rule reference](./reference/vue-no-deprecated-functional-template.md)

### `vue/no-deprecated-html-element-is`

Disallow the `is` attribute on native HTML elements

<span id="vue-no-deprecated-html-element-is-bad"></span>

[Bad](./vue.md#vue-no-deprecated-html-element-is-bad)

<span id="vue-no-deprecated-html-element-is-good"></span>

[Good](./vue.md#vue-no-deprecated-html-element-is-good)

[Group examples](./vue.md#vue-no-deprecated-html-element-is) · [Rule reference](./reference/vue-no-deprecated-html-element-is.md)

### `vue/no-deprecated-inline-template`

Disallow the deprecated `inline-template` attribute

<span id="vue-no-deprecated-inline-template-bad"></span>

[Bad](./vue.md#vue-no-deprecated-inline-template-bad)

<span id="vue-no-deprecated-inline-template-good"></span>

[Good](./vue.md#vue-no-deprecated-inline-template-good)

[Group examples](./vue.md#vue-no-deprecated-inline-template) · [Rule reference](./reference/vue-no-deprecated-inline-template.md)

### `vue/no-deprecated-router-link-tag-prop`

Disallow the `tag` prop on &lt;router-link&gt;

<span id="vue-no-deprecated-router-link-tag-prop-bad"></span>

[Bad](./vue.md#vue-no-deprecated-router-link-tag-prop-bad)

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

[Good](./vue.md#vue-no-deprecated-router-link-tag-prop-good)

[Group examples](./vue.md#vue-no-deprecated-router-link-tag-prop) · [Rule reference](./reference/vue-no-deprecated-router-link-tag-prop.md)

### `vue/no-deprecated-scope-attribute`

Disallow the deprecated `scope` attribute on &lt;template&gt;

<span id="vue-no-deprecated-scope-attribute-bad"></span>

[Bad](./vue.md#vue-no-deprecated-scope-attribute-bad)

<span id="vue-no-deprecated-scope-attribute-good"></span>

[Good](./vue.md#vue-no-deprecated-scope-attribute-good)

[Group examples](./vue.md#vue-no-deprecated-scope-attribute) · [Rule reference](./reference/vue-no-deprecated-scope-attribute.md)

### `vue/no-deprecated-slot-attribute`

Disallow the deprecated `slot` attribute

<span id="vue-no-deprecated-slot-attribute-bad"></span>

[Bad](./vue.md#vue-no-deprecated-slot-attribute-bad)

<span id="vue-no-deprecated-slot-attribute-good"></span>

[Good](./vue.md#vue-no-deprecated-slot-attribute-good)

[Group examples](./vue.md#vue-no-deprecated-slot-attribute) · [Rule reference](./reference/vue-no-deprecated-slot-attribute.md)

### `vue/no-deprecated-slot-scope-attribute`

Disallow the deprecated `slot-scope` attribute

<span id="vue-no-deprecated-slot-scope-attribute-bad"></span>

[Bad](./vue.md#vue-no-deprecated-slot-scope-attribute-bad)

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

[Good](./vue.md#vue-no-deprecated-slot-scope-attribute-good)

[Group examples](./vue.md#vue-no-deprecated-slot-scope-attribute) · [Rule reference](./reference/vue-no-deprecated-slot-scope-attribute.md)

### `vue/no-deprecated-v-bind-sync`

Disallow the deprecated `.sync` modifier on `v-bind`

<span id="vue-no-deprecated-v-bind-sync-bad"></span>

[Bad](./vue.md#vue-no-deprecated-v-bind-sync-bad)

<span id="vue-no-deprecated-v-bind-sync-good"></span>

[Good](./vue.md#vue-no-deprecated-v-bind-sync-good)

[Group examples](./vue.md#vue-no-deprecated-v-bind-sync) · [Rule reference](./reference/vue-no-deprecated-v-bind-sync.md)

### `vue/no-deprecated-v-is`

Disallow the deprecated `v-is` directive

<span id="vue-no-deprecated-v-is-bad"></span>

[Bad](./vue.md#vue-no-deprecated-v-is-bad)

<span id="vue-no-deprecated-v-is-good"></span>

[Good](./vue.md#vue-no-deprecated-v-is-good)

[Group examples](./vue.md#vue-no-deprecated-v-is) · [Rule reference](./reference/vue-no-deprecated-v-is.md)

### `vue/no-deprecated-v-on-native-modifier`

Disallow the deprecated `.native` modifier on `v-on`

<span id="vue-no-deprecated-v-on-native-modifier-bad"></span>

[Bad](./vue.md#vue-no-deprecated-v-on-native-modifier-bad)

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

[Good](./vue.md#vue-no-deprecated-v-on-native-modifier-good)

[Group examples](./vue.md#vue-no-deprecated-v-on-native-modifier) · [Rule reference](./reference/vue-no-deprecated-v-on-native-modifier.md)

### `vue/no-deprecated-v-on-number-modifiers`

Disallow deprecated numeric `keyCode` modifiers on `v-on`

<span id="vue-no-deprecated-v-on-number-modifiers-bad"></span>

[Bad](./vue.md#vue-no-deprecated-v-on-number-modifiers-bad)

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

[Good](./vue.md#vue-no-deprecated-v-on-number-modifiers-good)

[Group examples](./vue.md#vue-no-deprecated-v-on-number-modifiers) · [Rule reference](./reference/vue-no-deprecated-v-on-number-modifiers.md)

### `vue/no-dupe-v-else-if`

Disallow duplicate conditions in `v-if` / `v-else-if` chains

<span id="vue-no-dupe-v-else-if-bad"></span>

[Bad](./vue.md#vue-no-dupe-v-else-if-bad)

<span id="vue-no-dupe-v-else-if-good"></span>

[Good](./vue.md#vue-no-dupe-v-else-if-good)

[Group examples](./vue.md#vue-no-dupe-v-else-if) · [Rule reference](./reference/vue-no-dupe-v-else-if.md)

### `vue/no-duplicate-attributes`

Disallow duplicate attributes on the same element

<span id="vue-no-duplicate-attributes-bad"></span>

[Bad](./vue.md#vue-no-duplicate-attributes-bad)

<span id="vue-no-duplicate-attributes-good"></span>

[Good](./vue.md#vue-no-duplicate-attributes-good)

[Group examples](./vue.md#vue-no-duplicate-attributes) · [Rule reference](./reference/vue-no-duplicate-attributes.md)

### `vue/no-multiple-template-root`

Disallow multiple root nodes in a template

<span id="vue-no-multiple-template-root-bad"></span>

[Bad](./vue.md#vue-no-multiple-template-root-bad)

<span id="vue-no-multiple-template-root-good"></span>

[Good](./vue.md#vue-no-multiple-template-root-good)

[Group examples](./vue.md#vue-no-multiple-template-root) · [Rule reference](./reference/vue-no-multiple-template-root.md)

### `vue/no-mutating-props`

Disallow mutating component props

<span id="vue-no-mutating-props-bad"></span>

[Bad](./vue.md#vue-no-mutating-props-bad)

<span id="vue-no-mutating-props-good"></span>

[Good](./vue.md#vue-no-mutating-props-good)

[Group examples](./vue.md#vue-no-mutating-props) · [Rule reference](./reference/vue-no-mutating-props.md)

### `vue/no-reserved-component-names`

Disallow the use of reserved names as component names

<span id="vue-no-reserved-component-names-bad"></span>

[Bad](./vue.md#vue-no-reserved-component-names-bad)

<span id="vue-no-reserved-component-names-good"></span>

[Good](./vue.md#vue-no-reserved-component-names-good)

[Group examples](./vue.md#vue-no-reserved-component-names) · [Rule reference](./reference/vue-no-reserved-component-names.md)

### `vue/no-template-key`

Disallow `key` attribute on `<template>`

<span id="vue-no-template-key-bad"></span>

[Bad](./vue.md#vue-no-template-key-bad)

<span id="vue-no-template-key-good"></span>

[Good](./vue.md#vue-no-template-key-good)

[Group examples](./vue.md#vue-no-template-key) · [Rule reference](./reference/vue-no-template-key.md)

### `vue/no-textarea-mustache`

Disallow mustache interpolation in `<textarea>`

<span id="vue-no-textarea-mustache-bad"></span>

[Bad](./vue.md#vue-no-textarea-mustache-bad)

<span id="vue-no-textarea-mustache-good"></span>

[Good](./vue.md#vue-no-textarea-mustache-good)

[Group examples](./vue.md#vue-no-textarea-mustache) · [Rule reference](./reference/vue-no-textarea-mustache.md)

### `vue/no-unused-components`

Disallow registering components that are not used inside templates

<span id="vue-no-unused-components-bad"></span>

[Bad](./vue.md#vue-no-unused-components-bad)

<span id="vue-no-unused-components-good"></span>

[Good](./vue.md#vue-no-unused-components-good)

[Group examples](./vue.md#vue-no-unused-components) · [Rule reference](./reference/vue-no-unused-components.md)

### `vue/no-unused-vars`

Disallow unused variable definitions in v-for and v-slot directives

<span id="vue-no-unused-vars-bad"></span>

[Bad](./vue.md#vue-no-unused-vars-bad)

<span id="vue-no-unused-vars-good"></span>

[Good](./vue.md#vue-no-unused-vars-good)

[Group examples](./vue.md#vue-no-unused-vars) · [Rule reference](./reference/vue-no-unused-vars.md)

### `vue/no-use-v-if-with-v-for`

Disallow using `v-if` on the same element as `v-for`

<span id="vue-no-use-v-if-with-v-for-bad"></span>

[Bad](./vue.md#vue-no-use-v-if-with-v-for-bad)

<span id="vue-no-use-v-if-with-v-for-good"></span>

[Good](./vue.md#vue-no-use-v-if-with-v-for-good)

[Group examples](./vue.md#vue-no-use-v-if-with-v-for) · [Rule reference](./reference/vue-no-use-v-if-with-v-for.md)

### `vue/no-useless-template-attributes`

Disallow useless attributes on `<template>` elements

<span id="vue-no-useless-template-attributes-bad"></span>

[Bad](./vue.md#vue-no-useless-template-attributes-bad)

<span id="vue-no-useless-template-attributes-good"></span>

[Good](./vue.md#vue-no-useless-template-attributes-good)

[Group examples](./vue.md#vue-no-useless-template-attributes) · [Rule reference](./reference/vue-no-useless-template-attributes.md)

### `vue/no-v-for-template-key-on-child`

Disallow `key` on the child of a `<template v-for>`

<span id="vue-no-v-for-template-key-on-child-bad"></span>

[Bad](./vue.md#vue-no-v-for-template-key-on-child-bad)

<span id="vue-no-v-for-template-key-on-child-good"></span>

[Good](./vue.md#vue-no-v-for-template-key-on-child-good)

[Group examples](./vue.md#vue-no-v-for-template-key-on-child) · [Rule reference](./reference/vue-no-v-for-template-key-on-child.md)

### `vue/no-v-html`

Warn against v-html to prevent XSS vulnerabilities

<span id="vue-no-v-html-bad"></span>

[Bad](./vue.md#vue-no-v-html-bad)

<span id="vue-no-v-html-good"></span>

[Good](./vue.md#vue-no-v-html-good)

[Group examples](./vue.md#vue-no-v-html) · [Rule reference](./reference/vue-no-v-html.md)

### `vue/no-v-text-v-html-on-component`

Disallow v-text / v-html on component elements

<span id="vue-no-v-text-v-html-on-component-bad"></span>

[Bad](./vue.md#vue-no-v-text-v-html-on-component-bad)

<span id="vue-no-v-text-v-html-on-component-good"></span>

[Good](./vue.md#vue-no-v-text-v-html-on-component-good)

[Group examples](./vue.md#vue-no-v-text-v-html-on-component) · [Rule reference](./reference/vue-no-v-text-v-html-on-component.md)

### `vue/permitted-contents`

Enforce HTML content model rules

<span id="vue-permitted-contents-bad"></span>

[Bad](./vue.md#vue-permitted-contents-bad)

<span id="vue-permitted-contents-good"></span>

[Good](./vue.md#vue-permitted-contents-good)

[Group examples](./vue.md#vue-permitted-contents) · [Rule reference](./reference/vue-permitted-contents.md)

### `vue/require-component-is`

Require `v-bind:is` on `<component>` elements

<span id="vue-require-component-is-bad"></span>

[Bad](./vue.md#vue-require-component-is-bad)

<span id="vue-require-component-is-good"></span>

[Good](./vue.md#vue-require-component-is-good)

[Group examples](./vue.md#vue-require-component-is) · [Rule reference](./reference/vue-require-component-is.md)

### `vue/require-toggle-inside-transition`

Require a toggle on the element wrapped by `<transition>`

<span id="vue-require-toggle-inside-transition-bad"></span>

[Bad](./vue.md#vue-require-toggle-inside-transition-bad)

<span id="vue-require-toggle-inside-transition-good"></span>

[Good](./vue.md#vue-require-toggle-inside-transition-good)

[Group examples](./vue.md#vue-require-toggle-inside-transition) · [Rule reference](./reference/vue-require-toggle-inside-transition.md)

### `vue/require-v-for-key`

Require `v-bind:key` with `v-for` directives

<span id="vue-require-v-for-key-bad"></span>

[Bad](./vue.md#vue-require-v-for-key-bad)

<span id="vue-require-v-for-key-good"></span>

[Good](./vue.md#vue-require-v-for-key-good)

[Group examples](./vue.md#vue-require-v-for-key) · [Rule reference](./reference/vue-require-v-for-key.md)

### `vue/use-v-on-exact`

Enforce `.exact` modifier on `v-on` when there are modifier-based handlers

<span id="vue-use-v-on-exact-bad"></span>

[Bad](./vue.md#vue-use-v-on-exact-bad)

<span id="vue-use-v-on-exact-good"></span>

[Good](./vue.md#vue-use-v-on-exact-good)

[Group examples](./vue.md#vue-use-v-on-exact) · [Rule reference](./reference/vue-use-v-on-exact.md)

### `vue/valid-attribute-name`

Require valid attribute names

<span id="vue-valid-attribute-name-bad"></span>

[Bad](./vue.md#vue-valid-attribute-name-bad)

<span id="vue-valid-attribute-name-good"></span>

[Good](./vue.md#vue-valid-attribute-name-good)

[Group examples](./vue.md#vue-valid-attribute-name) · [Rule reference](./reference/vue-valid-attribute-name.md)

### `vue/valid-template-root`

Enforce a valid `<template>` root for Vue 3 fragment semantics

<span id="vue-valid-template-root-bad"></span>

[Bad](./vue.md#vue-valid-template-root-bad)

<span id="vue-valid-template-root-good"></span>

[Good](./vue.md#vue-valid-template-root-good)

[Group examples](./vue.md#vue-valid-template-root) · [Rule reference](./reference/vue-valid-template-root.md)

### `vue/valid-v-bind`

Enforce valid `v-bind` directives

<span id="vue-valid-v-bind-bad"></span>

[Bad](./vue.md#vue-valid-v-bind-bad)

<span id="vue-valid-v-bind-good"></span>

[Good](./vue.md#vue-valid-v-bind-good)

[Group examples](./vue.md#vue-valid-v-bind) · [Rule reference](./reference/vue-valid-v-bind.md)

### `vue/valid-v-cloak`

Enforce valid `v-cloak` directives

<span id="vue-valid-v-cloak-bad"></span>

[Bad](./vue.md#vue-valid-v-cloak-bad)

<span id="vue-valid-v-cloak-good"></span>

[Good](./vue.md#vue-valid-v-cloak-good)

[Group examples](./vue.md#vue-valid-v-cloak) · [Rule reference](./reference/vue-valid-v-cloak.md)

### `vue/valid-v-else`

Enforce valid `v-else` directives

<span id="vue-valid-v-else-bad"></span>

[Bad](./vue.md#vue-valid-v-else-bad)

<span id="vue-valid-v-else-good"></span>

[Good](./vue.md#vue-valid-v-else-good)

[Group examples](./vue.md#vue-valid-v-else) · [Rule reference](./reference/vue-valid-v-else.md)

### `vue/valid-v-else-if`

Enforce valid `v-else-if` directives

<span id="vue-valid-v-else-if-bad"></span>

[Bad](./vue.md#vue-valid-v-else-if-bad)

<span id="vue-valid-v-else-if-good"></span>

[Good](./vue.md#vue-valid-v-else-if-good)

[Group examples](./vue.md#vue-valid-v-else-if) · [Rule reference](./reference/vue-valid-v-else-if.md)

### `vue/valid-v-for`

Enforce valid `v-for` directives

<span id="vue-valid-v-for-bad"></span>

[Bad](./vue.md#vue-valid-v-for-bad)

<span id="vue-valid-v-for-good"></span>

[Good](./vue.md#vue-valid-v-for-good)

[Group examples](./vue.md#vue-valid-v-for) · [Rule reference](./reference/vue-valid-v-for.md)

### `vue/valid-v-html`

Enforce valid `v-html` directives

<span id="vue-valid-v-html-bad"></span>

[Bad](./vue.md#vue-valid-v-html-bad)

<span id="vue-valid-v-html-good"></span>

[Good](./vue.md#vue-valid-v-html-good)

[Group examples](./vue.md#vue-valid-v-html) · [Rule reference](./reference/vue-valid-v-html.md)

### `vue/valid-v-if`

Enforce valid `v-if` directives

<span id="vue-valid-v-if-bad"></span>

[Bad](./vue.md#vue-valid-v-if-bad)

<span id="vue-valid-v-if-good"></span>

[Good](./vue.md#vue-valid-v-if-good)

[Group examples](./vue.md#vue-valid-v-if) · [Rule reference](./reference/vue-valid-v-if.md)

### `vue/valid-v-is`

Enforce valid `v-is` directives

<span id="vue-valid-v-is-bad"></span>

[Bad](./vue.md#vue-valid-v-is-bad)

<span id="vue-valid-v-is-good"></span>

[Good](./vue.md#vue-valid-v-is-good)

[Group examples](./vue.md#vue-valid-v-is) · [Rule reference](./reference/vue-valid-v-is.md)

### `vue/valid-v-memo`

Enforce valid `v-memo` directives

<span id="vue-valid-v-memo-bad"></span>

[Bad](./vue.md#vue-valid-v-memo-bad)

<span id="vue-valid-v-memo-good"></span>

[Good](./vue.md#vue-valid-v-memo-good)

[Group examples](./vue.md#vue-valid-v-memo) · [Rule reference](./reference/vue-valid-v-memo.md)

### `vue/valid-v-model`

Enforce valid `v-model` directives

<span id="vue-valid-v-model-bad"></span>

[Bad](./vue.md#vue-valid-v-model-bad)

<span id="vue-valid-v-model-good"></span>

[Good](./vue.md#vue-valid-v-model-good)

[Group examples](./vue.md#vue-valid-v-model) · [Rule reference](./reference/vue-valid-v-model.md)

### `vue/valid-v-on`

Enforce valid `v-on` directives

<span id="vue-valid-v-on-bad"></span>

[Bad](./vue.md#vue-valid-v-on-bad)

<span id="vue-valid-v-on-good"></span>

[Good](./vue.md#vue-valid-v-on-good)

[Group examples](./vue.md#vue-valid-v-on) · [Rule reference](./reference/vue-valid-v-on.md)

### `vue/valid-v-once`

Enforce valid `v-once` directives

<span id="vue-valid-v-once-bad"></span>

[Bad](./vue.md#vue-valid-v-once-bad)

<span id="vue-valid-v-once-good"></span>

[Good](./vue.md#vue-valid-v-once-good)

[Group examples](./vue.md#vue-valid-v-once) · [Rule reference](./reference/vue-valid-v-once.md)

### `vue/valid-v-pre`

Enforce valid `v-pre` directives

<span id="vue-valid-v-pre-bad"></span>

[Bad](./vue.md#vue-valid-v-pre-bad)

<span id="vue-valid-v-pre-good"></span>

[Good](./vue.md#vue-valid-v-pre-good)

[Group examples](./vue.md#vue-valid-v-pre) · [Rule reference](./reference/vue-valid-v-pre.md)

### `vue/valid-v-show`

Enforce valid `v-show` directives

<span id="vue-valid-v-show-bad"></span>

[Bad](./vue.md#vue-valid-v-show-bad)

<span id="vue-valid-v-show-good"></span>

[Good](./vue.md#vue-valid-v-show-good)

[Group examples](./vue.md#vue-valid-v-show) · [Rule reference](./reference/vue-valid-v-show.md)

### `vue/valid-v-slot`

Enforce valid `v-slot` directives

<span id="vue-valid-v-slot-bad"></span>

[Bad](./vue.md#vue-valid-v-slot-bad)

<span id="vue-valid-v-slot-good"></span>

[Good](./vue.md#vue-valid-v-slot-good)

[Group examples](./vue.md#vue-valid-v-slot) · [Rule reference](./reference/vue-valid-v-slot.md)

### `vue/valid-v-text`

Enforce valid `v-text` directives

<span id="vue-valid-v-text-bad"></span>

[Bad](./vue.md#vue-valid-v-text-bad)

<span id="vue-valid-v-text-good"></span>

[Good](./vue.md#vue-valid-v-text-good)

[Group examples](./vue.md#vue-valid-v-text) · [Rule reference](./reference/vue-valid-v-text.md)

### `vue/attribute-hyphenation`

Enforce attribute naming style on custom components

<span id="vue-attribute-hyphenation-bad"></span>

[Bad](./vue.md#vue-attribute-hyphenation-bad)

<span id="vue-attribute-hyphenation-good"></span>

[Good](./vue.md#vue-attribute-hyphenation-good)

[Group examples](./vue.md#vue-attribute-hyphenation) · [Rule reference](./reference/vue-attribute-hyphenation.md)

### `vue/component-definition-name-casing`

Enforce PascalCase or kebab-case for component definition names

<span id="vue-component-definition-name-casing-bad"></span>

[Bad](./vue.md#vue-component-definition-name-casing-bad)

<span id="vue-component-definition-name-casing-good"></span>

[Good](./vue.md#vue-component-definition-name-casing-good)

[Group examples](./vue.md#vue-component-definition-name-casing) · [Rule reference](./reference/vue-component-definition-name-casing.md)

### `vue/html-quotes`

Enforce quotes style of HTML attributes

<span id="vue-html-quotes-bad"></span>

[Bad](./vue.md#vue-html-quotes-bad)

<span id="vue-html-quotes-good"></span>

[Good](./vue.md#vue-html-quotes-good)

[Group examples](./vue.md#vue-html-quotes) · [Rule reference](./reference/vue-html-quotes.md)

### `vue/html-self-closing`

Enforce self-closing style

<span id="vue-html-self-closing-bad"></span>

[Bad](./vue.md#vue-html-self-closing-bad)

<span id="vue-html-self-closing-good"></span>

[Good](./vue.md#vue-html-self-closing-good)

[Group examples](./vue.md#vue-html-self-closing) · [Rule reference](./reference/vue-html-self-closing.md)

### `vue/mustache-interpolation-spacing`

Enforce consistent spacing inside mustache interpolations

<span id="vue-mustache-interpolation-spacing-bad"></span>

[Bad](./vue.md#vue-mustache-interpolation-spacing-bad)

<span id="vue-mustache-interpolation-spacing-good"></span>

[Good](./vue.md#vue-mustache-interpolation-spacing-good)

[Group examples](./vue.md#vue-mustache-interpolation-spacing) · [Rule reference](./reference/vue-mustache-interpolation-spacing.md)

### `vue/no-multi-spaces`

Disallow multiple consecutive spaces

<span id="vue-no-multi-spaces-bad"></span>

[Bad](./vue.md#vue-no-multi-spaces-bad)

<span id="vue-no-multi-spaces-good"></span>

[Good](./vue.md#vue-no-multi-spaces-good)

[Group examples](./vue.md#vue-no-multi-spaces) · [Rule reference](./reference/vue-no-multi-spaces.md)

### `vue/no-template-shadow`

Disallow variable names that shadow variables in outer scope

<span id="vue-no-template-shadow-bad"></span>

[Bad](./vue.md#vue-no-template-shadow-bad)

<span id="vue-no-template-shadow-good"></span>

[Good](./vue.md#vue-no-template-shadow-good)

[Group examples](./vue.md#vue-no-template-shadow) · [Rule reference](./reference/vue-no-template-shadow.md)

### `vue/no-unused-properties`

Disallow unused properties defined in defineProps

<span id="vue-no-unused-properties-bad"></span>

[Bad](./vue.md#vue-no-unused-properties-bad)

<span id="vue-no-unused-properties-good"></span>

[Good](./vue.md#vue-no-unused-properties-good)

[Group examples](./vue.md#vue-no-unused-properties) · [Rule reference](./reference/vue-no-unused-properties.md)

### `vue/prop-name-casing`

Enforce a casing for declared prop names

<span id="vue-prop-name-casing-bad"></span>

[Bad](./vue.md#vue-prop-name-casing-bad)

<span id="vue-prop-name-casing-good"></span>

[Good](./vue.md#vue-prop-name-casing-good)

[Group examples](./vue.md#vue-prop-name-casing) · [Rule reference](./reference/vue-prop-name-casing.md)

### `vue/v-bind-style`

Enforce `v-bind` directive style

<span id="vue-v-bind-style-bad"></span>

[Bad](./vue.md#vue-v-bind-style-bad)

<span id="vue-v-bind-style-good"></span>

[Good](./vue.md#vue-v-bind-style-good)

[Group examples](./vue.md#vue-v-bind-style) · [Rule reference](./reference/vue-v-bind-style.md)

### `vue/v-on-style`

Enforce `v-on` directive style

<span id="vue-v-on-style-bad"></span>

[Bad](./vue.md#vue-v-on-style-bad)

<span id="vue-v-on-style-good"></span>

[Good](./vue.md#vue-v-on-style-good)

[Group examples](./vue.md#vue-v-on-style) · [Rule reference](./reference/vue-v-on-style.md)

### `vue/v-slot-style`

Enforce `v-slot` directive style

<span id="vue-v-slot-style-bad"></span>

[Bad](./vue.md#vue-v-slot-style-bad)

<span id="vue-v-slot-style-good"></span>

[Good](./vue.md#vue-v-slot-style-good)

[Group examples](./vue.md#vue-v-slot-style) · [Rule reference](./reference/vue-v-slot-style.md)

### `ssr/no-browser-globals-in-ssr`

Disallow browser-only globals in SSR context

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

[Bad](./ssr.md#ssr-no-browser-globals-in-ssr-bad)

<span id="ssr-no-browser-globals-in-ssr-good"></span>

[Good](./ssr.md#ssr-no-browser-globals-in-ssr-good)

[Group examples](./ssr.md#ssr-no-browser-globals-in-ssr) · [Rule reference](./reference/ssr-no-browser-globals-in-ssr.md)

### `ssr/no-hydration-mismatch`

Disallow non-deterministic values that cause hydration mismatch

<span id="ssr-no-hydration-mismatch-bad"></span>

[Bad](./ssr.md#ssr-no-hydration-mismatch-bad)

<span id="ssr-no-hydration-mismatch-good"></span>

[Good](./ssr.md#ssr-no-hydration-mismatch-good)

[Group examples](./ssr.md#ssr-no-hydration-mismatch) · [Rule reference](./reference/ssr-no-hydration-mismatch.md)

### `vue/a11y-img-alt`

Require alt attribute on images for accessibility

<span id="vue-a11y-img-alt-bad"></span>

[Bad](./vue.md#vue-a11y-img-alt-bad)

<span id="vue-a11y-img-alt-good"></span>

[Good](./vue.md#vue-a11y-img-alt-good)

[Group examples](./vue.md#vue-a11y-img-alt) · [Rule reference](./reference/vue-a11y-img-alt.md)

### `vue/attribute-order`

Enforce a consistent order of attributes

<span id="vue-attribute-order-bad"></span>

[Bad](./vue.md#vue-attribute-order-bad)

<span id="vue-attribute-order-good"></span>

[Good](./vue.md#vue-attribute-order-good)

[Group examples](./vue.md#vue-attribute-order) · [Rule reference](./reference/vue-attribute-order.md)

### `vue/component-name-in-template-casing`

Enforce specific casing for component names in templates

<span id="vue-component-name-in-template-casing-bad"></span>

[Bad](./vue.md#vue-component-name-in-template-casing-bad)

<span id="vue-component-name-in-template-casing-good"></span>

[Good](./vue.md#vue-component-name-in-template-casing-good)

[Group examples](./vue.md#vue-component-name-in-template-casing) · [Rule reference](./reference/vue-component-name-in-template-casing.md)

### `vue/html-button-has-type`

Require an explicit valid type on button elements

<span id="vue-html-button-has-type-bad"></span>

[Bad](./vue.md#vue-html-button-has-type-bad)

<span id="vue-html-button-has-type-good"></span>

[Good](./vue.md#vue-html-button-has-type-good)

[Group examples](./vue.md#vue-html-button-has-type) · [Rule reference](./reference/vue-html-button-has-type.md)

### `vue/max-template-complexity`

Limit a component's own template complexity (cyclomatic and cognitive)

<span id="vue-max-template-complexity-bad"></span>

[Bad](./vue.md#vue-max-template-complexity-bad)

<span id="vue-max-template-complexity-good"></span>

[Good](./vue.md#vue-max-template-complexity-good)

[Group examples](./vue.md#vue-max-template-complexity) · [Rule reference](./reference/vue-max-template-complexity.md)

### `vue/no-array-index-key`

Disallow using the v-for index variable directly as the :key

<span id="vue-no-array-index-key-bad"></span>

[Bad](./vue.md#vue-no-array-index-key-bad)

<span id="vue-no-array-index-key-good"></span>

[Good](./vue.md#vue-no-array-index-key-good)

[Group examples](./vue.md#vue-no-array-index-key) · [Rule reference](./reference/vue-no-array-index-key.md)

### `vue/no-bare-strings-in-template`

Disallow raw human-readable text in the template that should be internationalized

<span id="vue-no-bare-strings-in-template-bad"></span>

[Bad](./vue.md#vue-no-bare-strings-in-template-bad)

<span id="vue-no-bare-strings-in-template-good"></span>

[Good](./vue.md#vue-no-bare-strings-in-template-good)

[Group examples](./vue.md#vue-no-bare-strings-in-template) · [Rule reference](./reference/vue-no-bare-strings-in-template.md)

### `vue/no-boolean-attr-value`

Disallow explicit values for boolean HTML attributes

<span id="vue-no-boolean-attr-value-bad"></span>

[Bad](./vue.md#vue-no-boolean-attr-value-bad)

<span id="vue-no-boolean-attr-value-good"></span>

[Good](./vue.md#vue-no-boolean-attr-value-good)

[Group examples](./vue.md#vue-no-boolean-attr-value) · [Rule reference](./reference/vue-no-boolean-attr-value.md)

### `vue/no-empty-component-block`

Disallow empty SFC blocks

<span id="vue-no-empty-component-block-bad"></span>

[Bad](./vue.md#vue-no-empty-component-block-bad)

<span id="vue-no-empty-component-block-good"></span>

[Good](./vue.md#vue-no-empty-component-block-good)

[Group examples](./vue.md#vue-no-empty-component-block) · [Rule reference](./reference/vue-no-empty-component-block.md)

### `vue/no-inline-style`

Discourage use of inline style attributes

<span id="vue-no-inline-style-bad"></span>

[Bad](./vue.md#vue-no-inline-style-bad)

<span id="vue-no-inline-style-good"></span>

[Good](./vue.md#vue-no-inline-style-good)

[Group examples](./vue.md#vue-no-inline-style) · [Rule reference](./reference/vue-no-inline-style.md)

### `vue/no-invalid-html-attribute`

Disallow invalid static values for HTML attributes

<span id="vue-no-invalid-html-attribute-bad"></span>

[Bad](./vue.md#vue-no-invalid-html-attribute-bad)

<span id="vue-no-invalid-html-attribute-good"></span>

[Good](./vue.md#vue-no-invalid-html-attribute-good)

[Group examples](./vue.md#vue-no-invalid-html-attribute) · [Rule reference](./reference/vue-no-invalid-html-attribute.md)

### `vue/no-lone-template`

Disallow unnecessary `<template>` elements

<span id="vue-no-lone-template-bad"></span>

[Bad](./vue.md#vue-no-lone-template-bad)

<span id="vue-no-lone-template-good"></span>

[Good](./vue.md#vue-no-lone-template-good)

[Group examples](./vue.md#vue-no-lone-template) · [Rule reference](./reference/vue-no-lone-template.md)

### `vue/no-multiple-objects-in-class`

Disallow multiple object literals inside a :class array binding

<span id="vue-no-multiple-objects-in-class-bad"></span>

[Bad](./vue.md#vue-no-multiple-objects-in-class-bad)

<span id="vue-no-multiple-objects-in-class-good"></span>

[Good](./vue.md#vue-no-multiple-objects-in-class-good)

[Group examples](./vue.md#vue-no-multiple-objects-in-class) · [Rule reference](./reference/vue-no-multiple-objects-in-class.md)

### `vue/no-negated-v-if-condition`

Disallow a negated v-if condition when the chain has a v-else

<span id="vue-no-negated-v-if-condition-bad"></span>

[Bad](./vue.md#vue-no-negated-v-if-condition-bad)

<span id="vue-no-negated-v-if-condition-good"></span>

[Good](./vue.md#vue-no-negated-v-if-condition-good)

[Group examples](./vue.md#vue-no-negated-v-if-condition) · [Rule reference](./reference/vue-no-negated-v-if-condition.md)

### `vue/no-non-component-keep-alive-child`

Disallow plain element wrappers directly below `<KeepAlive>`

<span id="vue-no-non-component-keep-alive-child-bad"></span>

[Bad](./vue.md#vue-no-non-component-keep-alive-child-bad)

<span id="vue-no-non-component-keep-alive-child-good"></span>

[Good](./vue.md#vue-no-non-component-keep-alive-child-good)

[Group examples](./vue.md#vue-no-non-component-keep-alive-child) · [Rule reference](./reference/vue-no-non-component-keep-alive-child.md)

### `vue/no-preprocessor-lang`

Discourage CSS preprocessor usage in favor of modern CSS

<span id="vue-no-preprocessor-lang-bad"></span>

[Bad](./vue.md#vue-no-preprocessor-lang-bad)

<span id="vue-no-preprocessor-lang-good"></span>

[Good](./vue.md#vue-no-preprocessor-lang-good)

[Group examples](./vue.md#vue-no-preprocessor-lang) · [Rule reference](./reference/vue-no-preprocessor-lang.md)

### `vue/no-root-v-if`

Disallow v-if on the single root element of a template

<span id="vue-no-root-v-if-bad"></span>

[Bad](./vue.md#vue-no-root-v-if-bad)

<span id="vue-no-root-v-if-good"></span>

[Good](./vue.md#vue-no-root-v-if-good)

[Group examples](./vue.md#vue-no-root-v-if) · [Rule reference](./reference/vue-no-root-v-if.md)

### `vue/no-script-non-standard-lang`

Discourage non-standard script lang values

<span id="vue-no-script-non-standard-lang-bad"></span>

[Bad](./vue.md#vue-no-script-non-standard-lang-bad)

<span id="vue-no-script-non-standard-lang-good"></span>

[Good](./vue.md#vue-no-script-non-standard-lang-good)

[Group examples](./vue.md#vue-no-script-non-standard-lang) · [Rule reference](./reference/vue-no-script-non-standard-lang.md)

### `vue/no-src-attribute`

Discourage src attribute on SFC blocks

<span id="vue-no-src-attribute-bad"></span>

[Bad](./vue.md#vue-no-src-attribute-bad)

<span id="vue-no-src-attribute-good"></span>

[Good](./vue.md#vue-no-src-attribute-good)

[Group examples](./vue.md#vue-no-src-attribute) · [Rule reference](./reference/vue-no-src-attribute.md)

### `vue/no-static-inline-styles`

Disallow static inline style attributes

<span id="vue-no-static-inline-styles-bad"></span>

[Bad](./vue.md#vue-no-static-inline-styles-bad)

<span id="vue-no-static-inline-styles-good"></span>

[Good](./vue.md#vue-no-static-inline-styles-good)

[Group examples](./vue.md#vue-no-static-inline-styles) · [Rule reference](./reference/vue-no-static-inline-styles.md)

### `vue/no-template-lang`

Discourage lang attribute on template block

<span id="vue-no-template-lang-bad"></span>

[Bad](./vue.md#vue-no-template-lang-bad)

<span id="vue-no-template-lang-good"></span>

[Good](./vue.md#vue-no-template-lang-good)

[Group examples](./vue.md#vue-no-template-lang) · [Rule reference](./reference/vue-no-template-lang.md)

### `vue/no-template-target-blank`

Disallow target="_blank" without rel="noopener noreferrer"

<span id="vue-no-template-target-blank-bad"></span>

[Bad](./vue.md#vue-no-template-target-blank-bad)

<span id="vue-no-template-target-blank-good"></span>

[Good](./vue.md#vue-no-template-target-blank-good)

[Group examples](./vue.md#vue-no-template-target-blank) · [Rule reference](./reference/vue-no-template-target-blank.md)

### `vue/no-undefined-refs`

Disallow undefined variable references in templates

<span id="vue-no-undefined-refs-bad"></span>

[Bad](./vue.md#vue-no-undefined-refs-bad)

<span id="vue-no-undefined-refs-good"></span>

[Good](./vue.md#vue-no-undefined-refs-good)

[Group examples](./vue.md#vue-no-undefined-refs) · [Rule reference](./reference/vue-no-undefined-refs.md)

### `vue/no-unsafe-url`

Warn about potentially unsafe URL bindings

<span id="vue-no-unsafe-url-bad"></span>

[Bad](./vue.md#vue-no-unsafe-url-bad)

<span id="vue-no-unsafe-url-good"></span>

[Good](./vue.md#vue-no-unsafe-url-good)

[Group examples](./vue.md#vue-no-unsafe-url) · [Rule reference](./reference/vue-no-unsafe-url.md)

### `vue/no-unsandboxed-iframe`

Require a sandbox attribute on iframe elements

<span id="vue-no-unsandboxed-iframe-bad"></span>

[Bad](./vue.md#vue-no-unsandboxed-iframe-bad)

<span id="vue-no-unsandboxed-iframe-good"></span>

[Good](./vue.md#vue-no-unsandboxed-iframe-good)

[Group examples](./vue.md#vue-no-unsandboxed-iframe) · [Rule reference](./reference/vue-no-unsandboxed-iframe.md)

### `vue/no-unused-refs`

Report template refs (ref="x") never referenced in &lt;script&gt;

<span id="vue-no-unused-refs-bad"></span>

[Bad](./vue.md#vue-no-unused-refs-bad)

<span id="vue-no-unused-refs-good"></span>

[Good](./vue.md#vue-no-unused-refs-good)

[Group examples](./vue.md#vue-no-unused-refs) · [Rule reference](./reference/vue-no-unused-refs.md)

### `vue/no-unused-setup-bindings`

Disallow unread script setup bindings

<span id="vue-no-unused-setup-bindings-bad"></span>

[Bad](./vue.md#vue-no-unused-setup-bindings-bad)

<span id="vue-no-unused-setup-bindings-good"></span>

[Good](./vue.md#vue-no-unused-setup-bindings-good)

[Group examples](./vue.md#vue-no-unused-setup-bindings) · [Rule reference](./reference/vue-no-unused-setup-bindings.md)

### `vue/no-use-v-else-with-v-for`

Disallow using `v-else-if` or `v-else` on the same element as `v-for`

<span id="vue-no-use-v-else-with-v-for-bad"></span>

[Bad](./vue.md#vue-no-use-v-else-with-v-for-bad)

<span id="vue-no-use-v-else-with-v-for-good"></span>

[Good](./vue.md#vue-no-use-v-else-with-v-for-good)

[Group examples](./vue.md#vue-no-use-v-else-with-v-for) · [Rule reference](./reference/vue-no-use-v-else-with-v-for.md)

### `vue/no-useless-mustaches`

Disallow a mustache interpolation whose expression is a constant string literal

<span id="vue-no-useless-mustaches-bad"></span>

[Bad](./vue.md#vue-no-useless-mustaches-bad)

<span id="vue-no-useless-mustaches-good"></span>

[Good](./vue.md#vue-no-useless-mustaches-good)

[Group examples](./vue.md#vue-no-useless-mustaches) · [Rule reference](./reference/vue-no-useless-mustaches.md)

### `vue/no-useless-v-bind`

Disallow a v-bind whose value is a plain string literal

<span id="vue-no-useless-v-bind-bad"></span>

[Bad](./vue.md#vue-no-useless-v-bind-bad)

<span id="vue-no-useless-v-bind-good"></span>

[Good](./vue.md#vue-no-useless-v-bind-good)

[Group examples](./vue.md#vue-no-useless-v-bind) · [Rule reference](./reference/vue-no-useless-v-bind.md)

### `vue/no-v-text`

Disallow the v-text directive; prefer mustache interpolation

<span id="vue-no-v-text-bad"></span>

[Bad](./vue.md#vue-no-v-text-bad)

<span id="vue-no-v-text-good"></span>

[Good](./vue.md#vue-no-v-text-good)

[Group examples](./vue.md#vue-no-v-text) · [Rule reference](./reference/vue-no-v-text.md)

### `vue/prefer-props-shorthand`

Recommend shorthand syntax for props (Vue 3.4+)

<span id="vue-prefer-props-shorthand-bad"></span>

[Bad](./vue.md#vue-prefer-props-shorthand-bad)

<span id="vue-prefer-props-shorthand-good"></span>

[Good](./vue.md#vue-prefer-props-shorthand-good)

[Group examples](./vue.md#vue-prefer-props-shorthand) · [Rule reference](./reference/vue-prefer-props-shorthand.md)

### `vue/prefer-true-attribute-shorthand`

Prefer the shorthand for a boolean attribute bound to `true`

<span id="vue-prefer-true-attribute-shorthand-bad"></span>

[Bad](./vue.md#vue-prefer-true-attribute-shorthand-bad)

<span id="vue-prefer-true-attribute-shorthand-good"></span>

[Good](./vue.md#vue-prefer-true-attribute-shorthand-good)

[Group examples](./vue.md#vue-prefer-true-attribute-shorthand) · [Rule reference](./reference/vue-prefer-true-attribute-shorthand.md)

### `vue/require-component-registration`

Require explicit import or registration for components

<span id="vue-require-component-registration-bad"></span>

[Bad](./vue.md#vue-require-component-registration-bad)

<span id="vue-require-component-registration-good"></span>

[Good](./vue.md#vue-require-component-registration-good)

[Group examples](./vue.md#vue-require-component-registration) · [Rule reference](./reference/vue-require-component-registration.md)

### `vue/require-scoped-style`

Require scoped attribute on style tags

<span id="vue-require-scoped-style-bad"></span>

[Bad](./vue.md#vue-require-scoped-style-bad)

<span id="vue-require-scoped-style-good"></span>

[Good](./vue.md#vue-require-scoped-style-good)

[Group examples](./vue.md#vue-require-scoped-style) · [Rule reference](./reference/vue-require-scoped-style.md)

### `vue/scoped-event-names`

Recommend scoped event names using context:event format

<span id="vue-scoped-event-names-bad"></span>

[Bad](./vue.md#vue-scoped-event-names-bad)

<span id="vue-scoped-event-names-good"></span>

[Good](./vue.md#vue-scoped-event-names-good)

[Group examples](./vue.md#vue-scoped-event-names) · [Rule reference](./reference/vue-scoped-event-names.md)

### `vue/sfc-element-order`

Enforce consistent order of SFC top-level elements

<span id="vue-sfc-element-order-bad"></span>

[Bad](./vue.md#vue-sfc-element-order-bad)

<span id="vue-sfc-element-order-good"></span>

[Good](./vue.md#vue-sfc-element-order-good)

[Group examples](./vue.md#vue-sfc-element-order) · [Rule reference](./reference/vue-sfc-element-order.md)

### `vue/single-style-block`

Recommend having a single style block

<span id="vue-single-style-block-bad"></span>

[Bad](./vue.md#vue-single-style-block-bad)

<span id="vue-single-style-block-good"></span>

[Good](./vue.md#vue-single-style-block-good)

[Group examples](./vue.md#vue-single-style-block) · [Rule reference](./reference/vue-single-style-block.md)

### `vue/slot-name-casing`

Enforce kebab-case for named slots used via v-slot

<span id="vue-slot-name-casing-bad"></span>

[Bad](./vue.md#vue-slot-name-casing-bad)

<span id="vue-slot-name-casing-good"></span>

[Good](./vue.md#vue-slot-name-casing-good)

[Group examples](./vue.md#vue-slot-name-casing) · [Rule reference](./reference/vue-slot-name-casing.md)

### `vue/this-in-template`

Disallow `this.` in template expressions

<span id="vue-this-in-template-bad"></span>

[Bad](./vue.md#vue-this-in-template-bad)

<span id="vue-this-in-template-good"></span>

[Good](./vue.md#vue-this-in-template-good)

[Group examples](./vue.md#vue-this-in-template) · [Rule reference](./reference/vue-this-in-template.md)

### `vue/v-on-event-hyphenation`

Enforce hyphenation of custom event names in v-on on components

<span id="vue-v-on-event-hyphenation-bad"></span>

[Bad](./vue.md#vue-v-on-event-hyphenation-bad)

<span id="vue-v-on-event-hyphenation-good"></span>

[Good](./vue.md#vue-v-on-event-hyphenation-good)

[Group examples](./vue.md#vue-v-on-event-hyphenation) · [Rule reference](./reference/vue-v-on-event-hyphenation.md)

### `vue/v-on-handler-style`

Enforce writing v-on handlers as a method reference or an inline function

<span id="vue-v-on-handler-style-bad"></span>

[Bad](./vue.md#vue-v-on-handler-style-bad)

<span id="vue-v-on-handler-style-good"></span>

[Good](./vue.md#vue-v-on-handler-style-good)

[Group examples](./vue.md#vue-v-on-handler-style) · [Rule reference](./reference/vue-v-on-handler-style.md)

### `vue/warn-custom-block`

Warn about custom blocks in SFC files

<span id="vue-warn-custom-block-bad"></span>

[Bad](./vue.md#vue-warn-custom-block-bad)

<span id="vue-warn-custom-block-good"></span>

[Good](./vue.md#vue-warn-custom-block-good)

[Group examples](./vue.md#vue-warn-custom-block) · [Rule reference](./reference/vue-warn-custom-block.md)

### `vue/warn-custom-directive`

Warn about custom directives that need registration

<span id="vue-warn-custom-directive-bad"></span>

[Bad](./vue.md#vue-warn-custom-directive-bad)

<span id="vue-warn-custom-directive-good"></span>

[Good](./vue.md#vue-warn-custom-directive-good)

[Group examples](./vue.md#vue-warn-custom-directive) · [Rule reference](./reference/vue-warn-custom-directive.md)

### `a11y/alt-text`

Require alternative text for media elements

<span id="a11y-alt-text-bad"></span>

[Bad](./accessibility.md#a11y-alt-text-bad)

<span id="a11y-alt-text-good"></span>

[Good](./accessibility.md#a11y-alt-text-good)

[Group examples](./accessibility.md#a11y-alt-text) · [Rule reference](./reference/a11y-alt-text.md)

### `a11y/anchor-has-content`

Require anchor elements to have accessible content

<span id="a11y-anchor-has-content-bad"></span>

[Bad](./accessibility.md#a11y-anchor-has-content-bad)

<span id="a11y-anchor-has-content-good"></span>

[Good](./accessibility.md#a11y-anchor-has-content-good)

[Group examples](./accessibility.md#a11y-anchor-has-content) · [Rule reference](./reference/a11y-anchor-has-content.md)

### `a11y/anchor-is-valid`

Enforce valid href on anchor elements

<span id="a11y-anchor-is-valid-bad"></span>

[Bad](./accessibility.md#a11y-anchor-is-valid-bad)

<span id="a11y-anchor-is-valid-good"></span>

[Good](./accessibility.md#a11y-anchor-is-valid-good)

[Group examples](./accessibility.md#a11y-anchor-is-valid) · [Rule reference](./reference/a11y-anchor-is-valid.md)

### `a11y/aria-props`

Disallow invalid ARIA attributes

<span id="a11y-aria-props-bad"></span>

[Bad](./accessibility.md#a11y-aria-props-bad)

<span id="a11y-aria-props-good"></span>

[Good](./accessibility.md#a11y-aria-props-good)

[Group examples](./accessibility.md#a11y-aria-props) · [Rule reference](./reference/a11y-aria-props.md)

### `a11y/aria-role`

Elements with ARIA roles must use a valid, non-abstract ARIA role

<span id="a11y-aria-role-bad"></span>

[Bad](./accessibility.md#a11y-aria-role-bad)

<span id="a11y-aria-role-good"></span>

[Good](./accessibility.md#a11y-aria-role-good)

[Group examples](./accessibility.md#a11y-aria-role) · [Rule reference](./reference/a11y-aria-role.md)

### `a11y/aria-unsupported-elements`

Disallow ARIA attributes on elements that do not support them

<span id="a11y-aria-unsupported-elements-bad"></span>

[Bad](./accessibility.md#a11y-aria-unsupported-elements-bad)

<span id="a11y-aria-unsupported-elements-good"></span>

[Good](./accessibility.md#a11y-aria-unsupported-elements-good)

[Group examples](./accessibility.md#a11y-aria-unsupported-elements) · [Rule reference](./reference/a11y-aria-unsupported-elements.md)

### `a11y/click-events-have-key-events`

Require keyboard event handlers with click events

<span id="a11y-click-events-have-key-events-bad"></span>

[Bad](./accessibility.md#a11y-click-events-have-key-events-bad)

<span id="a11y-click-events-have-key-events-good"></span>

[Good](./accessibility.md#a11y-click-events-have-key-events-good)

[Group examples](./accessibility.md#a11y-click-events-have-key-events) · [Rule reference](./reference/a11y-click-events-have-key-events.md)

### `a11y/form-control-has-label`

Require form controls to have associated labels

<span id="a11y-form-control-has-label-bad"></span>

[Bad](./accessibility.md#a11y-form-control-has-label-bad)

<span id="a11y-form-control-has-label-good"></span>

[Good](./accessibility.md#a11y-form-control-has-label-good)

[Group examples](./accessibility.md#a11y-form-control-has-label) · [Rule reference](./reference/a11y-form-control-has-label.md)

### `a11y/heading-has-content`

Require heading elements to have accessible content

<span id="a11y-heading-has-content-bad"></span>

[Bad](./accessibility.md#a11y-heading-has-content-bad)

<span id="a11y-heading-has-content-good"></span>

[Good](./accessibility.md#a11y-heading-has-content-good)

[Group examples](./accessibility.md#a11y-heading-has-content) · [Rule reference](./reference/a11y-heading-has-content.md)

### `a11y/heading-levels`

Disallow skipping heading levels

<span id="a11y-heading-levels-bad"></span>

[Bad](./accessibility.md#a11y-heading-levels-bad)

<span id="a11y-heading-levels-good"></span>

[Good](./accessibility.md#a11y-heading-levels-good)

[Group examples](./accessibility.md#a11y-heading-levels) · [Rule reference](./reference/a11y-heading-levels.md)

### `a11y/iframe-has-title`

Require iframe elements to have a title attribute

<span id="a11y-iframe-has-title-bad"></span>

[Bad](./accessibility.md#a11y-iframe-has-title-bad)

<span id="a11y-iframe-has-title-good"></span>

[Good](./accessibility.md#a11y-iframe-has-title-good)

[Group examples](./accessibility.md#a11y-iframe-has-title) · [Rule reference](./reference/a11y-iframe-has-title.md)

### `a11y/img-alt`

Require alt attribute on images for accessibility

<span id="a11y-img-alt-bad"></span>

[Bad](./accessibility.md#a11y-img-alt-bad)

<span id="a11y-img-alt-good"></span>

[Good](./accessibility.md#a11y-img-alt-good)

[Group examples](./accessibility.md#a11y-img-alt) · [Rule reference](./reference/a11y-img-alt.md)

### `a11y/interactive-supports-focus`

Require interactive role elements to be focusable

<span id="a11y-interactive-supports-focus-bad"></span>

[Bad](./accessibility.md#a11y-interactive-supports-focus-bad)

<span id="a11y-interactive-supports-focus-good"></span>

[Good](./accessibility.md#a11y-interactive-supports-focus-good)

[Group examples](./accessibility.md#a11y-interactive-supports-focus) · [Rule reference](./reference/a11y-interactive-supports-focus.md)

### `a11y/label-has-for`

Require labels to have associated form controls

<span id="a11y-label-has-for-bad"></span>

[Bad](./accessibility.md#a11y-label-has-for-bad)

<span id="a11y-label-has-for-good"></span>

[Good](./accessibility.md#a11y-label-has-for-good)

[Group examples](./accessibility.md#a11y-label-has-for) · [Rule reference](./reference/a11y-label-has-for.md)

### `a11y/landmark-roles`

Validate landmark role placement and uniqueness

<span id="a11y-landmark-roles-bad"></span>

[Bad](./accessibility.md#a11y-landmark-roles-bad)

<span id="a11y-landmark-roles-good"></span>

[Good](./accessibility.md#a11y-landmark-roles-good)

[Group examples](./accessibility.md#a11y-landmark-roles) · [Rule reference](./reference/a11y-landmark-roles.md)

### `a11y/media-has-caption`

Require media elements to have captions

<span id="a11y-media-has-caption-bad"></span>

[Bad](./accessibility.md#a11y-media-has-caption-bad)

<span id="a11y-media-has-caption-good"></span>

[Good](./accessibility.md#a11y-media-has-caption-good)

[Group examples](./accessibility.md#a11y-media-has-caption) · [Rule reference](./reference/a11y-media-has-caption.md)

### `a11y/mouse-events-have-key-events`

Require focus/blur events with mouse events

<span id="a11y-mouse-events-have-key-events-bad"></span>

[Bad](./accessibility.md#a11y-mouse-events-have-key-events-bad)

<span id="a11y-mouse-events-have-key-events-good"></span>

[Good](./accessibility.md#a11y-mouse-events-have-key-events-good)

[Group examples](./accessibility.md#a11y-mouse-events-have-key-events) · [Rule reference](./reference/a11y-mouse-events-have-key-events.md)

### `a11y/no-access-key`

Disallow the use of the accesskey attribute

<span id="a11y-no-access-key-bad"></span>

[Bad](./accessibility.md#a11y-no-access-key-bad)

<span id="a11y-no-access-key-good"></span>

[Good](./accessibility.md#a11y-no-access-key-good)

[Group examples](./accessibility.md#a11y-no-access-key) · [Rule reference](./reference/a11y-no-access-key.md)

### `a11y/no-aria-hidden-on-focusable`

Disallow aria-hidden="true" on focusable elements

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

[Bad](./accessibility.md#a11y-no-aria-hidden-on-focusable-bad)

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

[Good](./accessibility.md#a11y-no-aria-hidden-on-focusable-good)

[Group examples](./accessibility.md#a11y-no-aria-hidden-on-focusable) · [Rule reference](./reference/a11y-no-aria-hidden-on-focusable.md)

### `a11y/no-autofocus`

Disallow the use of the autofocus attribute

<span id="a11y-no-autofocus-bad"></span>

[Bad](./accessibility.md#a11y-no-autofocus-bad)

<span id="a11y-no-autofocus-good"></span>

[Good](./accessibility.md#a11y-no-autofocus-good)

[Group examples](./accessibility.md#a11y-no-autofocus) · [Rule reference](./reference/a11y-no-autofocus.md)

### `a11y/no-distracting-elements`

Disallow distracting elements like &lt;marquee&gt; and &lt;blink&gt;

<span id="a11y-no-distracting-elements-bad"></span>

[Bad](./accessibility.md#a11y-no-distracting-elements-bad)

<span id="a11y-no-distracting-elements-good"></span>

[Good](./accessibility.md#a11y-no-distracting-elements-good)

[Group examples](./accessibility.md#a11y-no-distracting-elements) · [Rule reference](./reference/a11y-no-distracting-elements.md)

### `a11y/no-i-for-icon`

Disallow using &lt;i&gt; element for icons

<span id="a11y-no-i-for-icon-bad"></span>

[Bad](./accessibility.md#a11y-no-i-for-icon-bad)

<span id="a11y-no-i-for-icon-good"></span>

[Good](./accessibility.md#a11y-no-i-for-icon-good)

[Group examples](./accessibility.md#a11y-no-i-for-icon) · [Rule reference](./reference/a11y-no-i-for-icon.md)

### `a11y/no-redundant-roles`

Disallow redundant ARIA roles

<span id="a11y-no-redundant-roles-bad"></span>

[Bad](./accessibility.md#a11y-no-redundant-roles-bad)

<span id="a11y-no-redundant-roles-good"></span>

[Good](./accessibility.md#a11y-no-redundant-roles-good)

[Group examples](./accessibility.md#a11y-no-redundant-roles) · [Rule reference](./reference/a11y-no-redundant-roles.md)

### `a11y/no-refer-to-non-existent-id`

Disallow references to non-existent IDs

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

[Bad](./accessibility.md#a11y-no-refer-to-non-existent-id-bad)

<span id="a11y-no-refer-to-non-existent-id-good"></span>

[Good](./accessibility.md#a11y-no-refer-to-non-existent-id-good)

[Group examples](./accessibility.md#a11y-no-refer-to-non-existent-id) · [Rule reference](./reference/a11y-no-refer-to-non-existent-id.md)

### `a11y/no-role-presentation-on-focusable`

Disallow role="presentation" or role="none" on focusable elements

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

[Bad](./accessibility.md#a11y-no-role-presentation-on-focusable-bad)

<span id="a11y-no-role-presentation-on-focusable-good"></span>

[Good](./accessibility.md#a11y-no-role-presentation-on-focusable-good)

[Group examples](./accessibility.md#a11y-no-role-presentation-on-focusable) · [Rule reference](./reference/a11y-no-role-presentation-on-focusable.md)

### `a11y/no-static-element-interactions`

Disallow event handlers on static elements

<span id="a11y-no-static-element-interactions-bad"></span>

[Bad](./accessibility.md#a11y-no-static-element-interactions-bad)

<span id="a11y-no-static-element-interactions-good"></span>

[Good](./accessibility.md#a11y-no-static-element-interactions-good)

[Group examples](./accessibility.md#a11y-no-static-element-interactions) · [Rule reference](./reference/a11y-no-static-element-interactions.md)

### `a11y/placeholder-label-option`

Require disabled or hidden on select placeholder option

<span id="a11y-placeholder-label-option-bad"></span>

[Bad](./accessibility.md#a11y-placeholder-label-option-bad)

<span id="a11y-placeholder-label-option-good"></span>

[Good](./accessibility.md#a11y-placeholder-label-option-good)

[Group examples](./accessibility.md#a11y-placeholder-label-option) · [Rule reference](./reference/a11y-placeholder-label-option.md)

### `a11y/role-has-required-aria-props`

Require ARIA roles to have required properties

<span id="a11y-role-has-required-aria-props-bad"></span>

[Bad](./accessibility.md#a11y-role-has-required-aria-props-bad)

<span id="a11y-role-has-required-aria-props-good"></span>

[Good](./accessibility.md#a11y-role-has-required-aria-props-good)

[Group examples](./accessibility.md#a11y-role-has-required-aria-props) · [Rule reference](./reference/a11y-role-has-required-aria-props.md)

### `a11y/tabindex-no-positive`

Disallow positive tabindex values

<span id="a11y-tabindex-no-positive-bad"></span>

[Bad](./accessibility.md#a11y-tabindex-no-positive-bad)

<span id="a11y-tabindex-no-positive-good"></span>

[Good](./accessibility.md#a11y-tabindex-no-positive-good)

[Group examples](./accessibility.md#a11y-tabindex-no-positive) · [Rule reference](./reference/a11y-tabindex-no-positive.md)

### `a11y/use-list`

Suggest using list elements for bullet-like text

<span id="a11y-use-list-bad"></span>

[Bad](./accessibility.md#a11y-use-list-bad)

<span id="a11y-use-list-good"></span>

[Good](./accessibility.md#a11y-use-list-good)

[Group examples](./accessibility.md#a11y-use-list) · [Rule reference](./reference/a11y-use-list.md)

### `vue/use-unique-element-ids`

Enforce unique element IDs using useId() instead of static literals

<span id="vue-use-unique-element-ids-bad"></span>

[Bad](./vue.md#vue-use-unique-element-ids-bad)

<span id="vue-use-unique-element-ids-good"></span>

[Good](./vue.md#vue-use-unique-element-ids-good)

[Group examples](./vue.md#vue-use-unique-element-ids) · [Rule reference](./reference/vue-use-unique-element-ids.md)

### `html/deprecated-attr`

Disallow deprecated HTML attributes

<span id="html-deprecated-attr-bad"></span>

[Bad](./html.md#html-deprecated-attr-bad)

<span id="html-deprecated-attr-good"></span>

[Good](./html.md#html-deprecated-attr-good)

[Group examples](./html.md#html-deprecated-attr) · [Rule reference](./reference/html-deprecated-attr.md)

### `html/deprecated-element`

Disallow deprecated HTML elements

<span id="html-deprecated-element-bad"></span>

[Bad](./html.md#html-deprecated-element-bad)

<span id="html-deprecated-element-good"></span>

[Good](./html.md#html-deprecated-element-good)

[Group examples](./html.md#html-deprecated-element) · [Rule reference](./reference/html-deprecated-element.md)

### `html/id-duplication`

Disallow duplicate element IDs

<span id="html-id-duplication-bad"></span>

[Bad](./html.md#html-id-duplication-bad)

<span id="html-id-duplication-good"></span>

[Good](./html.md#html-id-duplication-good)

[Group examples](./html.md#html-id-duplication) · [Rule reference](./reference/html-id-duplication.md)

### `html/no-consecutive-br`

Disallow consecutive &lt;br&gt; elements

<span id="html-no-consecutive-br-bad"></span>

[Bad](./html.md#html-no-consecutive-br-bad)

<span id="html-no-consecutive-br-good"></span>

[Good](./html.md#html-no-consecutive-br-good)

[Group examples](./html.md#html-no-consecutive-br) · [Rule reference](./reference/html-no-consecutive-br.md)

### `html/no-dupe-style-properties`

Disallow duplicate properties in inline style attributes

<span id="html-no-dupe-style-properties-bad"></span>

[Bad](./html.md#html-no-dupe-style-properties-bad)

<span id="html-no-dupe-style-properties-good"></span>

[Good](./html.md#html-no-dupe-style-properties-good)

[Group examples](./html.md#html-no-dupe-style-properties) · [Rule reference](./reference/html-no-dupe-style-properties.md)

### `html/no-duplicate-class`

Disallow duplicate class names in a static class attribute

<span id="html-no-duplicate-class-bad"></span>

[Bad](./html.md#html-no-duplicate-class-bad)

<span id="html-no-duplicate-class-good"></span>

[Good](./html.md#html-no-duplicate-class-good)

[Group examples](./html.md#html-no-duplicate-class) · [Rule reference](./reference/html-no-duplicate-class.md)

### `html/no-duplicate-dt`

Disallow duplicate &lt;dt&gt; names in &lt;dl&gt;

<span id="html-no-duplicate-dt-bad"></span>

[Bad](./html.md#html-no-duplicate-dt-bad)

<span id="html-no-duplicate-dt-good"></span>

[Good](./html.md#html-no-duplicate-dt-good)

[Group examples](./html.md#html-no-duplicate-dt) · [Rule reference](./reference/html-no-duplicate-dt.md)

### `html/no-empty-palpable-content`

Disallow empty elements that expect visible content

<span id="html-no-empty-palpable-content-bad"></span>

[Bad](./html.md#html-no-empty-palpable-content-bad)

<span id="html-no-empty-palpable-content-good"></span>

[Good](./html.md#html-no-empty-palpable-content-good)

[Group examples](./html.md#html-no-empty-palpable-content) · [Rule reference](./reference/html-no-empty-palpable-content.md)

### `html/require-datetime`

Require datetime attribute on &lt;time&gt; element

<span id="html-require-datetime-bad"></span>

[Bad](./html.md#html-require-datetime-bad)

<span id="html-require-datetime-good"></span>

[Good](./html.md#html-require-datetime-good)

[Group examples](./html.md#html-require-datetime) · [Rule reference](./reference/html-require-datetime.md)

### `type/no-floating-promises`

Disallow floating (unhandled) Promises

<span id="type-no-floating-promises-bad"></span>

[Bad](./type-and-script.md#type-no-floating-promises-bad)

<span id="type-no-floating-promises-good"></span>

[Good](./type-and-script.md#type-no-floating-promises-good)

[Group examples](./type-and-script.md#type-no-floating-promises) · [Rule reference](./reference/type-no-floating-promises.md)

### `type/no-reactivity-loss`

Disallow plain snapshots of reactive values across assignments and calls

<span id="type-no-reactivity-loss-bad"></span>

[Bad](./type-and-script.md#type-no-reactivity-loss-bad)

<span id="type-no-reactivity-loss-good"></span>

[Good](./type-and-script.md#type-no-reactivity-loss-good)

[Group examples](./type-and-script.md#type-no-reactivity-loss) · [Rule reference](./reference/type-no-reactivity-loss.md)

### `type/no-unsafe-template-binding`

Disallow template bindings that resolve to unsafe types

<span id="type-no-unsafe-template-binding-bad"></span>

[Bad](./type-and-script.md#type-no-unsafe-template-binding-bad)

<span id="type-no-unsafe-template-binding-good"></span>

[Good](./type-and-script.md#type-no-unsafe-template-binding-good)

[Group examples](./type-and-script.md#type-no-unsafe-template-binding) · [Rule reference](./reference/type-no-unsafe-template-binding.md)

### `type/require-typed-emits`

Require type definition for defineEmits

<span id="type-require-typed-emits-bad"></span>

[Bad](./type-and-script.md#type-require-typed-emits-bad)

<span id="type-require-typed-emits-good"></span>

[Good](./type-and-script.md#type-require-typed-emits-good)

[Group examples](./type-and-script.md#type-require-typed-emits) · [Rule reference](./reference/type-require-typed-emits.md)

### `type/require-typed-props`

Require type definition for defineProps

<span id="type-require-typed-props-bad"></span>

[Bad](./type-and-script.md#type-require-typed-props-bad)

<span id="type-require-typed-props-good"></span>

[Good](./type-and-script.md#type-require-typed-props-good)

[Group examples](./type-and-script.md#type-require-typed-props) · [Rule reference](./reference/type-require-typed-props.md)

### `type/strict-boolean-expressions`

Require safe boolean expressions in script and template conditions

<span id="type-strict-boolean-expressions-bad"></span>

[Bad](./type-and-script.md#type-strict-boolean-expressions-bad)

<span id="type-strict-boolean-expressions-good"></span>

[Good](./type-and-script.md#type-strict-boolean-expressions-good)

[Group examples](./type-and-script.md#type-strict-boolean-expressions) · [Rule reference](./reference/type-strict-boolean-expressions.md)

### `script/no-get-current-instance`

Disallow getCurrentInstance() in Vapor mode (returns null)

<span id="script-no-get-current-instance-bad"></span>

[Bad](./type-and-script.md#script-no-get-current-instance-bad)

<span id="script-no-get-current-instance-good"></span>

[Good](./type-and-script.md#script-no-get-current-instance-good)

[Group examples](./type-and-script.md#script-no-get-current-instance) · [Rule reference](./reference/script-no-get-current-instance.md)

### `script/no-next-tick`

Disallow nextTick() usage in Vapor-oriented components

<span id="script-no-next-tick-bad"></span>

[Bad](./type-and-script.md#script-no-next-tick-bad)

<span id="script-no-next-tick-good"></span>

[Good](./type-and-script.md#script-no-next-tick-good)

[Group examples](./type-and-script.md#script-no-next-tick) · [Rule reference](./reference/script-no-next-tick.md)

### `script/no-options-api`

Disallow Options API patterns in Vapor mode

<span id="script-no-options-api-bad"></span>

[Bad](./type-and-script.md#script-no-options-api-bad)

<span id="script-no-options-api-good"></span>

[Good](./type-and-script.md#script-no-options-api-good)

[Group examples](./type-and-script.md#script-no-options-api) · [Rule reference](./reference/script-no-options-api.md)

### `vapor/no-inline-template`

Disallow deprecated inline-template attribute

<span id="vapor-no-inline-template-bad"></span>

[Bad](./vapor.md#vapor-no-inline-template-bad)

<span id="vapor-no-inline-template-good"></span>

[Good](./vapor.md#vapor-no-inline-template-good)

[Group examples](./vapor.md#vapor-no-inline-template) · [Rule reference](./reference/vapor-no-inline-template.md)

### `vapor/no-vue-lifecycle-events`

Disallow @vue:xxx per-element lifecycle events (not supported in Vapor)

<span id="vapor-no-vue-lifecycle-events-bad"></span>

[Bad](./vapor.md#vapor-no-vue-lifecycle-events-bad)

<span id="vapor-no-vue-lifecycle-events-good"></span>

[Good](./vapor.md#vapor-no-vue-lifecycle-events-good)

[Group examples](./vapor.md#vapor-no-vue-lifecycle-events) · [Rule reference](./reference/vapor-no-vue-lifecycle-events.md)

### `vapor/prefer-static-class`

Prefer static class over dynamic class binding for string literals

<span id="vapor-prefer-static-class-bad"></span>

[Bad](./vapor.md#vapor-prefer-static-class-bad)

<span id="vapor-prefer-static-class-good"></span>

[Good](./vapor.md#vapor-prefer-static-class-good)

[Group examples](./vapor.md#vapor-prefer-static-class) · [Rule reference](./reference/vapor-prefer-static-class.md)

### `vapor/require-vapor-attribute`

Suggest adding vapor attribute to script setup

<span id="vapor-require-vapor-attribute-bad"></span>

[Bad](./vapor.md#vapor-require-vapor-attribute-bad)

<span id="vapor-require-vapor-attribute-good"></span>

[Good](./vapor.md#vapor-require-vapor-attribute-good)

[Group examples](./vapor.md#vapor-require-vapor-attribute) · [Rule reference](./reference/vapor-require-vapor-attribute.md)

### `ecosystem/nuxt-prefer-nuxt-link`

Prefer NuxtLink for internal application links

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

[Bad](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link-bad)

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

[Good](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link-good)

[Group examples](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link) · [Rule reference](./reference/ecosystem-nuxt-prefer-nuxt-link.md)

### `ecosystem/pinia-prefer-store-to-refs`

Prefer storeToRefs() when destructuring Pinia stores

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

[Bad](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs-bad)

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

[Good](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs-good)

[Group examples](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs) · [Rule reference](./reference/ecosystem-pinia-prefer-store-to-refs.md)

### `ecosystem/router-link-require-to`

Require a `to` target on RouterLink and NuxtLink components

<span id="ecosystem-router-link-require-to-bad"></span>

[Bad](./ecosystem.md#ecosystem-router-link-require-to-bad)

<span id="ecosystem-router-link-require-to-good"></span>

[Good](./ecosystem.md#ecosystem-router-link-require-to-good)

[Group examples](./ecosystem.md#ecosystem-router-link-require-to) · [Rule reference](./reference/ecosystem-router-link-require-to.md)

### `ecosystem/void-link-require-href`

Require `href` on Void Vue Link components

<span id="ecosystem-void-link-require-href-bad"></span>

[Bad](./ecosystem.md#ecosystem-void-link-require-href-bad)

<span id="ecosystem-void-link-require-href-good"></span>

[Good](./ecosystem.md#ecosystem-void-link-require-href-good)

[Group examples](./ecosystem.md#ecosystem-void-link-require-href) · [Rule reference](./reference/ecosystem-void-link-require-href.md)

### `ecosystem/void-link-valid-method`

Validate static Void Vue Link method props

<span id="ecosystem-void-link-valid-method-bad"></span>

[Bad](./ecosystem.md#ecosystem-void-link-valid-method-bad)

<span id="ecosystem-void-link-valid-method-good"></span>

[Good](./ecosystem.md#ecosystem-void-link-valid-method-good)

[Group examples](./ecosystem.md#ecosystem-void-link-valid-method) · [Rule reference](./reference/ecosystem-void-link-valid-method.md)

### `ecosystem/vue-i18n-no-missing-key`

Report static vue-i18n keys that are absent from local SFC messages

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

[Bad](./ecosystem.md#ecosystem-vue-i18n-no-missing-key-bad)

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

[Good](./ecosystem.md#ecosystem-vue-i18n-no-missing-key-good)

[Group examples](./ecosystem.md#ecosystem-vue-i18n-no-missing-key) · [Rule reference](./reference/ecosystem-vue-i18n-no-missing-key.md)

### `ecosystem/vue-router-prefer-named-link`

Prefer named route objects over static path strings in RouterLink

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

[Bad](./ecosystem.md#ecosystem-vue-router-prefer-named-link-bad)

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

[Good](./ecosystem.md#ecosystem-vue-router-prefer-named-link-good)

[Group examples](./ecosystem.md#ecosystem-vue-router-prefer-named-link) · [Rule reference](./reference/ecosystem-vue-router-prefer-named-link.md)

### `ecosystem/vue-router-prefer-named-push`

Prefer named route objects for Vue Router programmatic navigation

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

[Bad](./ecosystem.md#ecosystem-vue-router-prefer-named-push-bad)

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

[Good](./ecosystem.md#ecosystem-vue-router-prefer-named-push-good)

[Group examples](./ecosystem.md#ecosystem-vue-router-prefer-named-push) · [Rule reference](./reference/ecosystem-vue-router-prefer-named-push.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

Avoid snapshotting wrapper.html() in Vue Test Utils tests

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

[Bad](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot-bad)

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

[Good](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot-good)

[Group examples](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot) · [Rule reference](./reference/ecosystem-vue-test-utils-no-html-snapshot.md)

### `css/no-display-none`

Suggest using v-show instead of display: none

<span id="css-no-display-none-bad"></span>

[Bad](./musea-and-css.md#css-no-display-none-bad)

<span id="css-no-display-none-good"></span>

[Good](./musea-and-css.md#css-no-display-none-good)

[Group examples](./musea-and-css.md#css-no-display-none) · [Rule reference](./reference/css-no-display-none.md)

### `css/no-hardcoded-values`

Suggest using CSS variables instead of hardcoded values

<span id="css-no-hardcoded-values-bad"></span>

[Bad](./musea-and-css.md#css-no-hardcoded-values-bad)

<span id="css-no-hardcoded-values-good"></span>

[Good](./musea-and-css.md#css-no-hardcoded-values-good)

[Group examples](./musea-and-css.md#css-no-hardcoded-values) · [Rule reference](./reference/css-no-hardcoded-values.md)

### `css/no-id-selectors`

Discourage use of ID selectors in CSS

<span id="css-no-id-selectors-bad"></span>

[Bad](./musea-and-css.md#css-no-id-selectors-bad)

<span id="css-no-id-selectors-good"></span>

[Good](./musea-and-css.md#css-no-id-selectors-good)

[Group examples](./musea-and-css.md#css-no-id-selectors) · [Rule reference](./reference/css-no-id-selectors.md)

### `css/no-important`

Discourage use of !important in CSS

<span id="css-no-important-bad"></span>

[Bad](./musea-and-css.md#css-no-important-bad)

<span id="css-no-important-good"></span>

[Good](./musea-and-css.md#css-no-important-good)

[Group examples](./musea-and-css.md#css-no-important) · [Rule reference](./reference/css-no-important.md)

### `css/no-utility-classes`

Warn against implementing utility classes in component styles

<span id="css-no-utility-classes-bad"></span>

[Bad](./musea-and-css.md#css-no-utility-classes-bad)

<span id="css-no-utility-classes-good"></span>

[Good](./musea-and-css.md#css-no-utility-classes-good)

[Group examples](./musea-and-css.md#css-no-utility-classes) · [Rule reference](./reference/css-no-utility-classes.md)

### `css/no-v-bind-performance`

Warn about performance cost of CSS v-bind()

<span id="css-no-v-bind-performance-bad"></span>

[Bad](./musea-and-css.md#css-no-v-bind-performance-bad)

<span id="css-no-v-bind-performance-good"></span>

[Good](./musea-and-css.md#css-no-v-bind-performance-good)

[Group examples](./musea-and-css.md#css-no-v-bind-performance) · [Rule reference](./reference/css-no-v-bind-performance.md)

### `css/prefer-logical-properties`

Recommend CSS logical properties for better i18n support

<span id="css-prefer-logical-properties-bad"></span>

[Bad](./musea-and-css.md#css-prefer-logical-properties-bad)

<span id="css-prefer-logical-properties-good"></span>

[Good](./musea-and-css.md#css-prefer-logical-properties-good)

[Group examples](./musea-and-css.md#css-prefer-logical-properties) · [Rule reference](./reference/css-prefer-logical-properties.md)

### `css/prefer-nested-selectors`

Recommend using CSS nesting for descendant selectors

<span id="css-prefer-nested-selectors-bad"></span>

[Bad](./musea-and-css.md#css-prefer-nested-selectors-bad)

<span id="css-prefer-nested-selectors-good"></span>

[Good](./musea-and-css.md#css-prefer-nested-selectors-good)

[Group examples](./musea-and-css.md#css-prefer-nested-selectors) · [Rule reference](./reference/css-prefer-nested-selectors.md)

### `css/prefer-slotted`

Recommend ::v-slotted() for styling slot content

<span id="css-prefer-slotted-bad"></span>

[Bad](./musea-and-css.md#css-prefer-slotted-bad)

<span id="css-prefer-slotted-good"></span>

[Good](./musea-and-css.md#css-prefer-slotted-good)

[Group examples](./musea-and-css.md#css-prefer-slotted) · [Rule reference](./reference/css-prefer-slotted.md)

### `css/require-font-display`

Require font-display in @font-face rules

<span id="css-require-font-display-bad"></span>

[Bad](./musea-and-css.md#css-require-font-display-bad)

<span id="css-require-font-display-good"></span>

[Good](./musea-and-css.md#css-require-font-display-good)

[Group examples](./musea-and-css.md#css-require-font-display) · [Rule reference](./reference/css-require-font-display.md)

### `musea/no-empty-variant`

Disallow empty &lt;variant&gt; blocks

<span id="musea-no-empty-variant-bad"></span>

[Bad](./musea-and-css.md#musea-no-empty-variant-bad)

<span id="musea-no-empty-variant-good"></span>

[Good](./musea-and-css.md#musea-no-empty-variant-good)

[Group examples](./musea-and-css.md#musea-no-empty-variant) · [Rule reference](./reference/musea-no-empty-variant.md)

### `musea/prefer-design-tokens`

Prefer design token CSS variables over hardcoded primitive values

<span id="musea-prefer-design-tokens-bad"></span>

[Bad](./musea-and-css.md#musea-prefer-design-tokens-bad)

<span id="musea-prefer-design-tokens-good"></span>

[Good](./musea-and-css.md#musea-prefer-design-tokens-good)

[Group examples](./musea-and-css.md#musea-prefer-design-tokens) · [Rule reference](./reference/musea-prefer-design-tokens.md)

### `musea/require-component`

Require component attribute in &lt;art&gt; block

<span id="musea-require-component-bad"></span>

[Bad](./musea-and-css.md#musea-require-component-bad)

<span id="musea-require-component-good"></span>

[Good](./musea-and-css.md#musea-require-component-good)

[Group examples](./musea-and-css.md#musea-require-component) · [Rule reference](./reference/musea-require-component.md)

### `musea/require-title`

Require title attribute in &lt;art&gt; block

<span id="musea-require-title-bad"></span>

[Bad](./musea-and-css.md#musea-require-title-bad)

<span id="musea-require-title-good"></span>

[Good](./musea-and-css.md#musea-require-title-good)

[Group examples](./musea-and-css.md#musea-require-title) · [Rule reference](./reference/musea-require-title.md)

### `musea/unique-variant-names`

Require unique variant names

<span id="musea-unique-variant-names-bad"></span>

[Bad](./musea-and-css.md#musea-unique-variant-names-bad)

<span id="musea-unique-variant-names-good"></span>

[Good](./musea-and-css.md#musea-unique-variant-names-good)

[Group examples](./musea-and-css.md#musea-unique-variant-names) · [Rule reference](./reference/musea-unique-variant-names.md)

### `musea/valid-variant`

Require name attribute in &lt;variant&gt; blocks

<span id="musea-valid-variant-bad"></span>

[Bad](./musea-and-css.md#musea-valid-variant-bad)

<span id="musea-valid-variant-good"></span>

[Good](./musea-and-css.md#musea-valid-variant-good)

[Group examples](./musea-and-css.md#musea-valid-variant) · [Rule reference](./reference/musea-valid-variant.md)

### `script/component-options-name-casing`

Enforce PascalCase for the component `name` option

<span id="script-component-options-name-casing-bad"></span>

[Bad](./type-and-script.md#script-component-options-name-casing-bad)

<span id="script-component-options-name-casing-good"></span>

[Good](./type-and-script.md#script-component-options-name-casing-good)

[Group examples](./type-and-script.md#script-component-options-name-casing) · [Rule reference](./reference/script-component-options-name-casing.md)

### `script/custom-event-name-casing`

Enforce camelCase for emitted custom event names

<span id="script-custom-event-name-casing-bad"></span>

[Bad](./type-and-script.md#script-custom-event-name-casing-bad)

<span id="script-custom-event-name-casing-good"></span>

[Good](./type-and-script.md#script-custom-event-name-casing-good)

[Group examples](./type-and-script.md#script-custom-event-name-casing) · [Rule reference](./reference/script-custom-event-name-casing.md)

### `script/define-emits-declaration`

Enforce the type-based defineEmits&lt;{}&gt;() form over the runtime/array form

<span id="script-define-emits-declaration-bad"></span>

[Bad](./type-and-script.md#script-define-emits-declaration-bad)

<span id="script-define-emits-declaration-good"></span>

[Good](./type-and-script.md#script-define-emits-declaration-good)

[Group examples](./type-and-script.md#script-define-emits-declaration) · [Rule reference](./reference/script-define-emits-declaration.md)

### `script/define-macros-order`

Enforce a consistent order of the Vue compiler macros in &lt;script setup&gt;

<span id="script-define-macros-order-bad"></span>

[Bad](./type-and-script.md#script-define-macros-order-bad)

<span id="script-define-macros-order-good"></span>

[Good](./type-and-script.md#script-define-macros-order-good)

[Group examples](./type-and-script.md#script-define-macros-order) · [Rule reference](./reference/script-define-macros-order.md)

### `script/define-props-declaration`

Enforce type-based defineProps&lt;{ ... }&gt;() over the runtime/object form

<span id="script-define-props-declaration-bad"></span>

[Bad](./type-and-script.md#script-define-props-declaration-bad)

<span id="script-define-props-declaration-good"></span>

[Good](./type-and-script.md#script-define-props-declaration-good)

[Group examples](./type-and-script.md#script-define-props-declaration) · [Rule reference](./reference/script-define-props-declaration.md)

### `script/define-props-destructuring`

Enforce consistent style for defineProps destructuring in &lt;script setup&gt;

<span id="script-define-props-destructuring-bad"></span>

[Bad](./type-and-script.md#script-define-props-destructuring-bad)

<span id="script-define-props-destructuring-good"></span>

[Good](./type-and-script.md#script-define-props-destructuring-good)

[Group examples](./type-and-script.md#script-define-props-destructuring) · [Rule reference](./reference/script-define-props-destructuring.md)

### `script/no-arrow-functions-in-watch`

Disallow arrow functions as Options API watch handlers

<span id="script-no-arrow-functions-in-watch-bad"></span>

[Bad](./type-and-script.md#script-no-arrow-functions-in-watch-bad)

<span id="script-no-arrow-functions-in-watch-good"></span>

[Good](./type-and-script.md#script-no-arrow-functions-in-watch-good)

[Group examples](./type-and-script.md#script-no-arrow-functions-in-watch) · [Rule reference](./reference/script-no-arrow-functions-in-watch.md)

### `script/no-async-in-computed`

Disallow async functions in computed properties

<span id="script-no-async-in-computed-bad"></span>

[Bad](./type-and-script.md#script-no-async-in-computed-bad)

<span id="script-no-async-in-computed-good"></span>

[Good](./type-and-script.md#script-no-async-in-computed-good)

[Group examples](./type-and-script.md#script-no-async-in-computed) · [Rule reference](./reference/script-no-async-in-computed.md)

### `script/no-boolean-default`

Disallow a default on a Boolean prop

<span id="script-no-boolean-default-bad"></span>

[Bad](./type-and-script.md#script-no-boolean-default-bad)

<span id="script-no-boolean-default-good"></span>

[Good](./type-and-script.md#script-no-boolean-default-good)

[Group examples](./type-and-script.md#script-no-boolean-default) · [Rule reference](./reference/script-no-boolean-default.md)

### `script/no-deep-destructure-in-props`

Disallow deeply nested destructuring in defineProps

<span id="script-no-deep-destructure-in-props-bad"></span>

[Bad](./type-and-script.md#script-no-deep-destructure-in-props-bad)

<span id="script-no-deep-destructure-in-props-good"></span>

[Good](./type-and-script.md#script-no-deep-destructure-in-props-good)

[Group examples](./type-and-script.md#script-no-deep-destructure-in-props) · [Rule reference](./reference/script-no-deep-destructure-in-props.md)

### `script/no-deprecated-data-object-declaration`

Disallow an object literal as the component data option (Vue 3 requires a function)

<span id="script-no-deprecated-data-object-declaration-bad"></span>

[Bad](./type-and-script.md#script-no-deprecated-data-object-declaration-bad)

<span id="script-no-deprecated-data-object-declaration-good"></span>

[Good](./type-and-script.md#script-no-deprecated-data-object-declaration-good)

[Group examples](./type-and-script.md#script-no-deprecated-data-object-declaration) · [Rule reference](./reference/script-no-deprecated-data-object-declaration.md)

### `script/no-deprecated-destroyed-lifecycle`

Disallow deprecated destroyed and beforeDestroy lifecycle hooks

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

[Bad](./type-and-script.md#script-no-deprecated-destroyed-lifecycle-bad)

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

[Good](./type-and-script.md#script-no-deprecated-destroyed-lifecycle-good)

[Group examples](./type-and-script.md#script-no-deprecated-destroyed-lifecycle) · [Rule reference](./reference/script-no-deprecated-destroyed-lifecycle.md)

### `script/no-deprecated-dollar-listeners-api`

Disallow the $listeners instance property removed in Vue 3 (merged into $attrs)

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

[Bad](./type-and-script.md#script-no-deprecated-dollar-listeners-api-bad)

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

[Good](./type-and-script.md#script-no-deprecated-dollar-listeners-api-good)

[Group examples](./type-and-script.md#script-no-deprecated-dollar-listeners-api) · [Rule reference](./reference/script-no-deprecated-dollar-listeners-api.md)

### `script/no-deprecated-dollar-scopedslots-api`

Disallow the $scopedSlots instance property removed in Vue 3 (use $slots)

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

[Bad](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api-bad)

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

[Good](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api-good)

[Group examples](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api) · [Rule reference](./reference/script-no-deprecated-dollar-scopedslots-api.md)

### `script/no-deprecated-events-api`

Disallow the removed Vue 2 events API ($on / $off / $once)

<span id="script-no-deprecated-events-api-bad"></span>

[Bad](./type-and-script.md#script-no-deprecated-events-api-bad)

<span id="script-no-deprecated-events-api-good"></span>

[Good](./type-and-script.md#script-no-deprecated-events-api-good)

[Group examples](./type-and-script.md#script-no-deprecated-events-api) · [Rule reference](./reference/script-no-deprecated-events-api.md)

### `script/no-deprecated-props-default-this`

Disallow `this` inside a prop default/validator function (removed in Vue 3)

<span id="script-no-deprecated-props-default-this-bad"></span>

[Bad](./type-and-script.md#script-no-deprecated-props-default-this-bad)

<span id="script-no-deprecated-props-default-this-good"></span>

[Good](./type-and-script.md#script-no-deprecated-props-default-this-good)

[Group examples](./type-and-script.md#script-no-deprecated-props-default-this) · [Rule reference](./reference/script-no-deprecated-props-default-this.md)

### `script/no-dupe-keys`

Disallow duplicate keys across Options API props/data/computed/methods/setup/inject

<span id="script-no-dupe-keys-bad"></span>

[Bad](./type-and-script.md#script-no-dupe-keys-bad)

<span id="script-no-dupe-keys-good"></span>

[Good](./type-and-script.md#script-no-dupe-keys-good)

[Group examples](./type-and-script.md#script-no-dupe-keys) · [Rule reference](./reference/script-no-dupe-keys.md)

### `script/no-duplicate-attr-inheritance`

Flag a component that applies its fallthrough attributes twice

<span id="script-no-duplicate-attr-inheritance-bad"></span>

[Bad](./type-and-script.md#script-no-duplicate-attr-inheritance-bad)

<span id="script-no-duplicate-attr-inheritance-good"></span>

[Good](./type-and-script.md#script-no-duplicate-attr-inheritance-good)

[Group examples](./type-and-script.md#script-no-duplicate-attr-inheritance) · [Rule reference](./reference/script-no-duplicate-attr-inheritance.md)

### `script/no-export-in-script-setup`

Disallow export statements inside &lt;script setup&gt;

<span id="script-no-export-in-script-setup-bad"></span>

[Bad](./type-and-script.md#script-no-export-in-script-setup-bad)

<span id="script-no-export-in-script-setup-good"></span>

[Good](./type-and-script.md#script-no-export-in-script-setup-good)

[Group examples](./type-and-script.md#script-no-export-in-script-setup) · [Rule reference](./reference/script-no-export-in-script-setup.md)

### `script/no-import-compiler-macros`

Disallow importing Vue compiler macros that are auto-imported

<span id="script-no-import-compiler-macros-bad"></span>

[Bad](./type-and-script.md#script-no-import-compiler-macros-bad)

<span id="script-no-import-compiler-macros-good"></span>

[Good](./type-and-script.md#script-no-import-compiler-macros-good)

[Group examples](./type-and-script.md#script-no-import-compiler-macros) · [Rule reference](./reference/script-no-import-compiler-macros.md)

### `script/no-internal-imports`

Disallow importing from Vue internal modules

<span id="script-no-internal-imports-bad"></span>

[Bad](./type-and-script.md#script-no-internal-imports-bad)

<span id="script-no-internal-imports-good"></span>

[Good](./type-and-script.md#script-no-internal-imports-good)

[Group examples](./type-and-script.md#script-no-internal-imports) · [Rule reference](./reference/script-no-internal-imports.md)

### `script/no-multiple-slot-args`

Disallow passing more than one argument to a scoped-slot function call

<span id="script-no-multiple-slot-args-bad"></span>

[Bad](./type-and-script.md#script-no-multiple-slot-args-bad)

<span id="script-no-multiple-slot-args-good"></span>

[Good](./type-and-script.md#script-no-multiple-slot-args-good)

[Group examples](./type-and-script.md#script-no-multiple-slot-args) · [Rule reference](./reference/script-no-multiple-slot-args.md)

### `script/no-potential-component-option-typo`

Flag likely typos in Options API component option names

<span id="script-no-potential-component-option-typo-bad"></span>

[Bad](./type-and-script.md#script-no-potential-component-option-typo-bad)

<span id="script-no-potential-component-option-typo-good"></span>

[Good](./type-and-script.md#script-no-potential-component-option-typo-good)

[Group examples](./type-and-script.md#script-no-potential-component-option-typo) · [Rule reference](./reference/script-no-potential-component-option-typo.md)

### `script/no-reactive-destructure`

Disallow destructuring reactive objects which loses reactivity

<span id="script-no-reactive-destructure-bad"></span>

[Bad](./type-and-script.md#script-no-reactive-destructure-bad)

<span id="script-no-reactive-destructure-good"></span>

[Good](./type-and-script.md#script-no-reactive-destructure-good)

[Group examples](./type-and-script.md#script-no-reactive-destructure) · [Rule reference](./reference/script-no-reactive-destructure.md)

### `script/no-ref-as-operand`

Require ref-bound variables to be accessed via `.value` when used as an operand

<span id="script-no-ref-as-operand-bad"></span>

[Bad](./type-and-script.md#script-no-ref-as-operand-bad)

<span id="script-no-ref-as-operand-good"></span>

[Good](./type-and-script.md#script-no-ref-as-operand-good)

[Group examples](./type-and-script.md#script-no-ref-as-operand) · [Rule reference](./reference/script-no-ref-as-operand.md)

### `script/no-required-prop-with-default`

Disallow a prop that is both required: true and has a default

<span id="script-no-required-prop-with-default-bad"></span>

[Bad](./type-and-script.md#script-no-required-prop-with-default-bad)

<span id="script-no-required-prop-with-default-good"></span>

[Good](./type-and-script.md#script-no-required-prop-with-default-good)

[Group examples](./type-and-script.md#script-no-required-prop-with-default) · [Rule reference](./reference/script-no-required-prop-with-default.md)

### `script/no-reserved-identifiers`

Disallow using Vue compiler reserved identifiers

<span id="script-no-reserved-identifiers-bad"></span>

[Bad](./type-and-script.md#script-no-reserved-identifiers-bad)

<span id="script-no-reserved-identifiers-good"></span>

[Good](./type-and-script.md#script-no-reserved-identifiers-good)

[Group examples](./type-and-script.md#script-no-reserved-identifiers) · [Rule reference](./reference/script-no-reserved-identifiers.md)

### `script/no-reserved-keys`

Disallow Vue-reserved names as Options API props/data/computed/methods/setup keys

<span id="script-no-reserved-keys-bad"></span>

[Bad](./type-and-script.md#script-no-reserved-keys-bad)

<span id="script-no-reserved-keys-good"></span>

[Good](./type-and-script.md#script-no-reserved-keys-good)

[Group examples](./type-and-script.md#script-no-reserved-keys) · [Rule reference](./reference/script-no-reserved-keys.md)

### `script/no-reserved-props`

Disallow reserved names in a component's props declaration

<span id="script-no-reserved-props-bad"></span>

[Bad](./type-and-script.md#script-no-reserved-props-bad)

<span id="script-no-reserved-props-good"></span>

[Good](./type-and-script.md#script-no-reserved-props-good)

[Group examples](./type-and-script.md#script-no-reserved-props) · [Rule reference](./reference/script-no-reserved-props.md)

### `script/no-restricted-globals`

Disallow references to runtime-environment globals that must go through a typed wrapper

<span id="script-no-restricted-globals-bad"></span>

[Bad](./type-and-script.md#script-no-restricted-globals-bad)

<span id="script-no-restricted-globals-good"></span>

[Good](./type-and-script.md#script-no-restricted-globals-good)

[Group examples](./type-and-script.md#script-no-restricted-globals) · [Rule reference](./reference/script-no-restricted-globals.md)

### `script/no-restricted-members`

Disallow project-configured object.property member accesses

<span id="script-no-restricted-members-bad"></span>

[Bad](./type-and-script.md#script-no-restricted-members-bad)

<span id="script-no-restricted-members-good"></span>

[Good](./type-and-script.md#script-no-restricted-members-good)

[Group examples](./type-and-script.md#script-no-restricted-members) · [Rule reference](./reference/script-no-restricted-members.md)

### `script/no-side-effects-in-computed-properties`

Disallow side effects in Options API computed getters

<span id="script-no-side-effects-in-computed-properties-bad"></span>

[Bad](./type-and-script.md#script-no-side-effects-in-computed-properties-bad)

<span id="script-no-side-effects-in-computed-properties-good"></span>

[Good](./type-and-script.md#script-no-side-effects-in-computed-properties-good)

[Group examples](./type-and-script.md#script-no-side-effects-in-computed-properties) · [Rule reference](./reference/script-no-side-effects-in-computed-properties.md)

### `script/no-top-level-ref-in-script`

Disallow top-level ref/reactive to prevent Cross-Request State Pollution

<span id="script-no-top-level-ref-in-script-bad"></span>

[Bad](./type-and-script.md#script-no-top-level-ref-in-script-bad)

<span id="script-no-top-level-ref-in-script-good"></span>

[Good](./type-and-script.md#script-no-top-level-ref-in-script-good)

[Group examples](./type-and-script.md#script-no-top-level-ref-in-script) · [Rule reference](./reference/script-no-top-level-ref-in-script.md)

### `script/no-unstable-nested-components`

Disallow component definitions inside setup or render functions

<span id="script-no-unstable-nested-components-bad"></span>

[Bad](./type-and-script.md#script-no-unstable-nested-components-bad)

<span id="script-no-unstable-nested-components-good"></span>

[Good](./type-and-script.md#script-no-unstable-nested-components-good)

[Group examples](./type-and-script.md#script-no-unstable-nested-components) · [Rule reference](./reference/script-no-unstable-nested-components.md)

### `script/no-unused-emit-declarations`

Flag declared events that are never emitted

<span id="script-no-unused-emit-declarations-bad"></span>

[Bad](./type-and-script.md#script-no-unused-emit-declarations-bad)

<span id="script-no-unused-emit-declarations-good"></span>

[Good](./type-and-script.md#script-no-unused-emit-declarations-good)

[Group examples](./type-and-script.md#script-no-unused-emit-declarations) · [Rule reference](./reference/script-no-unused-emit-declarations.md)

### `script/no-use-computed-property-like-method`

Disallow calling an Options API computed property like a method

<span id="script-no-use-computed-property-like-method-bad"></span>

[Bad](./type-and-script.md#script-no-use-computed-property-like-method-bad)

<span id="script-no-use-computed-property-like-method-good"></span>

[Good](./type-and-script.md#script-no-use-computed-property-like-method-good)

[Group examples](./type-and-script.md#script-no-use-computed-property-like-method) · [Rule reference](./reference/script-no-use-computed-property-like-method.md)

### `script/no-with-defaults`

Discourage withDefaults in favor of destructuring defaults (Vue 3.5+)

<span id="script-no-with-defaults-bad"></span>

[Bad](./type-and-script.md#script-no-with-defaults-bad)

<span id="script-no-with-defaults-good"></span>

[Good](./type-and-script.md#script-no-with-defaults-good)

[Group examples](./type-and-script.md#script-no-with-defaults) · [Rule reference](./reference/script-no-with-defaults.md)

### `script/prefer-computed`

Prefer computed() for derived reactive state

<span id="script-prefer-computed-bad"></span>

[Bad](./type-and-script.md#script-prefer-computed-bad)

<span id="script-prefer-computed-good"></span>

[Good](./type-and-script.md#script-prefer-computed-good)

[Group examples](./type-and-script.md#script-prefer-computed) · [Rule reference](./reference/script-prefer-computed.md)

### `script/prefer-define-options`

Prefer defineOptions() over a plain &lt;script&gt; that only sets name/inheritAttrs

<span id="script-prefer-define-options-bad"></span>

[Bad](./type-and-script.md#script-prefer-define-options-bad)

<span id="script-prefer-define-options-good"></span>

[Good](./type-and-script.md#script-prefer-define-options-good)

[Group examples](./type-and-script.md#script-prefer-define-options) · [Rule reference](./reference/script-prefer-define-options.md)

### `script/prefer-import-from-vue`

Prefer importing from 'vue' instead of internal packages

<span id="script-prefer-import-from-vue-bad"></span>

[Bad](./type-and-script.md#script-prefer-import-from-vue-bad)

<span id="script-prefer-import-from-vue-good"></span>

[Good](./type-and-script.md#script-prefer-import-from-vue-good)

[Group examples](./type-and-script.md#script-prefer-import-from-vue) · [Rule reference](./reference/script-prefer-import-from-vue.md)

### `script/prefer-ref-over-reactive`

Recommend using ref() over reactive() for state management

<span id="script-prefer-ref-over-reactive-bad"></span>

[Bad](./type-and-script.md#script-prefer-ref-over-reactive-bad)

<span id="script-prefer-ref-over-reactive-good"></span>

[Good](./type-and-script.md#script-prefer-ref-over-reactive-good)

[Group examples](./type-and-script.md#script-prefer-ref-over-reactive) · [Rule reference](./reference/script-prefer-ref-over-reactive.md)

### `script/prefer-use-attrs`

Recommend using useAttrs() over context.attrs

<span id="script-prefer-use-attrs-bad"></span>

[Bad](./type-and-script.md#script-prefer-use-attrs-bad)

<span id="script-prefer-use-attrs-good"></span>

[Good](./type-and-script.md#script-prefer-use-attrs-good)

[Group examples](./type-and-script.md#script-prefer-use-attrs) · [Rule reference](./reference/script-prefer-use-attrs.md)

### `script/prefer-use-id`

Recommend using useId() for generating unique IDs (Vue 3.5+)

<span id="script-prefer-use-id-bad"></span>

[Bad](./type-and-script.md#script-prefer-use-id-bad)

<span id="script-prefer-use-id-good"></span>

[Good](./type-and-script.md#script-prefer-use-id-good)

[Group examples](./type-and-script.md#script-prefer-use-id) · [Rule reference](./reference/script-prefer-use-id.md)

### `script/prefer-use-slots`

Recommend using useSlots() over context.slots

<span id="script-prefer-use-slots-bad"></span>

[Bad](./type-and-script.md#script-prefer-use-slots-bad)

<span id="script-prefer-use-slots-good"></span>

[Good](./type-and-script.md#script-prefer-use-slots-good)

[Group examples](./type-and-script.md#script-prefer-use-slots) · [Rule reference](./reference/script-prefer-use-slots.md)

### `script/prefer-use-template-ref`

Recommend useTemplateRef over ref for template references (Vue 3.5+)

<span id="script-prefer-use-template-ref-bad"></span>

[Bad](./type-and-script.md#script-prefer-use-template-ref-bad)

<span id="script-prefer-use-template-ref-good"></span>

[Good](./type-and-script.md#script-prefer-use-template-ref-good)

[Group examples](./type-and-script.md#script-prefer-use-template-ref) · [Rule reference](./reference/script-prefer-use-template-ref.md)

### `script/require-default-prop`

Require a default value for every optional, non-Boolean prop

<span id="script-require-default-prop-bad"></span>

[Bad](./type-and-script.md#script-require-default-prop-bad)

<span id="script-require-default-prop-good"></span>

[Good](./type-and-script.md#script-require-default-prop-good)

[Group examples](./type-and-script.md#script-require-default-prop) · [Rule reference](./reference/script-require-default-prop.md)

### `script/require-explicit-emits`

Require emitted events to be declared in defineEmits or the emits option

<span id="script-require-explicit-emits-bad"></span>

[Bad](./type-and-script.md#script-require-explicit-emits-bad)

<span id="script-require-explicit-emits-good"></span>

[Good](./type-and-script.md#script-require-explicit-emits-good)

[Group examples](./type-and-script.md#script-require-explicit-emits) · [Rule reference](./reference/script-require-explicit-emits.md)

### `script/require-explicit-slots`

Require slots consumed via useSlots() to be explicitly typed with defineSlots&lt;...&gt;()

<span id="script-require-explicit-slots-bad"></span>

[Bad](./type-and-script.md#script-require-explicit-slots-bad)

<span id="script-require-explicit-slots-good"></span>

[Good](./type-and-script.md#script-require-explicit-slots-good)

[Group examples](./type-and-script.md#script-require-explicit-slots) · [Rule reference](./reference/script-require-explicit-slots.md)

### `script/require-function-return-type`

Require return type annotations on functions

<span id="script-require-function-return-type-bad"></span>

[Bad](./type-and-script.md#script-require-function-return-type-bad)

<span id="script-require-function-return-type-good"></span>

[Good](./type-and-script.md#script-require-function-return-type-good)

[Group examples](./type-and-script.md#script-require-function-return-type) · [Rule reference](./reference/script-require-function-return-type.md)

### `script/require-prop-type-constructor`

Require prop `type` values to be constructors rather than string literals

<span id="script-require-prop-type-constructor-bad"></span>

[Bad](./type-and-script.md#script-require-prop-type-constructor-bad)

<span id="script-require-prop-type-constructor-good"></span>

[Good](./type-and-script.md#script-require-prop-type-constructor-good)

[Group examples](./type-and-script.md#script-require-prop-type-constructor) · [Rule reference](./reference/script-require-prop-type-constructor.md)

### `script/require-prop-types`

Require every prop to declare a type

<span id="script-require-prop-types-bad"></span>

[Bad](./type-and-script.md#script-require-prop-types-bad)

<span id="script-require-prop-types-good"></span>

[Good](./type-and-script.md#script-require-prop-types-good)

[Group examples](./type-and-script.md#script-require-prop-types) · [Rule reference](./reference/script-require-prop-types.md)

### `script/require-symbol-provide`

Recommend using Symbol as injection key for provide/inject

<span id="script-require-symbol-provide-bad"></span>

[Bad](./type-and-script.md#script-require-symbol-provide-bad)

<span id="script-require-symbol-provide-good"></span>

[Good](./type-and-script.md#script-require-symbol-provide-good)

[Group examples](./type-and-script.md#script-require-symbol-provide) · [Rule reference](./reference/script-require-symbol-provide.md)

### `script/require-typed-object-prop`

Require an explicit type on a prop whose runtime type is `Object` or `Array`

<span id="script-require-typed-object-prop-bad"></span>

[Bad](./type-and-script.md#script-require-typed-object-prop-bad)

<span id="script-require-typed-object-prop-good"></span>

[Good](./type-and-script.md#script-require-typed-object-prop-good)

[Group examples](./type-and-script.md#script-require-typed-object-prop) · [Rule reference](./reference/script-require-typed-object-prop.md)

### `script/require-typed-ref`

Require an explicit type argument on a ref() initialized with no value, null, or undefined

<span id="script-require-typed-ref-bad"></span>

[Bad](./type-and-script.md#script-require-typed-ref-bad)

<span id="script-require-typed-ref-good"></span>

[Good](./type-and-script.md#script-require-typed-ref-good)

[Group examples](./type-and-script.md#script-require-typed-ref) · [Rule reference](./reference/script-require-typed-ref.md)

### `script/require-valid-default-prop`

Require a prop's default value to be valid for its declared type

<span id="script-require-valid-default-prop-bad"></span>

[Bad](./type-and-script.md#script-require-valid-default-prop-bad)

<span id="script-require-valid-default-prop-good"></span>

[Good](./type-and-script.md#script-require-valid-default-prop-good)

[Group examples](./type-and-script.md#script-require-valid-default-prop) · [Rule reference](./reference/script-require-valid-default-prop.md)

### `script/return-in-computed-property`

Require a return value in every computed getter

<span id="script-return-in-computed-property-bad"></span>

[Bad](./type-and-script.md#script-return-in-computed-property-bad)

<span id="script-return-in-computed-property-good"></span>

[Good](./type-and-script.md#script-return-in-computed-property-good)

[Group examples](./type-and-script.md#script-return-in-computed-property) · [Rule reference](./reference/script-return-in-computed-property.md)

### `script/return-in-emits-validator`

Require a return value in every Vue emits validator

<span id="script-return-in-emits-validator-bad"></span>

[Bad](./type-and-script.md#script-return-in-emits-validator-bad)

<span id="script-return-in-emits-validator-good"></span>

[Good](./type-and-script.md#script-return-in-emits-validator-good)

[Group examples](./type-and-script.md#script-return-in-emits-validator) · [Rule reference](./reference/script-return-in-emits-validator.md)

### `script/valid-define-emits`

Enforce valid defineEmits() usage (no type+runtime args, no local references, single call)

<span id="script-valid-define-emits-bad"></span>

[Bad](./type-and-script.md#script-valid-define-emits-bad)

<span id="script-valid-define-emits-good"></span>

[Good](./type-and-script.md#script-valid-define-emits-good)

[Group examples](./type-and-script.md#script-valid-define-emits) · [Rule reference](./reference/script-valid-define-emits.md)

### `script/valid-define-options`

Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots)

<span id="script-valid-define-options-bad"></span>

[Bad](./type-and-script.md#script-valid-define-options-bad)

<span id="script-valid-define-options-good"></span>

[Good](./type-and-script.md#script-valid-define-options-good)

[Group examples](./type-and-script.md#script-valid-define-options) · [Rule reference](./reference/script-valid-define-options.md)

### `script/valid-define-props`

Enforce valid defineProps() usage (single call, not both type and runtime args, no local references)

<span id="script-valid-define-props-bad"></span>

[Bad](./type-and-script.md#script-valid-define-props-bad)

<span id="script-valid-define-props-good"></span>

[Good](./type-and-script.md#script-valid-define-props-good)

[Group examples](./type-and-script.md#script-valid-define-props) · [Rule reference](./reference/script-valid-define-props.md)

### `script/valid-next-tick`

Require the result of a nextTick() call to be awaited, chained, or given a callback

<span id="script-valid-next-tick-bad"></span>

[Bad](./type-and-script.md#script-valid-next-tick-bad)

<span id="script-valid-next-tick-good"></span>

[Good](./type-and-script.md#script-valid-next-tick-good)

[Group examples](./type-and-script.md#script-valid-next-tick) · [Rule reference](./reference/script-valid-next-tick.md)

### `nuxt/no-nuxt-config-test-key`

Disallow setting `test` key in Nuxt config

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

[Bad](./ecosystem.md#nuxt-no-nuxt-config-test-key-bad)

<span id="nuxt-no-nuxt-config-test-key-good"></span>

[Good](./ecosystem.md#nuxt-no-nuxt-config-test-key-good)

[Group examples](./ecosystem.md#nuxt-no-nuxt-config-test-key) · [Rule reference](./reference/nuxt-no-nuxt-config-test-key.md)

### `nuxt/no-page-meta-runtime-values`

Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

[Bad](./ecosystem.md#nuxt-no-page-meta-runtime-values-bad)

<span id="nuxt-no-page-meta-runtime-values-good"></span>

[Good](./ecosystem.md#nuxt-no-page-meta-runtime-values-good)

[Group examples](./ecosystem.md#nuxt-no-page-meta-runtime-values) · [Rule reference](./reference/nuxt-no-page-meta-runtime-values.md)

### `nuxt/nuxt-config-keys-order`

Prefer recommended order of Nuxt config properties

<span id="nuxt-nuxt-config-keys-order-bad"></span>

[Bad](./ecosystem.md#nuxt-nuxt-config-keys-order-bad)

<span id="nuxt-nuxt-config-keys-order-good"></span>

[Good](./ecosystem.md#nuxt-nuxt-config-keys-order-good)

[Group examples](./ecosystem.md#nuxt-nuxt-config-keys-order) · [Rule reference](./reference/nuxt-nuxt-config-keys-order.md)

### `nuxt/prefer-import-meta`

Prefer using `import.meta.*` over `process.*`

<span id="nuxt-prefer-import-meta-bad"></span>

[Bad](./ecosystem.md#nuxt-prefer-import-meta-bad)

<span id="nuxt-prefer-import-meta-good"></span>

[Good](./ecosystem.md#nuxt-prefer-import-meta-good)

[Group examples](./ecosystem.md#nuxt-prefer-import-meta) · [Rule reference](./reference/nuxt-prefer-import-meta.md)


## Project examples

### `ecosystem/vue-router-unknown-route`

The name is absent from the complete installed router.

<span id="ecosystem-vue-router-unknown-route-bad"></span>

[Bad](./cross-file.md#ecosystem-vue-router-unknown-route-bad)

<span id="ecosystem-vue-router-unknown-route-good"></span>

[Good](./cross-file.md#ecosystem-vue-router-unknown-route-good)

[Group examples](./cross-file.md#ecosystem-vue-router-unknown-route) · [Rule reference](./project/ecosystem-vue-router-unknown-route.md)

### `ecosystem/vue-router-extra-param`

The route does not declare tab; Vue Router discards it.

<span id="ecosystem-vue-router-extra-param-bad"></span>

[Bad](./cross-file.md#ecosystem-vue-router-extra-param-bad)

<span id="ecosystem-vue-router-extra-param-good"></span>

[Good](./cross-file.md#ecosystem-vue-router-extra-param-good)

[Group examples](./cross-file.md#ecosystem-vue-router-extra-param) · [Rule reference](./project/ecosystem-vue-router-extra-param.md)

### `ecosystem/vue-router-param-type`

postId is not repeatable, so an array is invalid.

<span id="ecosystem-vue-router-param-type-bad"></span>

[Bad](./cross-file.md#ecosystem-vue-router-param-type-bad)

<span id="ecosystem-vue-router-param-type-good"></span>

[Good](./cross-file.md#ecosystem-vue-router-param-type-good)

[Group examples](./cross-file.md#ecosystem-vue-router-param-type) · [Rule reference](./project/ecosystem-vue-router-param-type.md)

### `ecosystem/vue-router-missing-param`

Required postId is missing; relying on the current route is fragile.

<span id="ecosystem-vue-router-missing-param-bad"></span>

[Bad](./cross-file.md#ecosystem-vue-router-missing-param-bad)

<span id="ecosystem-vue-router-missing-param-good"></span>

[Good](./cross-file.md#ecosystem-vue-router-missing-param-good)

[Group examples](./cross-file.md#ecosystem-vue-router-missing-param) · [Rule reference](./project/ecosystem-vue-router-missing-param.md)

### `html/cross-component-nesting`

Check actual HTML nesting after imported components are composed.

<span id="html-cross-component-nesting-bad"></span>

[Bad](./cross-file.md#html-cross-component-nesting-bad)

<span id="html-cross-component-nesting-good"></span>

[Good](./cross-file.md#html-cross-component-nesting-good)

[Group examples](./cross-file.md#html-cross-component-nesting) · [Rule reference](./project/html-cross-component-nesting.md)

### `vue/cross-file-attrs-fallthrough`

A parent passes attributes to a resolved child whose root cannot inherit them and does not explicitly use $attrs.

<span id="vue-cross-file-attrs-fallthrough-bad"></span>

[Bad](./cross-file.md#vue-cross-file-attrs-fallthrough-bad)

<span id="vue-cross-file-attrs-fallthrough-good"></span>

[Good](./cross-file.md#vue-cross-file-attrs-fallthrough-good)

[Group examples](./cross-file.md#vue-cross-file-attrs-fallthrough) · [Rule reference](./project/vue-cross-file-attrs-fallthrough.md)

### `vize:croquis/cf/array-mutation`

This producer-free contract uses a historical Vue 2.7 array-index assignment scenario. Vue 2 does not notify updates for direct index assignment; Vue 3 tracks such assignments. The Bad/Good files illustrate the Vue 2 repair, not a Vue 3 defect.

<span id="vize-croquis-cf-array-mutation-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-array-mutation-bad)

<span id="vize-croquis-cf-array-mutation-good"></span>

[Good](./cross-file.md#vize-croquis-cf-array-mutation-good)

[Group examples](./cross-file.md#vize-croquis-cf-array-mutation) · [Rule reference](./project/vize-croquis-cf-array-mutation.md)

### `vize:croquis/cf/async-boundary`

Reactive state crosses an async boundary and can be observed stale.

<span id="vize-croquis-cf-async-boundary-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-async-boundary-bad)

<span id="vize-croquis-cf-async-boundary-good"></span>

[Good](./cross-file.md#vize-croquis-cf-async-boundary-good)

[Group examples](./cross-file.md#vize-croquis-cf-async-boundary) · [Rule reference](./project/vize-croquis-cf-async-boundary.md)

### `vize:croquis/cf/async-no-suspense`

An async component is rendered without a Suspense boundary.

<span id="vize-croquis-cf-async-no-suspense-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-async-no-suspense-bad)

<span id="vize-croquis-cf-async-no-suspense-good"></span>

[Good](./cross-file.md#vize-croquis-cf-async-no-suspense-good)

[Group examples](./cross-file.md#vize-croquis-cf-async-no-suspense) · [Rule reference](./project/vize-croquis-cf-async-no-suspense.md)

### `vize:croquis/cf/browser-api-ssr`

A browser-only API is used where the component can render on the server.

<span id="vize-croquis-cf-browser-api-ssr-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-browser-api-ssr-bad)

<span id="vize-croquis-cf-browser-api-ssr-good"></span>

[Good](./cross-file.md#vize-croquis-cf-browser-api-ssr-good)

[Group examples](./cross-file.md#vize-croquis-cf-browser-api-ssr) · [Rule reference](./project/vize-croquis-cf-browser-api-ssr.md)

### `vize:croquis/cf/circular-dep`

Components import each other in a cycle.

<span id="vize-croquis-cf-circular-dep-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-circular-dep-bad)

<span id="vize-croquis-cf-circular-dep-good"></span>

[Good](./cross-file.md#vize-croquis-cf-circular-dep-good)

[Group examples](./cross-file.md#vize-croquis-cf-circular-dep) · [Rule reference](./project/vize-croquis-cf-circular-dep.md)

### `vize:croquis/cf/circular-reactive-dependency`

Reactive computations depend on each other in a cycle.

<span id="vize-croquis-cf-circular-reactive-dependency-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-circular-reactive-dependency-bad)

<span id="vize-croquis-cf-circular-reactive-dependency-good"></span>

[Good](./cross-file.md#vize-croquis-cf-circular-reactive-dependency-good)

[Group examples](./cross-file.md#vize-croquis-cf-circular-reactive-dependency) · [Rule reference](./project/vize-croquis-cf-circular-reactive-dependency.md)

### `vize:croquis/cf/closure-captures-reactive`

A closure captures a reactive value and will not see later updates.

<span id="vize-croquis-cf-closure-captures-reactive-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-closure-captures-reactive-bad)

<span id="vize-croquis-cf-closure-captures-reactive-good"></span>

[Good](./cross-file.md#vize-croquis-cf-closure-captures-reactive-good)

[Group examples](./cross-file.md#vize-croquis-cf-closure-captures-reactive) · [Rule reference](./project/vize-croquis-cf-closure-captures-reactive.md)

### `vize:croquis/cf/composable-outside-setup`

A composable is called outside `setup`.

<span id="vize-croquis-cf-composable-outside-setup-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-composable-outside-setup-bad)

<span id="vize-croquis-cf-composable-outside-setup-good"></span>

[Good](./cross-file.md#vize-croquis-cf-composable-outside-setup-good)

[Group examples](./cross-file.md#vize-croquis-cf-composable-outside-setup) · [Rule reference](./project/vize-croquis-cf-composable-outside-setup.md)

### `vize:croquis/cf/computed-side-effects`

A computed getter writes state or performs another side effect.

<span id="vize-croquis-cf-computed-side-effects-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-computed-side-effects-bad)

<span id="vize-croquis-cf-computed-side-effects-good"></span>

[Good](./cross-file.md#vize-croquis-cf-computed-side-effects-good)

[Group examples](./cross-file.md#vize-croquis-cf-computed-side-effects) · [Rule reference](./project/vize-croquis-cf-computed-side-effects.md)

### `vize:croquis/cf/deep-import`

An import chain is deeper than the project allows.

<span id="vize-croquis-cf-deep-import-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-deep-import-bad)

<span id="vize-croquis-cf-deep-import-good"></span>

[Good](./cross-file.md#vize-croquis-cf-deep-import-good)

[Group examples](./cross-file.md#vize-croquis-cf-deep-import) · [Rule reference](./project/vize-croquis-cf-deep-import.md)

### `vize:croquis/cf/destructuring-breaks-reactivity`

Destructuring a reactive object copies the fields and drops tracking.

<span id="vize-croquis-cf-destructuring-breaks-reactivity-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity-bad)

<span id="vize-croquis-cf-destructuring-breaks-reactivity-good"></span>

[Good](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity-good)

[Group examples](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity) · [Rule reference](./project/vize-croquis-cf-destructuring-breaks-reactivity.md)

### `vize:croquis/cf/di-outside-setup`

`provide` or `inject` is called outside `setup`.

<span id="vize-croquis-cf-di-outside-setup-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-di-outside-setup-bad)

<span id="vize-croquis-cf-di-outside-setup-good"></span>

[Good](./cross-file.md#vize-croquis-cf-di-outside-setup-good)

[Group examples](./cross-file.md#vize-croquis-cf-di-outside-setup) · [Rule reference](./project/vize-croquis-cf-di-outside-setup.md)

### `vize:croquis/cf/dom-access-without-next-tick`

The DOM is read before Vue has flushed the update.

<span id="vize-croquis-cf-dom-access-without-next-tick-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick-bad)

<span id="vize-croquis-cf-dom-access-without-next-tick-good"></span>

[Good](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick-good)

[Group examples](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick) · [Rule reference](./project/vize-croquis-cf-dom-access-without-next-tick.md)

### `vize:croquis/cf/duplicate-id`

The same element id is used in more than one component.

<span id="vize-croquis-cf-duplicate-id-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-duplicate-id-bad)

<span id="vize-croquis-cf-duplicate-id-good"></span>

[Good](./cross-file.md#vize-croquis-cf-duplicate-id-good)

[Group examples](./cross-file.md#vize-croquis-cf-duplicate-id) · [Rule reference](./project/vize-croquis-cf-duplicate-id.md)

### `vize:croquis/cf/event-listener-leak`

An event listener is registered and never removed.

<span id="vize-croquis-cf-event-listener-leak-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-event-listener-leak-bad)

<span id="vize-croquis-cf-event-listener-leak-good"></span>

[Good](./cross-file.md#vize-croquis-cf-event-listener-leak-good)

[Group examples](./cross-file.md#vize-croquis-cf-event-listener-leak) · [Rule reference](./project/vize-croquis-cf-event-listener-leak.md)

### `vize:croquis/cf/event-modifier`

An event listener uses a modifier the emit does not support.

<span id="vize-croquis-cf-event-modifier-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-event-modifier-bad)

<span id="vize-croquis-cf-event-modifier-good"></span>

[Good](./cross-file.md#vize-croquis-cf-event-modifier-good)

[Group examples](./cross-file.md#vize-croquis-cf-event-modifier) · [Rule reference](./project/vize-croquis-cf-event-modifier.md)

### `vize:croquis/cf/hydration-risk`

This code groups several reactivity findings, including a prop copied into a ref. It does not imply that every Date.now() expression is detected by the cross-file pass.

<span id="vize-croquis-cf-hydration-risk-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-hydration-risk-bad)

<span id="vize-croquis-cf-hydration-risk-good"></span>

[Good](./cross-file.md#vize-croquis-cf-hydration-risk-good)

[Group examples](./cross-file.md#vize-croquis-cf-hydration-risk) · [Rule reference](./project/vize-croquis-cf-hydration-risk.md)

### `vize:croquis/cf/inherit-attrs-unused`

`inheritAttrs: false` is set and the component never reads the attributes.

<span id="vize-croquis-cf-inherit-attrs-unused-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-inherit-attrs-unused-bad)

<span id="vize-croquis-cf-inherit-attrs-unused-good"></span>

[Good](./cross-file.md#vize-croquis-cf-inherit-attrs-unused-good)

[Group examples](./cross-file.md#vize-croquis-cf-inherit-attrs-unused) · [Rule reference](./project/vize-croquis-cf-inherit-attrs-unused.md)

### `vize:croquis/cf/inject-without-symbol`

`inject` uses a plain key instead of an `InjectionKey` symbol.

<span id="vize-croquis-cf-inject-without-symbol-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-inject-without-symbol-bad)

<span id="vize-croquis-cf-inject-without-symbol-good"></span>

[Good](./cross-file.md#vize-croquis-cf-inject-without-symbol-good)

[Group examples](./cross-file.md#vize-croquis-cf-inject-without-symbol) · [Rule reference](./project/vize-croquis-cf-inject-without-symbol.md)

### `vize:croquis/cf/injected-async-mutation-race`

An injected value is mutated from an async task that can race.

<span id="vize-croquis-cf-injected-async-mutation-race-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-injected-async-mutation-race-bad)

<span id="vize-croquis-cf-injected-async-mutation-race-good"></span>

[Good](./cross-file.md#vize-croquis-cf-injected-async-mutation-race-good)

[Group examples](./cross-file.md#vize-croquis-cf-injected-async-mutation-race) · [Rule reference](./project/vize-croquis-cf-injected-async-mutation-race.md)

### `vize:croquis/cf/lifecycle-outside-setup`

A lifecycle hook is registered outside `setup`.

<span id="vize-croquis-cf-lifecycle-outside-setup-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup-bad)

<span id="vize-croquis-cf-lifecycle-outside-setup-good"></span>

[Good](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup-good)

[Group examples](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup) · [Rule reference](./project/vize-croquis-cf-lifecycle-outside-setup.md)

### `vize:croquis/cf/lifecycle-without-cleanup`

A lifecycle hook starts work and never cleans it up.

<span id="vize-croquis-cf-lifecycle-without-cleanup-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup-bad)

<span id="vize-croquis-cf-lifecycle-without-cleanup-good"></span>

[Good](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup-good)

[Group examples](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup) · [Rule reference](./project/vize-croquis-cf-lifecycle-without-cleanup.md)

### `vize:croquis/cf/missing-required-prop`

A required prop is not passed.

<span id="vize-croquis-cf-missing-required-prop-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-missing-required-prop-bad)

<span id="vize-croquis-cf-missing-required-prop-good"></span>

[Good](./cross-file.md#vize-croquis-cf-missing-required-prop-good)

[Group examples](./cross-file.md#vize-croquis-cf-missing-required-prop) · [Rule reference](./project/vize-croquis-cf-missing-required-prop.md)

### `vize:croquis/cf/missing-suspense`

An async dependency is used outside a Suspense boundary.

<span id="vize-croquis-cf-missing-suspense-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-missing-suspense-bad)

<span id="vize-croquis-cf-missing-suspense-good"></span>

[Good](./cross-file.md#vize-croquis-cf-missing-suspense-good)

[Group examples](./cross-file.md#vize-croquis-cf-missing-suspense) · [Rule reference](./project/vize-croquis-cf-missing-suspense.md)

### `vize:croquis/cf/module-scope-reactive`

Reactive state is created at module scope and shared by every caller.

<span id="vize-croquis-cf-module-scope-reactive-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-module-scope-reactive-bad)

<span id="vize-croquis-cf-module-scope-reactive-good"></span>

[Good](./cross-file.md#vize-croquis-cf-module-scope-reactive-good)

[Group examples](./cross-file.md#vize-croquis-cf-module-scope-reactive) · [Rule reference](./project/vize-croquis-cf-module-scope-reactive.md)

### `vize:croquis/cf/multi-root-attrs`

A multi-root component receives attributes and has nowhere to put them.

<span id="vize-croquis-cf-multi-root-attrs-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-multi-root-attrs-bad)

<span id="vize-croquis-cf-multi-root-attrs-good"></span>

[Good](./cross-file.md#vize-croquis-cf-multi-root-attrs-good)

[Group examples](./cross-file.md#vize-croquis-cf-multi-root-attrs) · [Rule reference](./project/vize-croquis-cf-multi-root-attrs.md)

### `vize:croquis/cf/mutated-after-escape`

A reactive object is mutated after it has escaped its owner.

<span id="vize-croquis-cf-mutated-after-escape-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-mutated-after-escape-bad)

<span id="vize-croquis-cf-mutated-after-escape-good"></span>

[Good](./cross-file.md#vize-croquis-cf-mutated-after-escape-good)

[Group examples](./cross-file.md#vize-croquis-cf-mutated-after-escape) · [Rule reference](./project/vize-croquis-cf-mutated-after-escape.md)

### `vize:croquis/cf/non-reactive-provide`

A provided value is not reactive, so descendants will not see updates.

<span id="vize-croquis-cf-non-reactive-provide-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-non-reactive-provide-bad)

<span id="vize-croquis-cf-non-reactive-provide-good"></span>

[Good](./cross-file.md#vize-croquis-cf-non-reactive-provide-good)

[Group examples](./cross-file.md#vize-croquis-cf-non-reactive-provide) · [Rule reference](./project/vize-croquis-cf-non-reactive-provide.md)

### `vize:croquis/cf/non-unique-id`

An element id inside a loop is not unique per item.

<span id="vize-croquis-cf-non-unique-id-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-non-unique-id-bad)

<span id="vize-croquis-cf-non-unique-id-good"></span>

[Good](./cross-file.md#vize-croquis-cf-non-unique-id-good)

[Group examples](./cross-file.md#vize-croquis-cf-non-unique-id) · [Rule reference](./project/vize-croquis-cf-non-unique-id.md)

### `vize:croquis/cf/object-identity-comparison`

A raw object and its reactive proxy have distinct object identities, and toRaw() returns the original object. This contract illustrates comparing stable record IDs when record identity is intended.

<span id="vize-croquis-cf-object-identity-comparison-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-object-identity-comparison-bad)

<span id="vize-croquis-cf-object-identity-comparison-good"></span>

[Good](./cross-file.md#vize-croquis-cf-object-identity-comparison-good)

[Group examples](./cross-file.md#vize-croquis-cf-object-identity-comparison) · [Rule reference](./project/vize-croquis-cf-object-identity-comparison.md)

### `vize:croquis/cf/pinia-getter`

This contract illustrates copying a Pinia getter's current primitive value during setup, leaving a snapshot that no longer follows updates. Direct getter reads within a tracked computation or template remain reactive; storeToRefs() is the extraction repair shown here.

<span id="vize-croquis-cf-pinia-getter-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-pinia-getter-bad)

<span id="vize-croquis-cf-pinia-getter-good"></span>

[Good](./cross-file.md#vize-croquis-cf-pinia-getter-good)

[Group examples](./cross-file.md#vize-croquis-cf-pinia-getter) · [Rule reference](./project/vize-croquis-cf-pinia-getter.md)

### `vize:croquis/cf/prop-type-mismatch`

A passed prop value does not match the declared type.

<span id="vize-croquis-cf-prop-type-mismatch-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-prop-type-mismatch-bad)

<span id="vize-croquis-cf-prop-type-mismatch-good"></span>

[Good](./cross-file.md#vize-croquis-cf-prop-type-mismatch-good)

[Group examples](./cross-file.md#vize-croquis-cf-prop-type-mismatch) · [Rule reference](./project/vize-croquis-cf-prop-type-mismatch.md)

### `vize:croquis/cf/provide-inject-type`

A provided value and its inject do not have the same type.

<span id="vize-croquis-cf-provide-inject-type-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-provide-inject-type-bad)

<span id="vize-croquis-cf-provide-inject-type-good"></span>

[Good](./cross-file.md#vize-croquis-cf-provide-inject-type-good)

[Group examples](./cross-file.md#vize-croquis-cf-provide-inject-type) · [Rule reference](./project/vize-croquis-cf-provide-inject-type.md)

### `vize:croquis/cf/provide-without-symbol`

`provide` uses a plain key instead of an `InjectionKey` symbol.

<span id="vize-croquis-cf-provide-without-symbol-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-provide-without-symbol-bad)

<span id="vize-croquis-cf-provide-without-symbol-good"></span>

[Good](./cross-file.md#vize-croquis-cf-provide-without-symbol-good)

[Group examples](./cross-file.md#vize-croquis-cf-provide-without-symbol) · [Rule reference](./project/vize-croquis-cf-provide-without-symbol.md)

### `vize:croquis/cf/reactive-export`

Reactive state is exported from the module.

<span id="vize-croquis-cf-reactive-export-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-reactive-export-bad)

<span id="vize-croquis-cf-reactive-export-good"></span>

[Good](./cross-file.md#vize-croquis-cf-reactive-export-good)

[Group examples](./cross-file.md#vize-croquis-cf-reactive-export) · [Rule reference](./project/vize-croquis-cf-reactive-export.md)

### `vize:croquis/cf/reactivity-outside-setup`

A reactive API is called outside `setup`.

<span id="vize-croquis-cf-reactivity-outside-setup-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-reactivity-outside-setup-bad)

<span id="vize-croquis-cf-reactivity-outside-setup-good"></span>

[Good](./cross-file.md#vize-croquis-cf-reactivity-outside-setup-good)

[Group examples](./cross-file.md#vize-croquis-cf-reactivity-outside-setup) · [Rule reference](./project/vize-croquis-cf-reactivity-outside-setup.md)

### `vize:croquis/cf/reassignment-breaks-reactivity`

Reassigning a reactive binding replaces it with a plain value.

<span id="vize-croquis-cf-reassignment-breaks-reactivity-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity-bad)

<span id="vize-croquis-cf-reassignment-breaks-reactivity-good"></span>

[Good](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity-good)

[Group examples](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity) · [Rule reference](./project/vize-croquis-cf-reassignment-breaks-reactivity.md)

### `vize:croquis/cf/reference-escapes-scope`

A reactive reference escapes the scope that owns its lifetime.

<span id="vize-croquis-cf-reference-escapes-scope-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-reference-escapes-scope-bad)

<span id="vize-croquis-cf-reference-escapes-scope-good"></span>

[Good](./cross-file.md#vize-croquis-cf-reference-escapes-scope-good)

[Group examples](./cross-file.md#vize-croquis-cf-reference-escapes-scope) · [Rule reference](./project/vize-croquis-cf-reference-escapes-scope.md)

### `vize:croquis/cf/setup-context-violation`

This experimental analyzer scenario places ref(0) at normal script module scope rather than per-instance setup. Vue permits this reactive state; the example requires each component instance to own its own count. The CLI does not emit this individual code.

<span id="vize-croquis-cf-setup-context-violation-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-setup-context-violation-bad)

<span id="vize-croquis-cf-setup-context-violation-good"></span>

[Good](./cross-file.md#vize-croquis-cf-setup-context-violation-good)

[Group examples](./cross-file.md#vize-croquis-cf-setup-context-violation) · [Rule reference](./project/vize-croquis-cf-setup-context-violation.md)

### `vize:croquis/cf/shallow-deep-access`

A deep property of a `shallowReactive` or `shallowRef` value is read as if it were tracked.

<span id="vize-croquis-cf-shallow-deep-access-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-shallow-deep-access-bad)

<span id="vize-croquis-cf-shallow-deep-access-good"></span>

[Good](./cross-file.md#vize-croquis-cf-shallow-deep-access-good)

[Group examples](./cross-file.md#vize-croquis-cf-shallow-deep-access) · [Rule reference](./project/vize-croquis-cf-shallow-deep-access.md)

### `vize:croquis/cf/spread-breaks-reactivity`

Spreading a reactive object copies its values and drops the tracking.

<span id="vize-croquis-cf-spread-breaks-reactivity-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity-bad)

<span id="vize-croquis-cf-spread-breaks-reactivity-good"></span>

[Good](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity-good)

[Group examples](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity) · [Rule reference](./project/vize-croquis-cf-spread-breaks-reactivity.md)

### `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` has no fallback content.

<span id="vize-croquis-cf-suspense-no-fallback-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-suspense-no-fallback-bad)

<span id="vize-croquis-cf-suspense-no-fallback-good"></span>

[Good](./cross-file.md#vize-croquis-cf-suspense-no-fallback-good)

[Group examples](./cross-file.md#vize-croquis-cf-suspense-no-fallback) · [Rule reference](./project/vize-croquis-cf-suspense-no-fallback.md)

### `vize:croquis/cf/template-ref-timing`

A template ref is read before the component is mounted.

<span id="vize-croquis-cf-template-ref-timing-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-template-ref-timing-bad)

<span id="vize-croquis-cf-template-ref-timing-good"></span>

[Good](./cross-file.md#vize-croquis-cf-template-ref-timing-good)

[Group examples](./cross-file.md#vize-croquis-cf-template-ref-timing) · [Rule reference](./project/vize-croquis-cf-template-ref-timing.md)

### `vize:croquis/cf/toraw-mutation`

`toRaw` is used and the raw object is then mutated.

<span id="vize-croquis-cf-toraw-mutation-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-toraw-mutation-bad)

<span id="vize-croquis-cf-toraw-mutation-good"></span>

[Good](./cross-file.md#vize-croquis-cf-toraw-mutation-good)

[Group examples](./cross-file.md#vize-croquis-cf-toraw-mutation) · [Rule reference](./project/vize-croquis-cf-toraw-mutation.md)

### `vize:croquis/cf/uncaught-error`

A component can throw and no error boundary catches it.

<span id="vize-croquis-cf-uncaught-error-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-uncaught-error-bad)

<span id="vize-croquis-cf-uncaught-error-good"></span>

[Good](./cross-file.md#vize-croquis-cf-uncaught-error-good)

[Group examples](./cross-file.md#vize-croquis-cf-uncaught-error) · [Rule reference](./project/vize-croquis-cf-uncaught-error.md)

### `vize:croquis/cf/undeclared-emit`

The component emits an event that is not declared.

<span id="vize-croquis-cf-undeclared-emit-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-undeclared-emit-bad)

<span id="vize-croquis-cf-undeclared-emit-good"></span>

[Good](./cross-file.md#vize-croquis-cf-undeclared-emit-good)

[Group examples](./cross-file.md#vize-croquis-cf-undeclared-emit) · [Rule reference](./project/vize-croquis-cf-undeclared-emit.md)

### `vize:croquis/cf/undeclared-prop`

A parent passes a prop the child does not declare.

<span id="vize-croquis-cf-undeclared-prop-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-undeclared-prop-bad)

<span id="vize-croquis-cf-undeclared-prop-good"></span>

[Good](./cross-file.md#vize-croquis-cf-undeclared-prop-good)

[Group examples](./cross-file.md#vize-croquis-cf-undeclared-prop) · [Rule reference](./project/vize-croquis-cf-undeclared-prop.md)

### `vize:croquis/cf/undefined-slot`

A parent fills a slot the child does not expose.

<span id="vize-croquis-cf-undefined-slot-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-undefined-slot-bad)

<span id="vize-croquis-cf-undefined-slot-good"></span>

[Good](./cross-file.md#vize-croquis-cf-undefined-slot-good)

[Group examples](./cross-file.md#vize-croquis-cf-undefined-slot) · [Rule reference](./project/vize-croquis-cf-undefined-slot.md)

### `vize:croquis/cf/unhandled-event`

A child emits an event that no parent handles.

<span id="vize-croquis-cf-unhandled-event-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-unhandled-event-bad)

<span id="vize-croquis-cf-unhandled-event-good"></span>

[Good](./cross-file.md#vize-croquis-cf-unhandled-event-good)

[Group examples](./cross-file.md#vize-croquis-cf-unhandled-event) · [Rule reference](./project/vize-croquis-cf-unhandled-event.md)

### `vize:croquis/cf/unmatched-inject`

`inject` names a key that no ancestor provides.

<span id="vize-croquis-cf-unmatched-inject-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-unmatched-inject-bad)

<span id="vize-croquis-cf-unmatched-inject-good"></span>

[Good](./cross-file.md#vize-croquis-cf-unmatched-inject-good)

[Group examples](./cross-file.md#vize-croquis-cf-unmatched-inject) · [Rule reference](./project/vize-croquis-cf-unmatched-inject.md)

### `vize:croquis/cf/unmatched-listener`

A parent listens for an event the child does not emit.

<span id="vize-croquis-cf-unmatched-listener-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-unmatched-listener-bad)

<span id="vize-croquis-cf-unmatched-listener-good"></span>

[Good](./cross-file.md#vize-croquis-cf-unmatched-listener-good)

[Group examples](./cross-file.md#vize-croquis-cf-unmatched-listener) · [Rule reference](./project/vize-croquis-cf-unmatched-listener.md)

### `vize:croquis/cf/unregistered-component`

A template uses a component that is not registered or imported.

<span id="vize-croquis-cf-unregistered-component-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-unregistered-component-bad)

<span id="vize-croquis-cf-unregistered-component-good"></span>

[Good](./cross-file.md#vize-croquis-cf-unregistered-component-good)

[Group examples](./cross-file.md#vize-croquis-cf-unregistered-component) · [Rule reference](./project/vize-croquis-cf-unregistered-component.md)

### `vize:croquis/cf/unresolved-import`

An import does not resolve to a module.

<span id="vize-croquis-cf-unresolved-import-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-unresolved-import-bad)

<span id="vize-croquis-cf-unresolved-import-good"></span>

[Good](./cross-file.md#vize-croquis-cf-unresolved-import-good)

[Group examples](./cross-file.md#vize-croquis-cf-unresolved-import) · [Rule reference](./project/vize-croquis-cf-unresolved-import.md)

### `vize:croquis/cf/unused-attrs`

Fallthrough attributes are passed to a multi-root component that does not use them.

<span id="vize-croquis-cf-unused-attrs-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-unused-attrs-bad)

<span id="vize-croquis-cf-unused-attrs-good"></span>

[Good](./cross-file.md#vize-croquis-cf-unused-attrs-good)

[Group examples](./cross-file.md#vize-croquis-cf-unused-attrs) · [Rule reference](./project/vize-croquis-cf-unused-attrs.md)

### `vize:croquis/cf/unused-emit`

A declared emit is never used.

<span id="vize-croquis-cf-unused-emit-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-unused-emit-bad)

<span id="vize-croquis-cf-unused-emit-good"></span>

[Good](./cross-file.md#vize-croquis-cf-unused-emit-good)

[Group examples](./cross-file.md#vize-croquis-cf-unused-emit) · [Rule reference](./project/vize-croquis-cf-unused-emit.md)

### `vize:croquis/cf/unused-provide`

A provided key is never injected.

<span id="vize-croquis-cf-unused-provide-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-unused-provide-bad)

<span id="vize-croquis-cf-unused-provide-good"></span>

[Good](./cross-file.md#vize-croquis-cf-unused-provide-good)

[Group examples](./cross-file.md#vize-croquis-cf-unused-provide) · [Rule reference](./project/vize-croquis-cf-unused-provide.md)

### `vize:croquis/cf/value-extraction-breaks-reactivity`

Reading a reactive value out into a local drops later updates.

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity-bad)

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-good"></span>

[Good](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity-good)

[Group examples](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity) · [Rule reference](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md)

### `vize:croquis/cf/watch-can-be-computed`

A watcher only copies a value into state and can be a computed.

<span id="vize-croquis-cf-watch-can-be-computed-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-watch-can-be-computed-bad)

<span id="vize-croquis-cf-watch-can-be-computed-good"></span>

[Good](./cross-file.md#vize-croquis-cf-watch-can-be-computed-good)

[Group examples](./cross-file.md#vize-croquis-cf-watch-can-be-computed) · [Rule reference](./project/vize-croquis-cf-watch-can-be-computed.md)

### `vize:croquis/cf/watcheffect-async`

The async watchEffect shown here has no invalidation cleanup or stale-response guard. Vue supports cleanup for watchEffect; the Good example uses an explicit watch source and cancels invalidated work.

<span id="vize-croquis-cf-watcheffect-async-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-watcheffect-async-bad)

<span id="vize-croquis-cf-watcheffect-async-good"></span>

[Good](./cross-file.md#vize-croquis-cf-watcheffect-async-good)

[Group examples](./cross-file.md#vize-croquis-cf-watcheffect-async) · [Rule reference](./project/vize-croquis-cf-watcheffect-async.md)

### `vize:croquis/cf/watcher-outside-setup`

`watch` or `watchEffect` is called outside `setup`.

<span id="vize-croquis-cf-watcher-outside-setup-bad"></span>

[Bad](./cross-file.md#vize-croquis-cf-watcher-outside-setup-bad)

<span id="vize-croquis-cf-watcher-outside-setup-good"></span>

[Good](./cross-file.md#vize-croquis-cf-watcher-outside-setup-good)

[Group examples](./cross-file.md#vize-croquis-cf-watcher-outside-setup) · [Rule reference](./project/vize-croquis-cf-watcher-outside-setup.md)
