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
| `vue/component-name-in-template-casing` | `{ casing?: "PascalCase" \| "kebab-case", registeredComponentsOnly?: boolean, globals?: string[], ignores?: string[] }` | Defaults to `PascalCase`, registered components only, no literal globals and no ignored names. `ignores` leaves a tag alone whether or not it is registered; an entry is the exact authored tag name or a `/pattern/flags` value using Rust regex syntax. |
| `script/custom-event-name-casing` | `{ casing?: "lowercase" \| "camelCase" \| "kebab-case" }` | Defaults to `camelCase`. |
| `script/consistent-type-imports` | `{ prefer?: "type-imports" \| "no-type-imports"; fixStyle?: "separate-type-imports" \| "inline-type-imports"; disallowTypeAnnotations?: boolean; emitDecoratorMetadata?: boolean }` | Defaults to `type-imports`, `separate-type-imports`, `true` and `true`. The first three are the typescript-eslint options. `emitDecoratorMetadata` says whether the project emits decorator metadata, which lint does not read from `tsconfig.json`; with `true`, a type that a decorated signature serializes counts as a value use. |
| `vue/component-definition-name-casing` | `"PascalCase" \| "kebab-case"` | Defaults to `"PascalCase"`, the eslint-plugin-vue option. Checks the static `name` of a component definition and the first argument of `app.component()`; the file name is `vue/component-file-name-casing`. Options do not enable the rule. |
| `vue/prop-name-casing` | `{ casing?: "camelCase" \| "snake_case" }` | Defaults to `camelCase`. Checks declared prop names; options do not enable the rule or rename its public API. |
| `vue/slot-name-casing` | `"camelCase" \| "kebab-case" \| "singleword"` | Defaults to `camelCase`. Checks static names on rendered `<slot>` outlets; provider-side `v-slot` and `#name` syntax is outside this rule. |
| `script/define-props-destructuring` | `{ destructure?: "only-when-assigned" \| "always" \| "never" }` | Defaults to `always`, checking assigned and bare typed calls. `only-when-assigned` skips bare calls; `never` requires props-object access. Vue 3.5 destructuring preserves reactivity. |
| `vue/no-mutating-props` | `{ shallowOnly?: boolean }` | Defaults to `false`; `true` allows nested mutation while still disallowing direct prop reassignment. |
| `vue/multi-word-component-names` | `{ ignores?: string[] }` | Defaults to an empty list in addition to the built-in `App` and Vue component exemptions. Names are exact and unique; PascalCase entries also admit their kebab-case spelling. |
| `vue/no-use-v-if-with-v-for` | `{ allowUsingIterationVar?: boolean }` | Defaults to `false`. `true` permits `v-if` only when its condition references a sibling `v-for` iteration variable; unrelated conditions are still reported. |
| `vue/enforce-style-attribute` | `{ allow?: Array<"plain" \| "scoped" \| "module"> }` | Defaults to `["scoped"]`. Lists the accepted style-block modes. |
| `vue/no-restricted-syntax` | `Array<string \| { selector: string, message?: string }>` or `{ selectors?: ... }` | Defaults to `[]`. The eslint-plugin-vue option form: esquery selectors over the vue-eslint-parser template AST (`VElement`, `VText`, `VExpressionContainer`, `VAttribute`, … and the TS-ESTree nodes of every expression), each with an optional message. See [the supported selector subset](#vue-no-restricted-syntax-selectors); a selector outside it fails configuration. |
| `vue/no-restricted-html-elements` | `string \| Array<string \| { element: string \| string[]; message?: string }>` | Checks nothing until configured. Reports each named native element. An entry's `message` replaces the default `Unexpected use of forbidden element <name>.`; the first entry that names an element decides its message. See [restricted elements](#vue-no-restricted-html-elements). |
| `vue/no-restricted-v-on` | `string \| Array<string \| { argument: string; modifiers?: string[]; element?: string; message?: string }>` | Checks nothing until configured. A name restricts every listener of that event. An object entry restricts the listeners that carry all of its `modifiers`, on the `element` it names if any, and may set its own `message`. Names are literal. See [restricted listeners](#vue-no-restricted-v-on). |
| `vue/quote-props` | `"as-needed"` | Rejects unnecessary quoted object keys in retained template expressions. |
| `vue/sort-keys` | `{ order?: "asc" \| "desc"; caseSensitive?: boolean; natural?: boolean }` | Defaults to ascending, case-sensitive lexical order. Checks every object literal of the script blocks except a component options object itself, the children of `model` and the grandchildren of `computed`, `directives`, `inject`, `props` and `watch`, as eslint-plugin-vue does by default; `natural` uses its natural-compare order. |
| `vue/warn-custom-directive` | `{ ignore?: string[] }` | Defaults to an empty list. Exact directive names in `ignore` are treated as registered. |
| `vue/no-bare-strings-in-template` | `{ allowlist?: string[] }` | Omitted uses the upstream default allowlist; an explicit list replaces it. Entries are literals or `/pattern/flags` values using Rust regex syntax; invalid patterns remain literals. |
| `a11y/anchor-has-content` | `{ components?: string[]; accessibleChildren?: string[]; accessibleDirectives?: string[] }` | Each list defaults to empty. `components` are checked as anchors, `accessibleChildren` count as content, and an element carrying one of `accessibleDirectives` (named without `v-`) has content. See [content options](#a11y-content-options). |
| `a11y/heading-has-content` | `{ components?: string[]; accessibleChildren?: string[]; accessibleDirectives?: string[] }` | The same options as `a11y/anchor-has-content`, for `<h1>` to `<h6>`. |
| `a11y/label-has-for` | `{ controlComponents?: string[] }` | Defaults to an empty list. Named components count as label controls. |
| `vue/require-component-registration` | `{ globals?: string[]; ignorePatterns?: string[] }` | Explicit application/preview component names; accepts PascalCase and kebab-case spellings. `ignorePatterns` takes regular expressions, as in eslint-plugin-vue `vue/no-undef-components`. Options do not enable the rule. |
| `vue/no-undefined-refs` | `{ globals?: string[] }` | Defaults to an empty list. Exact names a build step defines for every template, such as `unplugin-auto-import` imports. In a Nuxt project, `vize lint` adds the auto-imports Nuxt generated, so they need no entry. Options do not enable the rule; scoped entries replace the inherited list, and `[]` clears it. See the [configuration example](./reference/vue-no-undefined-refs.md). |
| `vue/sfc-element-order` | `{ order?: Array<string \| string[]> }` | Defaults to `[["script", "template"], "style"]`. A nested array means any of those block selectors may appear at that rank. |
| `vue/html-self-closing` | `{ html?: { void?: Style; normal?: Style; component?: Style }; svg?: Style; math?: Style }`, where `Style` is `"always"`, `"never"`, or `"any"` | Defaults match eslint-plugin-vue: `html.void: "never"`, `html.normal: "always"`, `html.component: "always"`, `svg: "always"`, and `math: "always"`. |
| `vue/html-button-has-type` | `{ button?: boolean; submit?: boolean; reset?: boolean }` | All three values default to `true`. Setting one to `false` reports that otherwise valid static button type as forbidden. |
| `vue/html-quotes` | `"double" \| "single"` or `{ style?: "double" \| "single"; avoidEscape?: boolean }` | Defaults to double quotes and `avoidEscape: false`. With `avoidEscape: true`, a quoted value containing the requested delimiter is left unchanged. |
| `vue/mustache-interpolation-spacing` | `"always" \| "never"` | Defaults to `"always"`. Requires or forbids whitespace immediately inside `{{` and `}}`; both sides are fixed independently. |
| `vue/v-on-event-hyphenation` | `"always" \| "never" \| { style?: "always" \| "never"; ignore?: string[] }` | Defaults to `always` and an empty ignore list. Exempts event names containing a configured case-sensitive substring. |
| `vue/v-on-handler-style` | `Style \| { style?: Style; ignoreIncludesComment?: boolean }`, where `Style` is `"method"`, `"inline"`, `"inline-function"`, `["method", "inline"]` or `["method", "inline-function"]` | Defaults to `"method"`: only a method reference. The other values are eslint-plugin-vue's; its default is `["method", "inline-function"]`. See [handler styles](./options-events.md#vue-v-on-handler-style). |
| `vue/v-bind-style` | `"shorthand" \| "longform"` or `[style, { sameNameShorthand?: "always" \| "never" \| "ignore" }]` | Defaults to `"shorthand"`. Configures whether binding directives use `:class` or `v-bind:class`; longform rewrites `.title` to `v-bind:title.prop`, preserving explicit modifiers. Object syntax such as `v-bind="attrs"` is unchanged. The list is eslint-plugin-vue's form: `sameNameShorthand: "always"` requires `:title` where `:title="title"` is written, `"never"` requires the written value, and `"ignore"`, the default, checks neither. See [binding style](./options-vue.md#vue-v-bind-style). |
| `vue/v-on-style` | `"shorthand" \| "longform"` | Defaults to `"shorthand"`. Configures whether event directives use `@click` or `v-on:click`; object syntax such as `v-on="listeners"` is unchanged. |
| `vue/v-slot-style` | `"shorthand" \| "longform"` or `{ atComponent?: SlotStyle; default?: SlotStyle; named?: "shorthand" \| "longform" }`, where `SlotStyle` also accepts `"v-slot"` | Defaults to `atComponent: "v-slot"`, `default: "shorthand"`, and `named: "shorthand"`. Bare `v-slot` is rejected for `named` because it would erase the slot name. |
| `vue/attribute-hyphenation` | `"always" \| "never"` or `{ style?, ignore? }` | Configures whether component prop attributes should be hyphenated in templates. See [attribute options](./options-attributes.md). |
| `vue/valid-v-slot` | `{ allowModifiers?: boolean }` | Defaults to `false`. Permit static and dynamic named-slot modifiers with `true`; argless modifiers and references to the slot's own parameters remain invalid. |
| `vue/no-duplicate-attributes` | `{ allowCoexistClass?: boolean; allowCoexistStyle?: boolean }` | Both default to `true`, as in eslint-plugin-vue. `false` reports `:class` next to `class`, or `:style` next to `style`. See [template options](./options-template.md#vue-no-duplicate-attributes). |
| `vue/no-multiple-template-root` | `{ disallowComments?: boolean }` | Defaults to `false`. `true` also reports comments placed directly in the template root. |
| `vue/no-lone-template` | `{ ignoreAccessible?: boolean }` | Defaults to `false`. `true` accepts a nested `<template>` with an `id` or a `ref`. |
| `vue/no-static-inline-styles` | `{ allowBinding?: boolean }` | Defaults to `false`. `true` reports only the static `style` attribute, not static values in `:style`. |
| `vue/no-unused-components` | `{ ignoreWhenBindingPresent?: boolean }` | Defaults to `true`: a dynamic `is` binding suppresses the file's reports. `false` still checks the registrations. |
| `vue/no-v-html` | `{ ignorePattern?: string }` | No default. A `v-html` whose expression matches the Rust regex is not reported. See [template options](./options-template.md#vue-no-v-html). |
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
| `type/unbound-method` | `{ ignoreStatic?: boolean; checkGlobalObjectMethods?: boolean }` | Both default to `false`. `ignoreStatic` is as in typescript-eslint. Library methods read from the global object (`window.setTimeout`, `self.requestAnimationFrame`) are not reported, because platform operations use the global object when called without a receiver; `checkGlobalObjectMethods: true` reports them as typescript-eslint does. |

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

## `vue/component-definition-name-casing`

The option is eslint-plugin-vue's: `"PascalCase"` (the default) or `"kebab-case"`. It
applies to the static `name` of `export default {}`, `defineComponent({})` and the other
component definitions, to `defineOptions({ name })`, and to the first argument of
`app.component(name, component)`. The file name is checked by
`vue/component-file-name-casing`, which takes no option.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/component-definition-name-casing": "warn" },
    ruleOptions: { "vue/component-definition-name-casing": "kebab-case" },
  },
});
```

Bad with this config:

```vue
<script setup lang="ts">
defineOptions({ name: "MyComponent" });
</script>
```

Good with this config:

```vue
<script setup lang="ts">
defineOptions({ name: "my-component" });
</script>
```

A computed name is not checked. The rule does not rewrite the name, because renaming a
component changes how `<KeepAlive include>` and recursive self-references find it; the help
names the converted spelling. Options configure the rule without enabling it.

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

## `vue/no-restricted-html-elements`

Each entry is an element name or eslint-plugin-vue's `{ element, message }` object, whose
`element` is one name or a list. An entry without a `message` reports the default text.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-restricted-html-elements": "error" },
    ruleOptions: {
      "vue/no-restricted-html-elements": [
        "marquee",
        { element: ["a", "button"], message: "Use BaseLink or BaseButton." },
      ],
    },
  },
});
```

Bad with this policy: `<marquee>` reports `Unexpected use of forbidden element marquee.` and
`<a>` reports `Use BaseLink or BaseButton.`

```vue
<template><marquee>News</marquee><a href="/">Home</a></template>
```

Good with this policy:

```vue
<template><p>News</p><BaseLink to="/">Home</BaseLink></template>
```

## `vue/no-restricted-v-on`

Each entry is an event name or eslint-plugin-vue's `{ argument, modifiers, element, message }`
object. `modifiers` restricts only the listeners that carry every listed modifier, and `element`
only the listeners on that authored tag name. Names are literal, and a `null` argument is refused.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-restricted-v-on": "error" },
    ruleOptions: {
      "vue/no-restricted-v-on": [
        { argument: "click", modifiers: ["stop"], message: "Handle the click in the child." },
        { argument: "focus", element: "BaseInput" },
      ],
    },
  },
});
```

Bad with this policy: `@click.stop` reports the entry's message, and `@focus` on `<BaseInput>`
reports ``Using `@focus` is not allowed on this <BaseInput>.``

```vue
<template><div @click.stop="close"><BaseInput @focus="open" /></div></template>
```

Good with this policy:

```vue
<template><div @click="close"><input @focus="open" /></div></template>
```

## `vue/no-restricted-syntax` selectors {#vue-no-restricted-syntax-selectors}

Patina matches a subset of [esquery](https://github.com/estools/esquery):

- node types (case-insensitive) and `*`;
- attributes: `[attr]`, `[a.b.c=value]`, `!=`, `<`, `<=`, `>`, `>=`, with quoted, numeric, bare or `/regex/flags` values (`i`, `m`, `s`, `u` flags; Rust regex syntax);
- fields: `.left`, `.a.b`;
- combinators: descendant (space), `>`, `~`, `+`, and comma-separated lists;
- `:matches()`, `:is()`, `:not()`, `:has()`, `:first-child`, `:last-child`, `:nth-child(n)`, `:nth-last-child(n)`, `:statement`, `:expression`, `:declaration`, `:function` and `:pattern`.

A `:has()` argument without a leading combinator matches any descendant of the subject. An argument led by `>`, `~` or `+` is relative to the subject, as in CSS: `VElement:has(> VStartTag > VAttribute[key.name='flex-wrap'])` matches an element whose own start tag carries `flex-wrap`, `:has(> A > B)` needs a child `A` of the subject with a child `B`, and `:has(+ A)` / `:has(~ A)` look at the siblings after the subject. esquery 1.7 parses the same arguments but anchors only the last compound at the subject (`:has(> A > B)` matches a subject that is itself `A` with a child `B`) and never matches `:has(+ …)` or `:has(~ …)`.

Not supported: `:nth-child(an+b)` formulas, `[attr=type(...)]` and the `:exit` suffix. The template is `VElement[name='template']` with `VStartTag` / `VAttribute` / `VDirectiveKey` / `VIdentifier` / `VLiteral` / `VExpressionContainer` / `VText` children as vue-eslint-parser builds them; `VText` holds the authored text between elements and interpolations, and element names are lowercased (`rawName` keeps the authored case). Expressions are TS-ESTree without parentheses, inside `VOnExpression`, `VForExpression` or `VSlotScopeExpression` where vue-eslint-parser uses them.
