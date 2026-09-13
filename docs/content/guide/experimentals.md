---
title: Experimentals
---

# Experimentals

`experimentals` is the shared opt-in surface for proposed Vue syntax, tooling-only type checks, and backend routes that must stay off unless a project asks for them. It is separate from stable `compiler` options and from Vue runtime `features`: every key is disabled by default, every RFC flag is explicit, and stable options win when both surfaces can describe the same backend choice.

Use this page as the source of truth when enabling:

- Vue RFC [#823](https://github.com/vuejs/rfcs/pull/823) patterned templates
- Vue RFC [#831](https://github.com/vuejs/rfcs/pull/831) in-tag comments
- Vue RFC [#833](https://github.com/vuejs/rfcs/pull/833) `<Self>` component references
- Vue RFC [#734](https://github.com/vuejs/rfcs/pull/734) strict slot child checks
- Vize backend experiments such as server-script, SFC Vapor, and JSX Vapor routing

For RFC-specific examples, API entry points, flag-off behavior, and deferred syntax, see
[Vue RFC Experimental Details](./experimentals-vue-rfcs.md).

## Recommended Config

Prefer the recommended camelCase names in new TypeScript config:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  experimentals: {
    patternedTemplate: true, inTagComment: true,
    selfComponent: true, strictSlotChildren: true,
    serverScript: false, vapor: null, jsxVapor: {},
  },
});
```

For JSON and Pkl config, the same values are accepted. Use `{}` only when you want to enable the feature while leaving room for future nested options:

```json
{
  "experimentals": {
    "patternedTemplate": true,
    "inTagComment": true,
    "strictSlotChildren": {},
    "vapor": false
  }
}
```

```pkl
amends "node_modules/vize/pkl/vize.pkl"

experimentals {
  patternedTemplate = true
  inTagComment = true
  strictSlotChildren = new Mapping {}
  vapor = false
}
```

## Switch Values

Each key is a switch:

| Value | Meaning |
| --- | --- |
| omitted | Disabled |
| `false` | Disabled, including when overriding a shared opt-in |
| `null` | Disabled; useful for JSON config generated from nullable settings |
| `true` | Enabled |
| `{}` | Enabled, with an object shape reserved for future per-feature options |

Missing keys, `false`, and `null` are off. `true` and `{}` are on.

Typed TypeScript and Pkl config should use only `true`, `false`, `null`, or `{}`. The config loader treats other present non-`false` and non-`null` JSON values as enabled for compatibility, but that is not the documented public shape.

## Precedence

Shared `vize.config.*` values are the default for tools that load project config. Direct Vite plugin options win over shared config, including explicit opt-outs:

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      experimentals: {
        patternedTemplate: false,
        inTagComment: true,
      },
    }),
  ],
});
```

In this example, `patternedTemplate: false` disables a shared config opt-in for this plugin instance, while `inTagComment: true` enables the parser feature for the same instance.

Stable compiler options take precedence where both surfaces exist:

- `vize({ vapor })` wins over `compiler.vapor`, then `experimentals.vapor`.
- `vize({ jsxMode })` wins over `compiler.jsxMode`, then `experimentals.jsxVapor`.
- Resolved fields such as `experimentalPatternedTemplate` are low-level compiler switches. Use them
only when an integration already owns config resolution.

## Surface Matrix

| Flag | Main effect | Consumed by | Runtime behavior |
| --- | --- | --- | --- |
| `patternedTemplate` | Parses and lowers `v-match` / `v-when` | DOM, SSR, Vapor, SFC, WASM compile APIs | Rewrites to ordinary conditional render code |
| `inTagComment` | Parses `//` comments inside opening tags | Parser, DOM, SSR, Vapor, SFC, WASM compile APIs | No generated runtime output |
| `selfComponent` | Treats exact `<Self>` as the current component | DOM, SSR, Vapor, SFC, WASM compile APIs | Resolves with Vue's maybe-self-reference hint |
| `strictSlotChildren` | Emits virtual TypeScript child assertions | `vize check`, LSP/type-check project APIs | No compiler/runtime output |
| `serverScript` | Enables reserved server-script compiler plumbing | Native compiler integrations that expose it | Experimental host-defined behavior |
| `vapor` | Selects SFC Vapor backend when stable Vapor is unset | Vite plugin, package build config | Changes SFC output backend |
| `jsxVapor` | Defaults JSX/TSX to Vapor when stable JSX mode is unset | Vite plugin, package build config | Changes JSX/TSX output backend |

## Flag Reference

| Flag | Upstream / source | Resolved compiler field | Default | Compatibility names |
| --- | --- | --- | --- | --- |
| `patternedTemplate` | Vue RFC [#823](https://github.com/vuejs/rfcs/pull/823) | `experimentalPatternedTemplate` | off | `pattenedTemplate` |
| `inTagComment` | Vue RFC [#831](https://github.com/vuejs/rfcs/pull/831) | `experimentalInTagComments` | off | `intagComment` |
| `selfComponent` | Vue RFC [#833](https://github.com/vuejs/rfcs/pull/833) | `experimentalSelfComponent` | off | `self_component` in JSON |
| `strictSlotChildren` | Vue RFC [#734](https://github.com/vuejs/rfcs/pull/734) | `experimentalStrictSlotChildren` | off | `strict_slot_children` in JSON |
| `serverScript` | Server script compiler experiment | `experimentalServerScript` | off | `"server script"`, `server_script` in JSON |
| `vapor` | SFC Vapor backend routing | `compiler.vapor` fallback | off | none |
| `jsxVapor` | JSX/TSX Vapor default | `compiler.jsxMode` fallback | off | none |

Do not set a recommended flag and its alias at the same time. Compatibility aliases exist so older configs keep loading, but mixed spelling makes precedence hard to read. New config should use `patternedTemplate`, `inTagComment`, `selfComponent`, `strictSlotChildren`, `serverScript`, `vapor`, and `jsxVapor`.

## Patterned Templates

Enable `patternedTemplate` to parse `v-match` containers and direct `v-when` branch children from Vue RFC #823:

```ts
export default defineConfig({
  experimentals: {
    patternedTemplate: true,
  },
});
```

```vue
<script setup lang="ts">
type Entry =
  | { kind: "article"; data: { title: string; published: boolean } }
  | { kind: "draft"; reason: string }
  | { kind: "archived"; id: string };

const entry = ref<Entry>({ kind: "draft", reason: "editing" });
</script>

<template v-match="entry">
  <article v-when="{ kind: 'article', data: const article } if (article.published)">
    {{ article.title }}
  </article>
  <p v-when="'draft' | 'archived'">Hidden</p>
  <p v-when="_">No published article</p>
</template>
```

The matched subject is evaluated once. Branches are checked top to bottom, the first matching branch renders, and no later branch falls through. `v-when="_"` is the fallback pattern and must be unique and last when it is an unguarded top-level fallback.

Supported patterns include:

| Pattern | Example | Notes |
| --- | --- | --- |
| Literal | `v-when="'ready'"`, `v-when="404"` | Uses strict equality; `NaN` uses `Number.isNaN` |
| Value | `v-when="Status.Active"` | Identifiers and member expressions compare to runtime values |
| Wildcard | `v-when="_"` | Matches without introducing a binding |
| Const binding | `v-when="const value"` | `let` and `var` are rejected |
| Object | `v-when="{ kind: 'ok', value: const result }"` | Extra object properties are allowed |
| Object shorthand | `v-when="{ kind: 'ok', const data }"` | `{ const data }` means `{ data: const data }` |
| Object rest | `v-when="{ kind: 'error', ...const payload }"` | Rest bindings must use `const`; lone `...` is accepted |
| Array / tuple | `v-when="[const first, ...const rest]"` | Requires `Array.isArray`; rest allows additional items |
| Or | `v-when="'idle' | 'loading'"` | Alternatives match left to right; bindings inside alternatives are not supported yet |
| As binding | `v-when="{ kind: 'ok', const data } as const entry"` | Binds the matched value in addition to nested bindings |
| Guard | `v-when="{ kind: 'error', error: const e } if (e.retriable)"` | The guard runs after the pattern matches |

`v-case` remains a compatibility alias for older Vize experiments, but new templates should use `v-when`. The shorthand candidates discussed in RFC #823 are not enabled: `?=`, `|=`, and `~=` are not public Vize syntax.

Invalid placements are reported instead of silently compiling. A `v-when` branch must be a direct child of a `v-match` container, `v-match` must have at least one direct branch, and `v-when` does not accept directive arguments or modifiers. Without the flag, Vize reports that `experimentals.patternedTemplate` is required.

## In-Tag Comments

Enable `inTagComment` to parse compile-time-only `//` comments inside opening tag attribute lists from Vue RFC #831:

```ts
export default defineConfig({
  experimentals: {
    inTagComment: true,
  },
});
```

```vue
<template>
  <LegacySelect
    :options="options"
    // @vue-expect-error legacy API accepts string IDs
    :selected-id="selectedId"
  />
</template>
```

The comment may appear after the tag name, between complete attributes or directives, or after the last attribute before `>` or `/>`. It is stored as an in-tag comment for tooling and source mapping; it is not emitted as a child comment node and it does not generate runtime output. `//` inside an attribute value remains ordinary text:

```vue
<template>
  <a href="https://example.test/path//segment">Link</a>
</template>
```

The normal template `comments` compiler option does not enable or disable this syntax. If the flag is off, the parser reports the unexpected solidus instead of accepting the opening tag. This syntax is intended for SFC/tooling pipelines that own template parsing; do not rely on it in raw in-DOM templates that must be parsed first by a browser HTML parser.

## Self Component

Enable `selfComponent` to reserve the exact `<Self>` tag for recursive component references from Vue RFC #833:

```ts
export default defineConfig({
  experimentals: {
    selfComponent: true,
  },
});
```

```vue
<script setup lang="ts">
defineProps<{
  node: { id: string; children: Array<{ id: string; children: unknown[] }> };
}>();
</script>

<template>
  <li>
    {{ node.id }}
    <ul v-if="node.children.length">
      <Self v-for="child in node.children" :key="child.id" :node="child" />
    </ul>
  </li>
</template>
```

In SFC builds, Vize resolves `<Self>` to the current component name from compiler metadata or from the filename. Direct template APIs can pass `componentName` when no SFC filename is available. Without `selfComponent`, `<Self>` stays a normal component tag and any local component binding named `Self` keeps its ordinary meaning.

The flag is threaded through DOM, SSR, and Vapor compilation. When enabled and a component name is available, emitted component resolution uses the current component name and Vue's maybe-self-reference hint. The reserved tag is exact and case-sensitive: `<Self>` is special, while `<self>` is not.

## Strict Slot Children

Enable `strictSlotChildren` to add virtual-TypeScript checks for slot children from Vue RFC #734. This flag is for `vize check`, the LSP/type-check project APIs, and declaration-aware virtual code; ordinary template compilation does not add runtime behavior.

```ts
export default defineConfig({
  experimentals: {
    strictSlotChildren: true,
  },
});
```

A component can expose a structural slot contract through `__vizeSlots`:

```ts
import TabItem from "./TabItem.vue";

declare const Tabs: {
  readonly __vizeSlots?: {
    default: () => (typeof TabItem)[];
    footer: () => [HTMLButtonElement];
  };
};
```

Vize compares provided children with that contract:

```vue
<template>
  <Tabs>
    <TabItem />
    <div />

    <template #footer>
      <button>Save</button>
    </template>
  </Tabs>
</template>
```

With the flag enabled, Vize synthesizes slot child assertions so TypeScript can compare the provided nodes with the slot return contract. Native DOM children map to their corresponding `HTML*Element`, component children map to `typeof ComponentRef`, and text or interpolation children map to `string`. Named slots and default slots are both checked. Comments and structural wrapper nodes are skipped when they do not contribute a child value.

Because this is a tooling constraint, failures appear as TypeScript diagnostics from `vize check` or the language server. Builds that only compile Vue templates will not enforce it.

## Server Script

Enable `serverScript` only for hosts that explicitly test the server-script compiler experiment:

```ts
export default defineConfig({
  experimentals: {
    serverScript: true,
  },
});
```

The flag resolves to the native `experimentalServerScript` compiler field. Some runtimes keep this switch reserved until their compiler stage supports the experiment, so application code should not rely on it as a stable syntax contract. Prefer feature-specific host documentation when a framework starts consuming this switch.

## Vapor

Enable `vapor` to route SFC compilation through experimental Vapor backend support when stable `compiler.vapor` and direct `vize({ vapor })` are not set:

```ts
export default defineConfig({
  experimentals: {
    vapor: true,
  },
});
```

Prefer `compiler.vapor` or direct `vize({ vapor })` when a project has made Vapor a stable build choice. Keep `experimentals.vapor` for trial runs, compatibility probes, and tests that need to prove the experimental fallback path still works.

## JSX Vapor

Enable `jsxVapor` to default JSX and TSX files to Vapor output when stable `compiler.jsxMode` and direct `vize({ jsxMode })` are not set:

```ts
export default defineConfig({
  experimentals: {
    jsxVapor: true,
  },
});
```

Prefer `compiler.jsxMode: "vapor"` or direct `vize({ jsxMode: "vapor" })` for a stable project-wide backend choice. Component-level `"use vue:vapor"` and `"use vue:vdom"` directives still describe the source file's own intent and can override the default mode for that file.

## Direct API Fields

Most applications should configure `experimentals`. Lower-level integrations that already perform their own config resolution can pass native compiler fields directly:

```ts
compileTemplate(source, {
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalSelfComponent: true,
  experimentalStrictSlotChildren: true,
  experimentalServerScript: true,
});
```

These fields are already resolved booleans. They do not understand aliases, `{}` switch objects, or shared-config precedence. Use them only at integration boundaries where the caller owns those rules.
