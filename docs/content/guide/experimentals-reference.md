---
title: Experimentals Reference
---

# Experimentals Reference

This page is the dense reference for [Experimentals](./experimentals.md) and
[Vue RFC Experimental Details](./experimentals-vue-rfcs.md). Use it when you need to know which
entry point resolves a flag, what proof belongs with a change, and which low-level API fields are
already final booleans.

## Entry Points

Use the highest-level entry point that still owns the decision. Project config is for tools that
load Vize config, direct plugin options are for one Vite plugin instance, and native compiler fields
are only for integrations that have already resolved project policy.

| Entry point | Accepted shape | Who resolves it | Notes |
| --- | --- | --- | --- |
| `vize.config.*` | `experimentals: { ... }` switch values | config loader | Shared default for npm commands, `vize check`, LSP sessions, and Vite plugin instances that load project config |
| `vize({ experimentals })` | the same switch values | Vite plugin option resolver | Direct values win over shared config; `false` and `null` are per-plugin opt-outs |
| `vize({ vapor })`, `vize({ jsxMode })`, `compiler.vapor`, `compiler.jsxMode` | stable compiler options | compiler option resolver | These stable choices win over `experimentals.vapor` and `experimentals.jsxVapor` fallback routing |
| `compile`, `compileVapor`, `parseTemplate` | `experimental*` boolean fields on `CompilerOptions` | caller | No aliases, no `{}` switch object, and no shared-config precedence |
| `compileSfc`, `compileSfcBatch`, `compileSfcBatchWithResults` | `experimental*` boolean fields on SFC/batch options | caller | Use for native SFC or WASM integrations after config resolution |
| `vize check` and LSP/type-check project APIs | project `experimentals` plus resolved type-checker flags | config loader and project session | `strictSlotChildren` produces virtual TypeScript checks here, not runtime codegen |

## Flag Contracts

| Flag | Enable when | Smallest useful proof | What remains outside the flag |
| --- | --- | --- | --- |
| `patternedTemplate` | A project intentionally wants RFC [#823](https://github.com/vuejs/rfcs/pull/823) `v-match` / `v-when` branches in authored templates | Compile one flagged component with a direct `v-match` child and one flag-off component that reports `experimentals.patternedTemplate` | Content Mapper routing, complete editor navigation coverage, and or-pattern bindings |
| `inTagComment` | Tooling needs line-local annotations inside an opening tag, such as `@vue-expect-error` near the affected prop | Parse one tagged component and assert the comment is preserved in `root.comments` while output code stays unchanged | Browser in-DOM templates, runtime comments, and the normal `comments` compiler option |
| `selfComponent` | A recursive SFC wants RFC [#833](https://github.com/vuejs/rfcs/pull/833) self-reference without relying on `name` or filename inference alone | Compile `<Self />` with `componentName` or SFC metadata and verify a local import named `Self` is not the target | JSX, render functions, and lowercase `<self>` |
| `strictSlotChildren` | A library or application exposes RFC [#734](https://github.com/vuejs/rfcs/pull/734) typed slot-child contracts and wants `vize check` or LSP diagnostics | Type-check one valid default/named slot tuple and one invalid child that TypeScript rejects | Runtime rendering, open `any` slot contracts, and built-in/dynamic component child typing |
| `serverScript` | A host integration owns and tests a server-script compiler experiment | Assert the native `experimentalServerScript` boolean is forwarded only when the switch is enabled | Public server-script syntax semantics until a host documents them |
| `vapor` | A project is trying SFC Vapor before promoting it to stable `compiler.vapor` | Verify `experimentals.vapor: true` selects Vapor only when `compiler.vapor` and direct `vize({ vapor })` are unset | Stable project-wide Vapor policy |
| `jsxVapor` | A project is trying JSX/TSX Vapor before promoting it to stable `compiler.jsxMode` | Verify `experimentals.jsxVapor: true` defaults JSX output to Vapor only when `jsxMode` is unset | Per-file `"use vue:*"` directives and stable JSX backend policy |

RFC [#831](https://github.com/vuejs/rfcs/pull/831) has the same opt-in shape as the other RFC
flags, but its behavior is parser/tooling-only: it keeps `//` annotations near attributes and does
not emit runtime comments.

## Direct API Fields

Most applications should configure `experimentals`. Lower-level integrations that already perform
their own config resolution can pass native compiler fields directly:

```ts
import {
  compile,
  compileSfc,
  compileSfcBatchWithResults,
  compileVapor,
  parseTemplate,
} from "@vizejs/native";

compile(templateSource, {
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalSelfComponent: true,
  experimentalServerScript: true,
  componentName: "TreeNode",
});

compileVapor(templateSource, {
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalSelfComponent: true,
  componentName: "TreeNode",
});

parseTemplate(templateSource, {
  experimentalInTagComments: true,
});

compileSfc(sfcSource, {
  filename: "TreeNode.vue",
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalSelfComponent: true,
  experimentalStrictSlotChildren: true,
  experimentalServerScript: true,
});

compileSfcBatchWithResults(files, {
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalSelfComponent: true,
  experimentalStrictSlotChildren: true,
  experimentalServerScript: true,
});
```

These fields are already resolved booleans. They do not understand aliases, `{}` switch objects, or
shared-config precedence. Use them only at integration boundaries where the caller owns those rules.

## Config Recipes

Enable the smallest flag set that proves the intended behavior. Do not group unrelated RFC switches
behind one project toggle, and do not promote an experimental flag to default-on in shared config
until the feature has moved to a stable `compiler` or tool option.

One RFC proposal in shared config:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  experimentals: {
    inTagComment: true,
  },
});
```

Temporary per-plugin opt-out while shared config stays enabled:

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      experimentals: {
        inTagComment: false,
        patternedTemplate: null,
      },
    }),
  ],
});
```

Low-level integration after config resolution:

```ts
compile(templateSource, {
  experimentalInTagComments: resolvedExperimentals.inTagComments,
  experimentalPatternedTemplate: resolvedExperimentals.patternedTemplate,
});
```

In native API calls, never pass compatibility aliases such as `intagComment` or switch objects such
as `{}`. Resolve them before calling the compiler.

## Failure Examples

`patternedTemplate` stays fail-closed when structure is wrong:

```vue
<template v-match="status"></template>

<template v-match="status">
  <p v-when:ready="'ready'">Ready</p>
  <p v-when.once="'ready'">Ready again</p>
</template>
```

`inTagComment` is not a second expression grammar for attributes:

```vue
<template>
  <LegacySelect :label="'// this is an attribute value, not an in-tag comment'" />

  <LegacySelect
    :// not a directive argument comment
    selected-id="abc"
  />
</template>
```

## Slot Cardinality

RFC #734 return shapes are preserved as TypeScript cardinality contracts:

| Slot return contract | Meaning in Vize virtual TS |
| --- | --- |
| `() => HTMLInputElement` | one input child, accepting the single-child shorthand |
| `() => HTMLInputElement[]` | zero or more input children |
| `() => [HTMLInputElement, HTMLInputElement]` | exactly two input children in tuple order |
| `() => (typeof TabItem)[]` | zero or more `TabItem` component children |
| `() => [typeof TabItem, HTMLButtonElement]` | one `TabItem` child followed by one button child |

## Implementation Coverage

| Concern | Required proof before changing docs |
| --- | --- |
| Default-off behavior | A flag-off parse, compile, check, or LSP path reports or preserves the documented off behavior |
| Config resolution | Shared config, direct Vite plugin values, aliases, `false`, `null`, and `{}` are covered separately from native booleans |
| Native template APIs | `compile`, `compileVapor`, and `parseTemplate` accept only the documented `experimental*` boolean fields |
| Native SFC APIs | `compileSfc`, `compileSfcBatch`, and `compileSfcBatchWithResults` forward the same booleans per file or batch |
| Type-checking APIs | `strictSlotChildren` is proven through virtual TypeScript diagnostics, not through runtime output snapshots |
| Boundary tests | Deferred RFC behavior has a negative assertion or documented absence, so users do not infer support from nearby syntax |

## Release Safety Checklist

Before release notes claim an experimental surface, check all of these against the shipped entry
point:

- default-off behavior is verified for the public config key and the direct native field
- aliases are documented as compatibility inputs, not as recommended names
- direct Vite plugin values can enable and explicitly opt out of shared config
- `false`, `null`, `true`, and `{}` keep the documented switch semantics
- backend fallback flags such as `vapor` and `jsxVapor` lose to stable `compiler` options
- RFC examples include both an enabled example and a flag-off or invalid-shape diagnostic example
- boundaries name deferred behavior, such as patterned-template editor navigation or strict slot
  support for dynamic components
