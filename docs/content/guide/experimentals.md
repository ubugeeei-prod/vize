---
title: Experimentals
---

# Experimentals

`experimentals` is the shared opt-in surface for Vue RFC and backend features that must stay off
unless a project explicitly enables them. These flags are intentionally separate from stable
`compiler` options and from Vue runtime `features`.

Use this page when a project needs to try a proposed Vue syntax, a type-checking experiment, or an
unfinished backend route. Keep new project config on the recommended flag names in the table below;
compatibility aliases are accepted only so older configs keep loading.

## Switch Values

Missing keys, `false`, and `null` are off. `true` enables a flag. `{}` also enables a flag and keeps
the shape open for future per-feature options.

```ts
import { defineConfig } from "vize";

export default defineConfig({
  experimentals: {
    patternedTemplate: true,
    inTagComment: true,
    selfComponent: true,
    strictSlotChildren: true,
    serverScript: false,
    vapor: null,
    jsxVapor: {},
  },
});
```

Typed TypeScript and Pkl config should use `true`, `false`, `null`, or `{}`. The loader treats other
present non-`false` and non-`null` JSON values as enabled for compatibility, but they are not the
documented public shape.

## Precedence

Shared `vize.config.*` values are the default for tools that load project config. Direct Vite plugin
options win over shared config, including explicit opt-outs.

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

In this example, `patternedTemplate: false` disables a shared config opt-in for this Vite plugin
instance, while `inTagComment: true` enables the parser feature for that instance.

Stable compiler options take precedence where both surfaces exist:

- `vize({ vapor })` wins over `compiler.vapor`, then `experimentals.vapor`.
- `vize({ jsxMode })` wins over `compiler.jsxMode`, then `experimentals.jsxVapor`.
- Resolved fields such as `experimentalPatternedTemplate` are low-level compiler switches. Use them
  only when an integration already owns config resolution.

## Flag Reference

| Flag | Maps to | Compiler field | Default | Alias |
| --- | --- | --- | --- | --- |
| `patternedTemplate` | Vue RFC [#823](https://github.com/vuejs/rfcs/pull/823) | `experimentalPatternedTemplate` | off | `pattenedTemplate` |
| `inTagComment` | Vue RFC [#831](https://github.com/vuejs/rfcs/pull/831) | `experimentalInTagComments` | off | `intagComment` |
| `selfComponent` | Vue RFC [#833](https://github.com/vuejs/rfcs/pull/833) | `experimentalSelfComponent` | off | `self_component` in JSON |
| `strictSlotChildren` | Vue RFC [#734](https://github.com/vuejs/rfcs/pull/734) | `experimentalStrictSlotChildren` | off | `strict_slot_children` in JSON |
| `serverScript` | Server script compiler experiment | `experimentalServerScript` | off | `"server script"`, `server_script` in JSON |
| `vapor` | SFC Vapor backend routing | `compiler.vapor` fallback | off | none |
| `jsxVapor` | JSX/TSX Vapor default | `compiler.jsxMode` fallback | off | none |

Do not set a recommended flag and its alias at the same time. The aliases exist for migration and
can make intent hard to read when both names are present.

## Patterned Templates

Enable `patternedTemplate` to parse `v-match` containers and direct `v-when` branch children from
Vue RFC #823.

```ts
export default defineConfig({
  experimentals: {
    patternedTemplate: true,
  },
});
```

```vue
<template v-match="entry">
  <article v-when="{ kind: 'article', data: const article } if (article.published)">
    {{ article.title }}
  </article>
  <p v-when="'draft' | 'archived'">Hidden</p>
  <p v-when="_">No published article</p>
</template>
```

The matched subject is evaluated once. Branches are checked top to bottom, the first matching branch
renders, and `_` is the fallback pattern. Pattern bindings are branch-local and const-only. Rest
patterns, object and array structure checks, top-level alternatives, `as const` bindings, and
optional `if (...)` guards are part of the enabled syntax. `v-case` remains a compatibility alias,
but new templates should use `v-when`.

## In-Tag Comments

Enable `inTagComment` to allow compile-time-only `//` comments inside opening tags from Vue RFC
#831.

```ts
export default defineConfig({
  experimentals: {
    inTagComment: true,
  },
});
```

```vue
<template>
  <button
    // Kept for tooling, stripped from generated output.
    type="button"
    @click="submit"
  >
    Submit
  </button>
</template>
```

Comments may appear after the tag name, between complete attributes or directives, or after the
final attribute before `>` or `/>`. They are source annotations and do not generate runtime output.
The normal template `comments` compiler option does not enable or disable this syntax.

## Self Component

Enable `selfComponent` to reserve `<Self>` for recursive component references from Vue RFC #833.

```ts
export default defineConfig({
  experimentals: {
    selfComponent: true,
  },
});
```

```vue
<template>
  <Self v-for="child in node.children" :key="child.id" :node="child" />
</template>
```

In SFC builds, Vize resolves `<Self>` to the current component name from compiler metadata or the
filename. Without the flag, `<Self>` remains a normal component tag. Direct template APIs can pass
`componentName` when no SFC filename is available.

## Strict Slot Children

Enable `strictSlotChildren` to add virtual-TypeScript checks for slot children from Vue RFC #734.
This flag is used by `vize check`; ordinary compilation does not add runtime behavior.

```ts
export default defineConfig({
  experimentals: {
    strictSlotChildren: true,
  },
});
```

```ts
declare const Tabs: {
  readonly __vizeSlots?: {
    default: () => (typeof TabItem)[];
  };
};
```

```vue
<template>
  <Tabs>
    <TabItem />
    <div />
  </Tabs>
</template>
```

With the flag enabled, Vize synthesizes slot child assertions so TypeScript can compare provided
children with the slot return contract. Native DOM children map to their corresponding
`HTML*Element`, component children map to `typeof ComponentRef`, and text or interpolation children
map to `string`.

## Server Script

Enable `serverScript` only for hosts that explicitly test the server-script compiler experiment.

```ts
export default defineConfig({
  experimentals: {
    serverScript: true,
  },
});
```

The flag resolves to `experimentalServerScript` for native compiler integrations. Some runtimes keep
this switch reserved until their compiler stage supports the experiment, so application code should
not rely on it as a stable syntax contract.

## Vapor

Enable `vapor` to route SFC compilation through experimental Vapor backend support when stable
`compiler.vapor` is not set.

```ts
export default defineConfig({
  experimentals: {
    vapor: true,
  },
});
```

Prefer `compiler.vapor` or direct `vize({ vapor })` when a project has made Vapor a stable build
choice. Keep `experimentals.vapor` for trial runs and compatibility probes.

## JSX Vapor

Enable `jsxVapor` to default JSX and TSX files to Vapor output when stable `compiler.jsxMode` is not
set.

```ts
export default defineConfig({
  experimentals: {
    jsxVapor: true,
  },
});
```

Prefer `compiler.jsxMode: "vapor"` or direct `vize({ jsxMode: "vapor" })` for a stable project-wide
backend choice. `jsxVapor` is useful while testing the JSX Vapor path without changing the stable
compiler section.
