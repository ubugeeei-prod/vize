---
title: Rule Options
---

# Rule Options

With Vite+, configure `linter.ruleOptions` inside `withVue` in `vite.config.ts` and run
`vp run lint`. The standalone compatibility config uses the same values under
`linter.ruleOptions`.

`linter.ruleOptions` holds typed project-local settings for rules that accept options. Unknown
option fields are rejected, and later matching config entries replace the full option object for the
same rule. Set severity separately under `linter.rules`; options only describe how an enabled rule
should behave unless noted below.

Use the [native lint execution guide](../guide/lint-execution.md) to select rules and their severity
before adjusting the options below.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: {
      "script/no-restricted-globals": "error",
      "vue/html-self-closing": "warn",
      "musea/prefer-design-tokens": "warn",
    },
    ruleOptions: {
      "script/no-restricted-globals": {
        globals: [
          {
            name: "process",
            message: "Read env via a typed helper.",
          },
          {
            name: "alert",
          },
        ],
      },
      "vue/html-self-closing": {
        html: {
          void: "always",
          normal: "never",
          component: "always",
        },
        svg: "always",
        math: "always",
      },
      "musea/prefer-design-tokens": {
        tokens: [
          {
            path: "color.primary",
            value: "#3b82f6",
            tier: "semantic",
          },
        ],
      },
    },
  },
});
```

| Rule | Option shape | Defaults and behavior |
| --- | --- | --- |
| `html/no-empty-palpable-content` | `{ contentDirectives?: string[] }` | Defaults to an empty list. Exact bare names identify directives that supply visible content. Rule enablement remains separate. |
| `script/no-restricted-globals` | `{ globals?: Array<{ name: string; message?: string }> }` | Without options, the built-in deny list is `process`, `localStorage`, and `sessionStorage`. A non-empty `globals` list replaces that built-in list. |
| `script/no-restricted-members` | `{ members?: Array<{ object: string; property: string; message?: string }> }` | Off unless `members` is configured and the rule is enabled. A missing `message` uses the generic diagnostic help. |
| `script/define-macros-order` | `{ order?: string[]; defineExposeLast?: boolean }` | Defaults to `["defineProps", "defineEmits"]` and `false`. Listed macros appear first after imports and type declarations. With `defineExposeLast: true`, the last direct `defineExpose` must be the final statement in `<script setup>`; omit it from `order`. This finding requires manual repair. |
| `vue/valid-v-on` | `{ modifiers?: string[] }` | Defaults to an empty list. Adds exact, case-sensitive custom names to the built-in modifiers; it does not enable the rule or permit a missing handler. Scoped entries replace the inherited list, and `[]` clears it. See the [complete modifier example](./reference/vue-valid-v-on.md). |
| `vue/component-name-in-template-casing` | `{ casing?: "PascalCase" \| "kebab-case", registeredComponentsOnly?: boolean, globals?: string[] }` | Defaults to `PascalCase`, registered components only, and no literal globals. |
| `script/custom-event-name-casing` | `{ casing?: "lowercase" \| "camelCase" \| "kebab-case" }` | Defaults to `camelCase`. |
| `vue/prop-name-casing` | `{ casing?: "camelCase" \| "snake_case" }` | Defaults to `camelCase`. Checks declared prop names; options do not enable the rule or rename its public API. |
| `vue/slot-name-casing` | `"camelCase" \| "kebab-case" \| "singleword"` | Defaults to `camelCase`. Checks static names on rendered `<slot>` outlets; provider-side `v-slot` and `#name` syntax is outside this rule. |
| `script/define-props-destructuring` | `{ destructure?: "only-when-assigned" \| "always" \| "never" }` | Defaults to `always`, checking assigned and bare typed calls. `only-when-assigned` skips bare calls; `never` requires props-object access. Vue 3.5 destructuring preserves reactivity. |
| `vue/no-mutating-props` | `{ shallowOnly?: boolean }` | Defaults to `false`; `true` allows nested mutation while still disallowing direct prop reassignment. |
| `vue/enforce-style-attribute` | `{ allow?: Array<"plain" \| "scoped" \| "module"> }` | Defaults to `["scoped"]`. Lists the accepted style-block modes. |
| `vue/no-restricted-syntax` | `{ selectors?: Array<"ArrayExpression" \| "BinaryExpression" \| "CallExpression" \| "ConditionalExpression" \| "ObjectExpression" \| "UnaryExpression"> }` | Defaults to `[]`. Only exact supported ESTree node types are accepted; compound esquery selectors fail configuration. |
| `vue/quote-props` | `"as-needed"` | Rejects unnecessary quoted object keys in retained template expressions. |
| `vue/sort-keys` | `{ order?: "asc" \| "desc"; caseSensitive?: boolean; natural?: boolean }` | Defaults to ascending, case-sensitive lexical order for supported nested component option objects. |
| `vue/warn-custom-directive` | `{ ignore?: string[] }` | Defaults to an empty list. Exact directive names in `ignore` are treated as registered. |
| `vue/no-bare-strings-in-template` | `{ allowlist?: string[] }` | Omitted uses the upstream default allowlist; an explicit list replaces it. Entries are literals or `/pattern/flags` values using Rust regex syntax; invalid patterns remain literals. |
| `a11y/anchor-has-content` | `{ accessibleChildren?: string[] }` | Defaults to an empty list. Named components count as accessible anchor content. |
| `a11y/label-has-for` | `{ controlComponents?: string[] }` | Defaults to an empty list. Named components count as label controls. |
| `vue/require-component-registration` | `{ globals?: string[] }` | Explicit application/preview component names; accepts PascalCase and kebab-case spellings. Options do not enable the rule. |
| `vue/sfc-element-order` | `{ order?: Array<string \| string[]> }` | Defaults to `[["script", "template"], "style"]`. A nested array means any of those block selectors may appear at that rank. |
| `vue/html-self-closing` | `{ html?: { void?: Style; normal?: Style; component?: Style }; svg?: Style; math?: Style }`, where `Style` is `"always"`, `"never"`, or `"any"` | Defaults match eslint-plugin-vue: `html.void: "never"`, `html.normal: "always"`, `html.component: "always"`, `svg: "always"`, and `math: "always"`. |
| `vue/html-button-has-type` | `{ button?: boolean; submit?: boolean; reset?: boolean }` | All three values default to `true`. Setting one to `false` reports that otherwise valid static button type as forbidden. |
| `vue/html-quotes` | `"double" \| "single"` or `{ style?: "double" \| "single"; avoidEscape?: boolean }` | Defaults to double quotes and `avoidEscape: false`. With `avoidEscape: true`, a quoted value containing the requested delimiter is left unchanged. |
| `vue/mustache-interpolation-spacing` | `"always" \| "never"` | Defaults to `"always"`. Requires or forbids whitespace immediately inside `{{` and `}}`; both sides are fixed independently. |
| `vue/v-on-event-hyphenation` | `"always" \| "never" \| { style?: "always" \| "never"; ignore?: string[] }` | Defaults to `always` and an empty ignore list. Exempts event names containing a configured case-sensitive substring. |
| `vue/v-bind-style` | `"shorthand" \| "longform"` | Defaults to `"shorthand"`. Configures whether binding directives use `:class` or `v-bind:class`; longform rewrites `.title` to `v-bind:title.prop`, preserving explicit modifiers. Object syntax such as `v-bind="attrs"` is unchanged. |
| `vue/v-on-style` | `"shorthand" \| "longform"` | Defaults to `"shorthand"`. Configures whether event directives use `@click` or `v-on:click`; object syntax such as `v-on="listeners"` is unchanged. |
| `vue/v-slot-style` | `"shorthand" \| "longform"` or `{ atComponent?: SlotStyle; default?: SlotStyle; named?: "shorthand" \| "longform" }`, where `SlotStyle` also accepts `"v-slot"` | Defaults to `atComponent: "v-slot"`, `default: "shorthand"`, and `named: "shorthand"`. Bare `v-slot` is rejected for `named` because it would erase the slot name. |
| `vue/attribute-hyphenation` | `"always" \| "never"` or `{ style?, ignore? }` | Configures whether component prop attributes should be hyphenated in templates. See [attribute options](./options-attributes.md). |
| `vue/valid-v-slot` | `{ allowModifiers?: boolean }` | Defaults to `false`. Permit static and dynamic named-slot modifiers with `true`; argless modifiers and references to the slot's own parameters remain invalid. |
| `musea/prefer-design-tokens` | `{ tokens?: Array<{ path: string; value: string; tier?: string }> }` | Off unless token data is configured and the rule is enabled or implicitly selected by a non-empty token list. `tier` defaults to `primitive`. |
| `type/strict-boolean-expressions` | `{ allowString?: boolean; allowNumber?: boolean; allowNullableObject?: boolean; allowNullableBoolean?: boolean; allowNullableString?: boolean; allowNullableNumber?: boolean; allowNullableEnum?: boolean; allowAny?: boolean }` | `allowString`, `allowNumber`, and `allowNullableObject` default to `true`; the other options default to `false`. Explicit rule enablement is required. |

## Detailed Examples

- [Type Rule Options](./options-type.md): strict boolean conditions and nullable allowances.
- [Script Rule Options](./options-script.md): restricted globals, restricted members, and custom
  event casing.
- [Vue Rule Options](./options-vue.md): component casing, prop mutation depth, SFC block order,
  self-closing tags, event names, and attribute casing.
- [Musea Rule Options](./options-musea.md): design-token inventory configuration.

## `html/no-empty-palpable-content`

Custom directives may focus, style, or fill an element. List only the directives your project
knows provide visible content. Use the exact template name without `v-`; arguments and modifiers
do not change that name. Unlisted directives still leave an empty element reportable. An empty
list restores the default behavior, and a matching scoped entry replaces the entire option object.
This setting does not validate sanitization or enable the rule.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: {
      "html/no-empty-palpable-content": "error",
    },
    ruleOptions: {
      "html/no-empty-palpable-content": {
        contentDirectives: ["safe-html"],
      },
    },
  },
});
```

Bad (unlisted content source):

```vue
<p v-sanitize="message" />
```

Good (configured provider):

```vue
<p v-safe-html="message" />
```

## Scoped Entries

When `entries` match a file, their `linter.ruleOptions` overlay the root options for that file.
Each configured rule option object replaces the same root object; arrays such as `globals`,
`members`, `order`, and `tokens` are not concatenated across entries.

## `vue/html-button-has-type`

Use the three booleans to restrict which valid static button types a project permits. Missing,
empty, and invalid types are always reported, while a populated dynamic `:type` binding remains
valid because its value cannot be checked statically. The upstream rule is not fixable, so this
policy produces diagnostics without an automatic rewrite.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/html-button-has-type": "warn" },
    ruleOptions: {
      "vue/html-button-has-type": {
        button: true,
        submit: false,
        reset: false,
      },
    },
  },
});
```

Bad with this policy:

```vue
<button type="submit">Save</button>
```

Good with this policy:

```vue
<button type="button">Cancel</button>
```

## `vue/require-component-registration`

For components supplied by application plugins or Musea `previewSetup`, configure
`vue/require-component-registration` with explicit global names:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: {
      "vue/require-component-registration": "warn",
    },
    ruleOptions: {
      "vue/require-component-registration": {
        globals: ["MyButton", "MyIcon"],
      },
    },
  },
});
```

A name permits its PascalCase and kebab-case template spellings. The list uses
component names; regular expressions are not interpreted. The same options apply
to regular SFCs and Musea Art variants. Options alone do not enable the rule, and
unknown components continue to be reported. A later config layer replaces the
list; an empty list clears an inherited list.

Bad (unlisted component):

```vue
<MissingWidget />
```

Good (a configured global):

```vue
<MyButton />
```
