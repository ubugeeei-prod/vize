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
| `vue/multi-word-component-names` | `{ ignores?: string[] }` | Defaults to an empty list in addition to the built-in `App` and Vue component exemptions. Names are exact and unique; PascalCase entries also admit their kebab-case spelling. |
| `vue/no-use-v-if-with-v-for` | `{ allowUsingIterationVar?: boolean }` | Defaults to `false`. `true` permits `v-if` only when its condition references a sibling `v-for` iteration variable; unrelated conditions are still reported. |
| `vue/enforce-style-attribute` | `{ allow?: Array<"plain" \| "scoped" \| "module"> }` | Defaults to `["scoped"]`. Lists the accepted style-block modes. |
| `vue/no-restricted-syntax` | `Array<string \| { selector: string, message?: string }>` or `{ selectors?: ... }` | Defaults to `[]`. The eslint-plugin-vue option form: esquery selectors over the vue-eslint-parser template AST (`VElement`, `VText`, `VExpressionContainer`, `VAttribute`, … and the TS-ESTree nodes of every expression), each with an optional message. See [the supported selector subset](#vue-no-restricted-syntax-selectors); a selector outside it fails configuration. |
| `vue/quote-props` | `"as-needed"` | Rejects unnecessary quoted object keys in retained template expressions. |
| `vue/sort-keys` | `{ order?: "asc" \| "desc"; caseSensitive?: boolean; natural?: boolean }` | Defaults to ascending, case-sensitive lexical order. Checks every object literal of the script blocks except a component options object itself, the children of `model` and the grandchildren of `computed`, `directives`, `inject`, `props` and `watch`, as eslint-plugin-vue does by default; `natural` uses its natural-compare order. |
| `vue/warn-custom-directive` | `{ ignore?: string[] }` | Defaults to an empty list. Exact directive names in `ignore` are treated as registered. |
| `vue/no-bare-strings-in-template` | `{ allowlist?: string[] }` | Omitted uses the upstream default allowlist; an explicit list replaces it. Entries are literals or `/pattern/flags` values using Rust regex syntax; invalid patterns remain literals. |
| `a11y/anchor-has-content` | `{ components?: string[]; accessibleChildren?: string[]; accessibleDirectives?: string[] }` | Each list defaults to empty. `components` are checked as anchors, `accessibleChildren` count as content, and an element carrying one of `accessibleDirectives` (named without `v-`) has content. See [content options](#a11y-content-options). |
| `a11y/heading-has-content` | `{ components?: string[]; accessibleChildren?: string[]; accessibleDirectives?: string[] }` | The same options as `a11y/anchor-has-content`, for `<h1>` to `<h6>`. |
| `a11y/label-has-for` | `{ controlComponents?: string[] }` | Defaults to an empty list. Named components count as label controls. |
| `vue/require-component-registration` | `{ globals?: string[]; ignorePatterns?: string[] }` | Explicit application/preview component names; accepts PascalCase and kebab-case spellings. `ignorePatterns` takes regular expressions, as in eslint-plugin-vue `vue/no-undef-components`. Options do not enable the rule. |
| `vue/sfc-element-order` | `{ order?: Array<string \| string[]> }` | Defaults to `[["script", "template"], "style"]`. A nested array means any of those block selectors may appear at that rank. |
| `vue/html-self-closing` | `{ html?: { void?: Style; normal?: Style; component?: Style }; svg?: Style; math?: Style }`, where `Style` is `"always"`, `"never"`, or `"any"` | Defaults match eslint-plugin-vue: `html.void: "never"`, `html.normal: "always"`, `html.component: "always"`, `svg: "always"`, and `math: "always"`. |
| `vue/html-button-has-type` | `{ button?: boolean; submit?: boolean; reset?: boolean }` | All three values default to `true`. Setting one to `false` reports that otherwise valid static button type as forbidden. |
| `vue/html-quotes` | `"double" \| "single"` or `{ style?: "double" \| "single"; avoidEscape?: boolean }` | Defaults to double quotes and `avoidEscape: false`. With `avoidEscape: true`, a quoted value containing the requested delimiter is left unchanged. |
| `vue/mustache-interpolation-spacing` | `"always" \| "never"` | Defaults to `"always"`. Requires or forbids whitespace immediately inside `{{` and `}}`; both sides are fixed independently. |
| `vue/v-on-event-hyphenation` | `"always" \| "never" \| { style?: "always" \| "never"; ignore?: string[] }` | Defaults to `always` and an empty ignore list. Exempts event names containing a configured case-sensitive substring. |
| `vue/v-on-handler-style` | `Style \| { style?: Style; ignoreIncludesComment?: boolean }`, where `Style` is `"method"`, `"inline"`, `"inline-function"`, `["method", "inline"]` or `["method", "inline-function"]` | Defaults to `"method"`: only a method reference. The other values are eslint-plugin-vue's; its default is `["method", "inline-function"]`. See [handler styles](./options-events.md#vue-v-on-handler-style). |
| `vue/v-bind-style` | `"shorthand" \| "longform"` | Defaults to `"shorthand"`. Configures whether binding directives use `:class` or `v-bind:class`; longform rewrites `.title` to `v-bind:title.prop`, preserving explicit modifiers. Object syntax such as `v-bind="attrs"` is unchanged. |
| `vue/v-on-style` | `"shorthand" \| "longform"` | Defaults to `"shorthand"`. Configures whether event directives use `@click` or `v-on:click`; object syntax such as `v-on="listeners"` is unchanged. |
| `vue/v-slot-style` | `"shorthand" \| "longform"` or `{ atComponent?: SlotStyle; default?: SlotStyle; named?: "shorthand" \| "longform" }`, where `SlotStyle` also accepts `"v-slot"` | Defaults to `atComponent: "v-slot"`, `default: "shorthand"`, and `named: "shorthand"`. Bare `v-slot` is rejected for `named` because it would erase the slot name. |
| `vue/attribute-hyphenation` | `"always" \| "never"` or `{ style?, ignore? }` | Configures whether component prop attributes should be hyphenated in templates. See [attribute options](./options-attributes.md). |
| `vue/valid-v-slot` | `{ allowModifiers?: boolean }` | Defaults to `false`. Permit static and dynamic named-slot modifiers with `true`; argless modifiers and references to the slot's own parameters remain invalid. |
| `musea/prefer-design-tokens` | `{ tokens?: Array<{ path: string; value: string; tier?: string }> }` | Off unless token data is configured and the rule is enabled or implicitly selected by a non-empty token list. `tier` defaults to `primitive`. |
| `type/strict-boolean-expressions` | `{ allowString?: boolean; allowNumber?: boolean; allowNullableObject?: boolean; allowNullableBoolean?: boolean; allowNullableString?: boolean; allowNullableNumber?: boolean; allowNullableEnum?: boolean; allowAny?: boolean }` | `allowString`, `allowNumber`, and `allowNullableObject` default to `true`; the other options default to `false`. Explicit rule enablement is required. |
| `type/no-confusing-void-expression` | `{ ignoreArrowShorthand?: boolean; ignoreVoidOperator?: boolean; ignoreVoidReturningFunctions?: boolean }` | All three default to `false`, as in typescript-eslint. `ignoreArrowShorthand` allows `() => voidCall()`. |
| `type/no-unnecessary-condition` | `{ allowConstantLoopConditions?: boolean \| "always" \| "never" \| "only-allowed-literals"; checkTypePredicates?: boolean; allowRuleToRunWithoutStrictNullChecksIKnowWhatIAmDoing?: boolean }` | Defaults to `"never"` and `false`, as in typescript-eslint. `"only-allowed-literals"` allows `true`, `false`, `1` and `0` as loop conditions; `true` means `"always"`. |
| `type/restrict-plus-operands` | `{ allowAny?: boolean; allowBoolean?: boolean; allowNullish?: boolean; allowNumberAndString?: boolean; allowRegExp?: boolean; skipCompoundAssignments?: boolean }` | The `allow*` options default to `true` and `skipCompoundAssignments` to `false`, as in typescript-eslint. typescript-eslint's `strict-type-checked` sets every `allow*` option to `false`. |
| `type/restrict-template-expressions` | `{ allowAny?: boolean; allowArray?: boolean; allowBoolean?: boolean; allowNullish?: boolean; allowNumber?: boolean; allowRegExp?: boolean; allowNever?: boolean; allow?: Array<string \| { from: "file" \| "lib" \| "package"; name: string \| string[]; path?: string; package?: string }> }` | `allowAny`, `allowBoolean`, `allowNullish`, `allowNumber` and `allowRegExp` default to `true`; `allowArray` and `allowNever` to `false`; `allow` to `Error`, `URL` and `URLSearchParams` from the default library, as in typescript-eslint. |
| `type/return-await` | `"in-try-catch" \| "always" \| "error-handling-correctness-only" \| "never"` | Defaults to `"in-try-catch"`, as in typescript-eslint. `"error-handling-correctness-only"`, which `strict-type-checked` uses, checks only error-handling contexts. |
| `type/await-thenable` | `{ allowMixedPromiseArrays?: boolean }` | Defaults to `false`, as in typescript-eslint. |
| `type/no-base-to-string` | `{ checkUnknown?: boolean; ignoredTypeNames?: string[] }` | Defaults to `false` and `["Error", "RegExp", "URL", "URLSearchParams"]`, as in typescript-eslint. |
| `type/no-deprecated` | `{ allow?: TypeSpecifier[] }` | Defaults to `[]`. Each entry is a type or value specifier (`string \| { from: "file" \| "lib" \| "package"; name: string \| string[]; path?: string; package?: string }`), as in typescript-eslint. |
| `type/no-duplicate-type-constituents` | `{ ignoreIntersections?: boolean; ignoreUnions?: boolean }` | Both default to `false`, as in typescript-eslint. |
| `type/no-floating-promises` | `{ allowForKnownSafeCalls?: TypeSpecifier[]; allowForKnownSafePromises?: TypeSpecifier[]; checkThenables?: boolean; ignoreIIFE?: boolean; ignoreVoid?: boolean }` | Defaults to `[]`, `[]`, `false`, `false` and `true`, as in typescript-eslint. |
| `type/no-meaningless-void-operator` | `{ checkNever?: boolean }` | Defaults to `false`, as in typescript-eslint. |
| `type/no-misused-promises` | `{ checksConditionals?: boolean \| { flagUnions?: "all" \| "strict" \| "none" }; checksSpreads?: boolean; checksVoidReturn?: boolean \| { arguments?, attributes?, inheritedMethods?, properties?, returns?, variables?: boolean } }` | Every check defaults to `true`, as in typescript-eslint. The object form of `checksVoidReturn` turns single positions off. |
| `type/no-misused-spread` | `{ allow?: TypeSpecifier[] }` | Defaults to `[]`, as in typescript-eslint. |
| `type/no-unnecessary-boolean-literal-compare` | `{ allowComparingNullableBooleansToFalse?: boolean; allowComparingNullableBooleansToTrue?: boolean }` | Both default to `true`, as in typescript-eslint. |
| `type/no-unnecessary-type-assertion` | `{ checkLiteralConstAssertions?: boolean; typesToIgnore?: string[] }` | Defaults to `false` and `[]`, as in typescript-eslint. |
| `type/no-unsafe-member-access` | `{ allowOptionalChaining?: boolean }` | Defaults to `false`, as in typescript-eslint. |
| `type/only-throw-error` | `{ allow?: TypeSpecifier[]; allowRethrowing?: boolean; allowThrowingAny?: boolean; allowThrowingUnknown?: boolean }` | Defaults to `[]` and `true` for the three switches, as in typescript-eslint. |
| `type/prefer-promise-reject-errors` | `{ allow?: TypeSpecifier[]; allowEmptyReject?: boolean; allowThrowingAny?: boolean; allowThrowingUnknown?: boolean }` | Defaults to `[]` and `false` for the three switches, as in typescript-eslint. |
| `type/unbound-method` | `{ ignoreStatic?: boolean }` | Defaults to `false`, as in typescript-eslint. |

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

<span id="a11y-content-options"></span>

## `a11y/anchor-has-content`

The options follow eslint-plugin-vuejs-accessibility. `components` lists components checked as
anchors, such as a router link. `accessibleChildren` lists components that always render
accessible content. `accessibleDirectives` lists custom directives, named without `v-`, that fill
the element they sit on; the element itself or a descendant may carry one, with or without a value.
`v-text` and `v-html` with a value always count. PascalCase, camelCase and kebab-case spellings
are equivalent. A child hidden with `aria-hidden="true"` does not count, even when listed.

```ts
const ruleOptions = {
  "a11y/anchor-has-content": {
    components: ["NuxtLink"],
    accessibleChildren: ["RetryImage"],
    accessibleDirectives: ["markup-inline"],
  },
};
```

Bad: `<NuxtLink to="/docs" />` and `<a href="/retry"><UnknownImage /></a>`
Good: `<a href="/docs" v-markup-inline="label" />` and `<a href="/retry"><RetryImage /></a>`

## `a11y/heading-has-content`

The same three options apply to `<h1>` to `<h6>` and `<component is="h2">`. A heading filled by a
custom directive is not empty, and the rule cannot tell that without `accessibleDirectives`.

```ts
const ruleOptions = {
  "a11y/heading-has-content": {
    components: ["SectionTitle"],
    accessibleDirectives: ["markup-inline"],
  },
};
```

Bad: `<h4 />` and `<SectionTitle />`
Good: `<h2 v-markup-inline="text" />` and `<h3 v-text="text" />`

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

## `vue/multi-word-component-names`

List single-word component names that are intentionally part of the project's public vocabulary.
The list extends the built-in `App` and Vue component exemptions and does not enable the rule.
Matching is exact; a PascalCase entry also permits its kebab-case spelling. The upstream rule is
not fixable, so an unlisted name must be renamed or explicitly configured.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/multi-word-component-names": "error" },
    ruleOptions: {
      "vue/multi-word-component-names": { ignores: ["Item"] },
    },
  },
});
```

Bad with this policy: `Thing.vue`

```vue
<template><p>Thing</p></template>
```

Good with this policy: `Item.vue`

```vue
<template><p>Item</p></template>
```

## `vue/no-use-v-if-with-v-for`

Set `allowUsingIterationVar` to `true` when a project intentionally filters a loop item with a
same-element `v-if`. Conditions that do not reference a sibling `v-for` variable remain invalid,
and the option does not enable the rule. The upstream rule is not fixable, so this policy produces
diagnostics without an automatic rewrite.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-use-v-if-with-v-for": "warn" },
    ruleOptions: {
      "vue/no-use-v-if-with-v-for": {
        allowUsingIterationVar: true,
      },
    },
  },
});
```

Bad with this policy:

```vue
<li v-for="item in items" v-if="showAll">{{ item.name }}</li>
```

Good with this policy:

```vue
<li v-for="item in items" v-if="item.visible">{{ item.name }}</li>
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

For a family of globally registered components, `ignorePatterns` takes regular
expressions, as eslint-plugin-vue `vue/no-undef-components` does: a tag is skipped
when a pattern matches its authored name, its kebab-case form or its PascalCase
form. `ignorePatterns: ["^C[A-Z]"]` skips `<CButton>` and `<c-toast>` and still
reports `<Card>`. Patterns use Rust `regex` syntax; one that does not compile
matches nothing.

Bad (unlisted component):

```vue
<MissingWidget />
```

Good (a configured global):

```vue
<MyButton />
```

## `vue/no-restricted-syntax` selectors {#vue-no-restricted-syntax-selectors}

Patina matches a subset of [esquery](https://github.com/estools/esquery):

- node types (case-insensitive) and `*`;
- attributes: `[attr]`, `[a.b.c=value]`, `!=`, `<`, `<=`, `>`, `>=`, with quoted, numeric, bare or `/regex/flags` values (`i`, `m`, `s`, `u` flags; Rust regex syntax);
- fields: `.left`, `.a.b`;
- combinators: descendant (space), `>`, `~`, `+`, and comma-separated lists;
- `:matches()`, `:is()`, `:not()`, `:has()` (descendants), `:first-child`, `:last-child`, `:nth-child(n)`, `:nth-last-child(n)`, `:statement`, `:expression`, `:declaration`, `:function` and `:pattern`.

Not supported: `:nth-child(an+b)` formulas, `:has(> …)` relative selectors, `[attr=type(...)]` and the `:exit` suffix. The template is `VElement[name='template']` with `VStartTag` / `VAttribute` / `VDirectiveKey` / `VIdentifier` / `VLiteral` / `VExpressionContainer` / `VText` children as vue-eslint-parser builds them; `VText` holds the authored text between elements and interpolations, and element names are lowercased (`rawName` keeps the authored case). Expressions are TS-ESTree without parentheses, inside `VOnExpression`, `VForExpression` or `VSlotScopeExpression` where vue-eslint-parser uses them.
