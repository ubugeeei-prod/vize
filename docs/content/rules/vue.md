---
title: Vue Rules
---

# Vue Rules

Vue rules are Patina single-file rules. They inspect SFC template structure, directive syntax,
component naming, and Vue-specific correctness hazards before the code reaches the runtime.

Each detail page lists rule behavior, default severity, presets, and Bad/Good examples.
See [Vue Rule Options](./options-vue.md) for configurable rule settings.

## Rule Detail Pages

- [Template Structure](./vue-template-structure.md): Loops, conditional branches, template scope, and template containers.
- [Template Safety](./vue-template-safety.md): HTML content, duplicate attributes, URL safety, and reusable element IDs.
- [Components and Props](./vue-components.md): Component naming, registration, dynamic components, and prop ownership.
- [Template Formatting](./vue-formatting.md): Attribute conventions, tag style, whitespace, and prop spelling.
- [SFC Blocks](./vue-sfc.md): Block order, supported languages, external sources, and style scope.
- [Directive Conventions](./vue-directives.md): Event naming, modifier usage, directive spelling, and custom directives.
- [Directive Validity](./vue-directive-validity.md): Required arguments and valid expressions for built-in Vue directives.

## Rule References

The rule links below preserve the headings used by existing bookmarks.

## `vue/require-v-for-key`

See [Template Structure: `require-v-for-key`](./vue-template-structure.md#vuerequire-v-for-key).

## `vue/no-use-v-if-with-v-for`

See [Template Structure: `no-use-v-if-with-v-for`](./vue-template-structure.md#vueno-use-v-if-with-v-for).

## `vue/no-child-content`

See [Template Structure: `no-child-content`](./vue-template-structure.md#vueno-child-content).

## `vue/no-dupe-v-else-if`

See [Template Structure: `no-dupe-v-else-if`](./vue-template-structure.md#vueno-dupe-v-else-if).

## `vue/no-template-shadow`

See [Template Structure: `no-template-shadow`](./vue-template-structure.md#vueno-template-shadow).

## `vue/no-lone-template`

See [Template Structure: `no-lone-template`](./vue-template-structure.md#vueno-lone-template).

## `vue/no-template-key`

See [Template Structure: `no-template-key`](./vue-template-structure.md#vueno-template-key).

## `vue/no-unused-vars`

See [Template Structure: `no-unused-vars`](./vue-template-structure.md#vueno-unused-vars).

## `vue/no-useless-template-attributes`

See [Template Structure: `no-useless-template-attributes`](./vue-template-structure.md#vueno-useless-template-attributes).

## `vue/no-v-html`

See [Template Safety: `no-v-html`](./vue-template-safety.md#vueno-v-html).

## `vue/no-duplicate-attributes`

See [Template Safety: `no-duplicate-attributes`](./vue-template-safety.md#vueno-duplicate-attributes).

## `vue/no-unsafe-url`

See [Template Safety: `no-unsafe-url`](./vue-template-safety.md#vueno-unsafe-url).

## `vue/use-unique-element-ids`

See [Template Safety: `use-unique-element-ids`](./vue-template-safety.md#vueuse-unique-element-ids).

## `vue/no-textarea-mustache`

See [Template Safety: `no-textarea-mustache`](./vue-template-safety.md#vueno-textarea-mustache).

## `vue/no-v-text-v-html-on-component`

See [Template Safety: `no-v-text-v-html-on-component`](./vue-template-safety.md#vueno-v-text-v-html-on-component).

## `vue/permitted-contents`

See [Template Safety: `permitted-contents`](./vue-template-safety.md#vuepermitted-contents).

## `vue/no-mutating-props`

See [Components and Props: `no-mutating-props`](./vue-components.md#vueno-mutating-props).

## `vue/no-unused-components`

See [Components and Props: `no-unused-components`](./vue-components.md#vueno-unused-components).

## `vue/no-unused-properties`

See [Components and Props: `no-unused-properties`](./vue-components.md#vueno-unused-properties).

## `vue/require-component-is`

See [Components and Props: `require-component-is`](./vue-components.md#vuerequire-component-is).

## `vue/component-definition-name-casing`

See [Components and Props: `component-definition-name-casing`](./vue-components.md#vuecomponent-definition-name-casing).

## `vue/component-name-in-template-casing`

See [Components and Props: `component-name-in-template-casing`](./vue-components.md#vuecomponent-name-in-template-casing).

## `vue/multi-word-component-names`

See [Components and Props: `multi-word-component-names`](./vue-components.md#vuemulti-word-component-names).

## `vue/no-non-component-keep-alive-child`

See [Components and Props: `no-non-component-keep-alive-child`](./vue-components.md#vueno-non-component-keep-alive-child).

## `vue/no-reserved-component-names`

See [Components and Props: `no-reserved-component-names`](./vue-components.md#vueno-reserved-component-names).

## `vue/require-component-registration`

See [Components and Props: `require-component-registration`](./vue-components.md#vuerequire-component-registration).

## `vue/attribute-hyphenation`

See [Template Formatting: `attribute-hyphenation`](./vue-formatting.md#vueattribute-hyphenation).

## `vue/attribute-order`

See [Template Formatting: `attribute-order`](./vue-formatting.md#vueattribute-order).

## `vue/html-quotes`

See [Template Formatting: `html-quotes`](./vue-formatting.md#vuehtml-quotes).

## `vue/html-self-closing`

See [Template Formatting: `html-self-closing`](./vue-formatting.md#vuehtml-self-closing).

## `vue/mustache-interpolation-spacing`

See [Template Formatting: `mustache-interpolation-spacing`](./vue-formatting.md#vuemustache-interpolation-spacing).

## `vue/no-boolean-attr-value`

See [Template Formatting: `no-boolean-attr-value`](./vue-formatting.md#vueno-boolean-attr-value).

## `vue/no-inline-style`

See [Template Formatting: `no-inline-style`](./vue-formatting.md#vueno-inline-style).

## `vue/no-multi-spaces`

See [Template Formatting: `no-multi-spaces`](./vue-formatting.md#vueno-multi-spaces).

## `vue/prefer-props-shorthand`

See [Template Formatting: `prefer-props-shorthand`](./vue-formatting.md#vueprefer-props-shorthand).

## `vue/prop-name-casing`

See [Template Formatting: `prop-name-casing`](./vue-formatting.md#vueprop-name-casing).

## `vue/valid-attribute-name`

See [Template Formatting: `valid-attribute-name`](./vue-formatting.md#vuevalid-attribute-name).

## `vue/no-preprocessor-lang`

See [SFC Blocks: `no-preprocessor-lang`](./vue-sfc.md#vueno-preprocessor-lang).

## `vue/no-script-non-standard-lang`

See [SFC Blocks: `no-script-non-standard-lang`](./vue-sfc.md#vueno-script-non-standard-lang).

## `vue/no-src-attribute`

See [SFC Blocks: `no-src-attribute`](./vue-sfc.md#vueno-src-attribute).

## `vue/no-template-lang`

See [SFC Blocks: `no-template-lang`](./vue-sfc.md#vueno-template-lang).

## `vue/require-scoped-style`

See [SFC Blocks: `require-scoped-style`](./vue-sfc.md#vuerequire-scoped-style).

## `vue/sfc-element-order`

See [SFC Blocks: `sfc-element-order`](./vue-sfc.md#vuesfc-element-order).

## `vue/single-style-block`

See [SFC Blocks: `single-style-block`](./vue-sfc.md#vuesingle-style-block).

## `vue/warn-custom-block`

See [SFC Blocks: `warn-custom-block`](./vue-sfc.md#vuewarn-custom-block).

## `vue/scoped-event-names`

See [Directive Conventions: `scoped-event-names`](./vue-directives.md#vuescoped-event-names).

## `vue/use-v-on-exact`

See [Directive Conventions: `use-v-on-exact`](./vue-directives.md#vueuse-v-on-exact).

## `vue/v-bind-style`

See [Directive Conventions: `v-bind-style`](./vue-directives.md#vuev-bind-style).

## `vue/v-on-style`

See [Directive Conventions: `v-on-style`](./vue-directives.md#vuev-on-style).

## `vue/v-slot-style`

See [Directive Conventions: `v-slot-style`](./vue-directives.md#vuev-slot-style).

## `vue/warn-custom-directive`

See [Directive Conventions: `warn-custom-directive`](./vue-directives.md#vuewarn-custom-directive).

## `vue/valid-v-bind`

See [Directive Validity: `valid-v-bind`](./vue-directive-validity.md#vuevalid-v-bind).

## `vue/valid-v-else`

See [Directive Validity: `valid-v-else`](./vue-directive-validity.md#vuevalid-v-else).

## `vue/valid-v-for`

See [Directive Validity: `valid-v-for`](./vue-directive-validity.md#vuevalid-v-for).

## `vue/valid-v-if`

See [Directive Validity: `valid-v-if`](./vue-directive-validity.md#vuevalid-v-if).

## `vue/valid-v-memo`

See [Directive Validity: `valid-v-memo`](./vue-directive-validity.md#vuevalid-v-memo).

## `vue/valid-v-model`

See [Directive Validity: `valid-v-model`](./vue-directive-validity.md#vuevalid-v-model).

## `vue/valid-v-on`

See [Directive Validity: `valid-v-on`](./vue-directive-validity.md#vuevalid-v-on).

## `vue/valid-v-show`

See [Directive Validity: `valid-v-show`](./vue-directive-validity.md#vuevalid-v-show).

## `vue/valid-v-slot`

See [Directive Validity: `valid-v-slot`](./vue-directive-validity.md#vuevalid-v-slot).
