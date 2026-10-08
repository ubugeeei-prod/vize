---
title: ESLint rule migration map
---

# ESLint rule migration map

The committed ESLint mapping records 252 rule IDs: 123 mapped, 2 intentional divergences, and 127 unimplemented. A mapped ID identifies a Vize rule; it does not guarantee identical options, findings, fixes, or coverage.

Enable Vize rules under `lint.vize.rules` in the Vite+ configuration. Keep JS/TS rules handled by Oxlint in `lint.rules`. Do not copy an ESLint option tuple directly: Vize severity and typed `ruleOptions` are separate.

```ts annotate="remove:1,2;add:3,4,5,6"
 import vue from "eslint-plugin-vue";
 export default [{ plugins: { vue }, rules: { "vue/attributes-order": "warn" } }];
 import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
 export default defineConfig({
   lint: { vize: { rules: { "vue/attribute-order": "warn" } } },
 });
```

Remove only the overlapping Vue rule after checking the linked Bad/Good examples. Run `vp run lint`; if an existing package script occupies that task name, the generated task is `vp run vize:lint`. Explicit task configuration takes precedence.

`vue/component-definition-name-casing` has a deliberate scope difference: Vize checks the SFC filename, while ESLint checks the component definition name. The mapping below retains this distinction.

For option conversion, see [Rule Options](./options.md) and [props destructuring modes](./reference/script-define-props-destructuring.md). Unsupported rows must stay with an existing checker until a supported replacement is verified.

| ESLint ID | Vize ID / reference | Status |
| --- | --- | --- |
| `vue/array-bracket-newline` | — | unimplemented |
| `vue/array-bracket-spacing` | — | unimplemented |
| `vue/array-element-newline` | — | unimplemented |
| `vue/arrow-spacing` | — | unimplemented |
| `vue/attribute-hyphenation` | [`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | mapped |
| `vue/attributes-order` | [`vue/attribute-order`](./reference/vue-attribute-order.md) | mapped |
| `vue/block-lang` | — | unimplemented |
| `vue/block-order` | [`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | mapped |
| `vue/block-spacing` | — | unimplemented |
| `vue/block-tag-newline` | — | unimplemented |
| `vue/brace-style` | — | unimplemented |
| `vue/camelcase` | — | unimplemented |
| `vue/comma-dangle` | — | unimplemented |
| `vue/comma-spacing` | — | unimplemented |
| `vue/comma-style` | — | unimplemented |
| `vue/comment-directive` | — | unimplemented |
| `vue/component-api-style` | — | unimplemented |
| `vue/component-definition-name-casing` | — | intentional-divergence |
| `vue/component-name-in-template-casing` | [`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | mapped |
| `vue/component-options-name-casing` | [`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | mapped |
| `vue/custom-event-name-casing` | [`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | mapped |
| `vue/define-emits-declaration` | [`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | mapped |
| `vue/define-macros-order` | [`script/define-macros-order`](./reference/script-define-macros-order.md) | mapped |
| `vue/define-props-declaration` | [`script/define-props-declaration`](./reference/script-define-props-declaration.md) | mapped |
| `vue/define-props-destructuring` | [`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | mapped |
| `vue/dot-location` | — | unimplemented |
| `vue/dot-notation` | — | unimplemented |
| `vue/enforce-style-attribute` | — | unimplemented |
| `vue/eqeqeq` | — | unimplemented |
| `vue/first-attribute-linebreak` | — | unimplemented |
| `vue/func-call-spacing` | — | unimplemented |
| `vue/html-button-has-type` | [`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | mapped |
| `vue/html-closing-bracket-newline` | — | unimplemented |
| `vue/html-closing-bracket-spacing` | — | unimplemented |
| `vue/html-comment-content-newline` | — | unimplemented |
| `vue/html-comment-content-spacing` | — | unimplemented |
| `vue/html-comment-indent` | — | unimplemented |
| `vue/html-end-tags` | — | unimplemented |
| `vue/html-indent` | — | unimplemented |
| `vue/html-quotes` | [`vue/html-quotes`](./reference/vue-html-quotes.md) | mapped |
| `vue/html-self-closing` | [`vue/html-self-closing`](./reference/vue-html-self-closing.md) | mapped |
| `vue/jsx-uses-vars` | — | unimplemented |
| `vue/key-spacing` | — | unimplemented |
| `vue/keyword-spacing` | — | unimplemented |
| `vue/match-component-file-name` | — | unimplemented |
| `vue/match-component-import-name` | — | unimplemented |
| `vue/max-attributes-per-line` | — | unimplemented |
| `vue/max-len` | — | unimplemented |
| `vue/max-lines-per-block` | — | unimplemented |
| `vue/max-props` | — | unimplemented |
| `vue/max-template-depth` | — | unimplemented |
| `vue/multi-word-component-names` | [`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | mapped |
| `vue/multiline-html-element-content-newline` | — | unimplemented |
| `vue/multiline-ternary` | — | unimplemented |
| `vue/mustache-interpolation-spacing` | [`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | mapped |
| `vue/new-line-between-multi-line-property` | — | unimplemented |
| `vue/next-tick-style` | — | unimplemented |
| `vue/no-arrow-functions-in-watch` | [`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | mapped |
| `vue/no-async-in-computed-properties` | [`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | mapped |
| `vue/no-bare-strings-in-template` | [`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | mapped |
| `vue/no-boolean-default` | [`script/no-boolean-default`](./reference/script-no-boolean-default.md) | mapped |
| `vue/no-child-content` | [`vue/no-child-content`](./reference/vue-no-child-content.md) | mapped |
| `vue/no-computed-properties-in-data` | — | unimplemented |
| `vue/no-console` | — | unimplemented |
| `vue/no-constant-condition` | — | unimplemented |
| `vue/no-custom-modifiers-on-v-model` | — | unimplemented |
| `vue/no-deprecated-data-object-declaration` | [`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | mapped |
| `vue/no-deprecated-delete-set` | — | unimplemented |
| `vue/no-deprecated-destroyed-lifecycle` | [`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | mapped |
| `vue/no-deprecated-dollar-listeners-api` | [`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | mapped |
| `vue/no-deprecated-dollar-scopedslots-api` | [`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | mapped |
| `vue/no-deprecated-events-api` | [`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | mapped |
| `vue/no-deprecated-filter` | [`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | mapped |
| `vue/no-deprecated-functional-template` | [`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | mapped |
| `vue/no-deprecated-html-element-is` | [`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | mapped |
| `vue/no-deprecated-inline-template` | [`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | mapped |
| `vue/no-deprecated-model-definition` | — | unimplemented |
| `vue/no-deprecated-props-default-this` | [`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | mapped |
| `vue/no-deprecated-router-link-tag-prop` | [`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | mapped |
| `vue/no-deprecated-scope-attribute` | [`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | mapped |
| `vue/no-deprecated-slot-attribute` | [`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | mapped |
| `vue/no-deprecated-slot-scope-attribute` | [`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | mapped |
| `vue/no-deprecated-v-bind-sync` | [`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | mapped |
| `vue/no-deprecated-v-is` | — | unimplemented |
| `vue/no-deprecated-v-on-native-modifier` | [`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | mapped |
| `vue/no-deprecated-v-on-number-modifiers` | [`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | mapped |
| `vue/no-deprecated-vue-config-keycodes` | — | unimplemented |
| `vue/no-dupe-keys` | [`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | mapped |
| `vue/no-dupe-v-else-if` | [`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | mapped |
| `vue/no-duplicate-attr-inheritance` | [`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | mapped |
| `vue/no-duplicate-attributes` | [`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | mapped |
| `vue/no-duplicate-class-names` | — | unimplemented |
| `vue/no-empty-component-block` | [`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | mapped |
| `vue/no-empty-pattern` | — | unimplemented |
| `vue/no-export-in-script-setup` | [`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | mapped |
| `vue/no-expose-after-await` | — | unimplemented |
| `vue/no-extra-parens` | — | unimplemented |
| `vue/no-implicit-coercion` | — | unimplemented |
| `vue/no-import-compiler-macros` | [`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | mapped |
| `vue/no-irregular-whitespace` | — | unimplemented |
| `vue/no-lifecycle-after-await` | — | unimplemented |
| `vue/no-literals-in-template` | — | unimplemented |
| `vue/no-lone-template` | [`vue/no-lone-template`](./reference/vue-no-lone-template.md) | mapped |
| `vue/no-loss-of-precision` | — | unimplemented |
| `vue/no-multi-spaces` | [`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | mapped |
| `vue/no-multiple-objects-in-class` | [`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | mapped |
| `vue/no-multiple-slot-args` | [`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | mapped |
| `vue/no-multiple-template-root` | [`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | mapped |
| `vue/no-mutating-props` | [`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | mapped |
| `vue/no-negated-condition` | — | unimplemented |
| `vue/no-negated-v-if-condition` | [`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | mapped |
| `vue/no-parsing-error` | — | unimplemented |
| `vue/no-potential-component-option-typo` | [`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | mapped |
| `vue/no-ref-as-operand` | [`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | mapped |
| `vue/no-ref-object-reactivity-loss` | — | unimplemented |
| `vue/no-required-prop-with-default` | [`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | mapped |
| `vue/no-reserved-component-names` | [`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | mapped |
| `vue/no-reserved-keys` | [`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | mapped |
| `vue/no-reserved-props` | [`script/no-reserved-props`](./reference/script-no-reserved-props.md) | mapped |
| `vue/no-restricted-block` | — | unimplemented |
| `vue/no-restricted-call-after-await` | — | unimplemented |
| `vue/no-restricted-class` | — | unimplemented |
| `vue/no-restricted-component-names` | — | unimplemented |
| `vue/no-restricted-component-options` | — | unimplemented |
| `vue/no-restricted-custom-event` | — | unimplemented |
| `vue/no-restricted-html-elements` | — | unimplemented |
| `vue/no-restricted-props` | — | unimplemented |
| `vue/no-restricted-static-attribute` | — | unimplemented |
| `vue/no-restricted-syntax` | — | unimplemented |
| `vue/no-restricted-v-bind` | — | unimplemented |
| `vue/no-restricted-v-on` | — | unimplemented |
| `vue/no-root-v-if` | [`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | mapped |
| `vue/no-setup-props-reactivity-loss` | — | unimplemented |
| `vue/no-shared-component-data` | — | unimplemented |
| `vue/no-side-effects-in-computed-properties` | [`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | mapped |
| `vue/no-spaces-around-equal-signs-in-attribute` | — | unimplemented |
| `vue/no-sparse-arrays` | — | unimplemented |
| `vue/no-static-inline-styles` | [`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | mapped |
| `vue/no-template-key` | [`vue/no-template-key`](./reference/vue-no-template-key.md) | mapped |
| `vue/no-template-shadow` | [`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | mapped |
| `vue/no-template-target-blank` | [`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | mapped |
| `vue/no-textarea-mustache` | [`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | mapped |
| `vue/no-this-in-before-route-enter` | — | unimplemented |
| `vue/no-undef-components` | — | unimplemented |
| `vue/no-undef-directives` | — | unimplemented |
| `vue/no-undef-properties` | — | unimplemented |
| `vue/no-unsupported-features` | — | unimplemented |
| `vue/no-unused-components` | [`vue/no-unused-components`](./reference/vue-no-unused-components.md) | mapped |
| `vue/no-unused-emit-declarations` | [`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | mapped |
| `vue/no-unused-properties` | — | intentional-divergence |
| `vue/no-unused-refs` | [`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | mapped |
| `vue/no-unused-vars` | [`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | mapped |
| `vue/no-use-computed-property-like-method` | [`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | mapped |
| `vue/no-use-v-else-with-v-for` | [`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | mapped |
| `vue/no-use-v-if-with-v-for` | [`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | mapped |
| `vue/no-useless-concat` | — | unimplemented |
| `vue/no-useless-mustaches` | [`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | mapped |
| `vue/no-useless-template-attributes` | [`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | mapped |
| `vue/no-useless-v-bind` | [`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | mapped |
| `vue/no-v-for-template-key` | — | unimplemented |
| `vue/no-v-for-template-key-on-child` | [`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | mapped |
| `vue/no-v-html` | [`vue/no-v-html`](./reference/vue-no-v-html.md) | mapped |
| `vue/no-v-model-argument` | — | unimplemented |
| `vue/no-v-text` | [`vue/no-v-text`](./reference/vue-no-v-text.md) | mapped |
| `vue/no-v-text-v-html-on-component` | [`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | mapped |
| `vue/no-watch-after-await` | — | unimplemented |
| `vue/object-curly-newline` | — | unimplemented |
| `vue/object-curly-spacing` | — | unimplemented |
| `vue/object-property-newline` | — | unimplemented |
| `vue/object-shorthand` | — | unimplemented |
| `vue/one-component-per-file` | — | unimplemented |
| `vue/operator-linebreak` | — | unimplemented |
| `vue/order-in-components` | — | unimplemented |
| `vue/padding-line-between-blocks` | — | unimplemented |
| `vue/padding-line-between-tags` | — | unimplemented |
| `vue/padding-lines-in-component-definition` | — | unimplemented |
| `vue/prefer-define-options` | [`script/prefer-define-options`](./reference/script-prefer-define-options.md) | mapped |
| `vue/prefer-import-from-vue` | [`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | mapped |
| `vue/prefer-prop-type-boolean-first` | — | unimplemented |
| `vue/prefer-separate-static-class` | — | unimplemented |
| `vue/prefer-single-event-payload` | — | unimplemented |
| `vue/prefer-template` | — | unimplemented |
| `vue/prefer-true-attribute-shorthand` | [`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | mapped |
| `vue/prefer-use-template-ref` | [`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | mapped |
| `vue/prefer-v-model` | — | unimplemented |
| `vue/prop-name-casing` | [`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | mapped |
| `vue/quote-props` | — | unimplemented |
| `vue/require-component-is` | [`vue/require-component-is`](./reference/vue-require-component-is.md) | mapped |
| `vue/require-default-export` | — | unimplemented |
| `vue/require-default-prop` | [`script/require-default-prop`](./reference/script-require-default-prop.md) | mapped |
| `vue/require-direct-export` | — | unimplemented |
| `vue/require-emit-validator` | — | unimplemented |
| `vue/require-explicit-emits` | [`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | mapped |
| `vue/require-explicit-slots` | [`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | mapped |
| `vue/require-expose` | — | unimplemented |
| `vue/require-macro-variable-name` | — | unimplemented |
| `vue/require-name-property` | — | unimplemented |
| `vue/require-prop-comment` | — | unimplemented |
| `vue/require-prop-type-constructor` | [`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | mapped |
| `vue/require-prop-types` | [`script/require-prop-types`](./reference/script-require-prop-types.md) | mapped |
| `vue/require-render-return` | — | unimplemented |
| `vue/require-slots-as-functions` | — | unimplemented |
| `vue/require-toggle-inside-transition` | [`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | mapped |
| `vue/require-typed-object-prop` | [`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | mapped |
| `vue/require-typed-ref` | [`script/require-typed-ref`](./reference/script-require-typed-ref.md) | mapped |
| `vue/require-v-for-key` | [`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | mapped |
| `vue/require-valid-default-prop` | [`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | mapped |
| `vue/restricted-component-names` | — | unimplemented |
| `vue/return-in-computed-property` | [`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | mapped |
| `vue/return-in-emits-validator` | [`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | mapped |
| `vue/script-indent` | — | unimplemented |
| `vue/singleline-html-element-content-newline` | — | unimplemented |
| `vue/slot-name-casing` | [`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | mapped |
| `vue/sort-keys` | — | unimplemented |
| `vue/space-in-parens` | — | unimplemented |
| `vue/space-infix-ops` | — | unimplemented |
| `vue/space-unary-ops` | — | unimplemented |
| `vue/static-class-names-order` | — | unimplemented |
| `vue/template-curly-spacing` | — | unimplemented |
| `vue/this-in-template` | [`vue/this-in-template`](./reference/vue-this-in-template.md) | mapped |
| `vue/use-v-on-exact` | [`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | mapped |
| `vue/v-bind-style` | [`vue/v-bind-style`](./reference/vue-v-bind-style.md) | mapped |
| `vue/v-for-delimiter-style` | — | unimplemented |
| `vue/v-if-else-key` | — | unimplemented |
| `vue/v-on-event-hyphenation` | [`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | mapped |
| `vue/v-on-handler-style` | [`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | mapped |
| `vue/v-on-style` | [`vue/v-on-style`](./reference/vue-v-on-style.md) | mapped |
| `vue/v-slot-style` | [`vue/v-slot-style`](./reference/vue-v-slot-style.md) | mapped |
| `vue/valid-attribute-name` | [`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | mapped |
| `vue/valid-define-emits` | [`script/valid-define-emits`](./reference/script-valid-define-emits.md) | mapped |
| `vue/valid-define-options` | [`script/valid-define-options`](./reference/script-valid-define-options.md) | mapped |
| `vue/valid-define-props` | [`script/valid-define-props`](./reference/script-valid-define-props.md) | mapped |
| `vue/valid-model-definition` | — | unimplemented |
| `vue/valid-next-tick` | [`script/valid-next-tick`](./reference/script-valid-next-tick.md) | mapped |
| `vue/valid-template-root` | [`vue/valid-template-root`](./reference/vue-valid-template-root.md) | mapped |
| `vue/valid-v-bind` | [`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | mapped |
| `vue/valid-v-bind-sync` | — | unimplemented |
| `vue/valid-v-cloak` | [`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | mapped |
| `vue/valid-v-else` | [`vue/valid-v-else`](./reference/vue-valid-v-else.md) | mapped |
| `vue/valid-v-else-if` | — | unimplemented |
| `vue/valid-v-for` | [`vue/valid-v-for`](./reference/vue-valid-v-for.md) | mapped |
| `vue/valid-v-html` | [`vue/valid-v-html`](./reference/vue-valid-v-html.md) | mapped |
| `vue/valid-v-if` | [`vue/valid-v-if`](./reference/vue-valid-v-if.md) | mapped |
| `vue/valid-v-is` | — | unimplemented |
| `vue/valid-v-memo` | [`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | mapped |
| `vue/valid-v-model` | [`vue/valid-v-model`](./reference/vue-valid-v-model.md) | mapped |
| `vue/valid-v-on` | [`vue/valid-v-on`](./reference/vue-valid-v-on.md) | mapped |
| `vue/valid-v-once` | [`vue/valid-v-once`](./reference/vue-valid-v-once.md) | mapped |
| `vue/valid-v-pre` | — | unimplemented |
| `vue/valid-v-show` | [`vue/valid-v-show`](./reference/vue-valid-v-show.md) | mapped |
| `vue/valid-v-slot` | [`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | mapped |
| `vue/valid-v-text` | [`vue/valid-v-text`](./reference/vue-valid-v-text.md) | mapped |

[Committed mapping and capture identity](https://github.com/ubugeeei-prod/vize/blob/main/tests/_fixtures/patina-eslint-vue-rule-map.json)
