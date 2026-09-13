# Experimental Vue RFC Flags

Every flag in this page is fully opt-in. Vize keeps these RFC surfaces disabled unless the matching
`experimentals` key is `true` or `{}` in `vize.config.*`. Missing keys, `false`, and `null` are off.

The full docs site reference is https://vizejs.dev/guide/experimentals. The RFC-specific details are
at https://vizejs.dev/guide/experimentals-vue-rfcs. They cover backend experiments such as
`serverScript`, `vapor`, and `jsxVapor` from the shared Experimentals page.

## Resolution Surfaces

| Surface                          | Accepted option shape                                                                               | Off and override behavior                                                                                       |
| -------------------------------- | --------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `vize.config.*`                  | `experimentals` switch values                                                                       | Shared default for npm commands, `vize check`, LSP sessions, and Vite plugin instances that load project config |
| Direct `vize({ experimentals })` | the same switch values                                                                              | Wins over shared config; `false` and `null` explicitly disable a shared opt-in for that plugin invocation       |
| Native template APIs             | `experimental*` boolean fields on `compile`, `compileVapor`, and `parseTemplate` options            | Caller must pass final booleans; aliases and `{}` are not resolved                                              |
| Native SFC APIs                  | `experimental*` boolean fields on `compileSfc`, `compileSfcBatch`, and `compileSfcBatchWithResults` | Caller must pass final booleans; `strictSlotChildren` is meaningful for SFC/check virtual TypeScript surfaces   |

```json
{
  "experimentals": {
    "inTagComment": true,
    "patternedTemplate": true,
    "selfComponent": true,
    "strictSlotChildren": true
  }
}
```

The historical names `intagComment` and `pattenedTemplate` remain accepted as aliases for
`inTagComment` and `patternedTemplate`, but new configs should use the recommended names. Avoid
setting a recommended name and its alias together.

## In-Tag Comments

`experimentals.inTagComment` enables compile-time-only `//` comments inside
opening tags.

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

Comments may appear after the tag name, between complete attributes or
directives, or after the final attribute before `>` / `/>`. They are collected as
source annotations for tooling and do not generate runtime output. The normal
template `comments` compiler option does not enable or disable this syntax. If
the flag is off, `//` in an opening tag fails as invalid tag syntax instead of
being passed through as an attribute.

Upstream reference: https://github.com/vuejs/rfcs/pull/831

## Patterned Templates

`experimentals.patternedTemplate` enables `v-match` containers and direct
`v-when` branch children. The matched subject is evaluated once, branches are
tested top to bottom, and the first matching branch renders. `_` is the fallback
pattern. `v-case` is still accepted as a compatibility alias, but new templates
should prefer `v-when`.

```vue
<template v-match="entry">
  <article v-when="{ kind: 'article', data: const article } if (article.published)">
    {{ article.title }}
  </article>
  <p v-when="'draft' | 'archived'">Hidden</p>
  <p v-when="_">No published article</p>
</template>
```

Supported branch patterns include:

- Literal and value equality patterns
- `NaN`, emitted with `Number.isNaN`
- Open object patterns such as `{ kind: 'ok', value: const result }`
- Array and tuple structure checks
- Wildcard `_`
- `const` bindings and object shorthand such as `{ const data }`
- Lone `...` rest and rest bindings
- Top-level `|` alternatives
- `as const` bindings
- Parenthesized patterns
- Optional `if (...)` guards after the pattern

Pattern bindings are branch-local. `let` and `var` bindings are rejected so the
surface stays const-only. Bindings inside `|` alternatives are intentionally not
supported in the first implementation. Unguarded top-level `_` fallbacks must be
unique and last.

Upstream reference: https://github.com/vuejs/rfcs/pull/823

## Self Component

`experimentals.selfComponent` reserves `<Self>` for recursive component
references. In SFC builds, Vize resolves `<Self>` to the current component name
from compiler metadata or from the filename. Without the flag, `<Self>` remains a
normal component tag.

```vue
<template>
  <Self v-for="child in node.children" :key="child.id" :node="child" />
</template>
```

The flag is threaded through DOM, SSR, and Vapor compilation surfaces. Direct
template APIs can also receive `componentName` when no SFC filename is available.
When the flag is off, `<Self>` remains an ordinary component tag. When it is on,
the exact uppercase tag shadows a local/imported/global component also named
`Self`; lowercase `<self>` is not special.

Upstream reference: https://github.com/vuejs/rfcs/pull/833

## Strict Slot Children

`experimentals.strictSlotChildren` enables virtual-TypeScript checks for
components whose slot contract declares child node types. It is used by
`vize check`; ordinary compilation does not enable the checks.

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

With the flag enabled, Vize synthesizes slot child assertions so TypeScript can
compare the provided child nodes with the slot return contract. Native DOM
children map to their corresponding `HTML*Element`, component children map to
`typeof ComponentRef`, and text/interpolation children map to `string`.
Single-child, array, and tuple return shapes are preserved, so
`() => HTMLInputElement`, `() => HTMLInputElement[]`, and
`() => [HTMLInputElement, HTMLInputElement]` describe different cardinality
contracts.

The check is intentionally virtual-TS-only because the RFC describes a tooling
constraint, not a runtime compiler transform.

Upstream reference: https://github.com/vuejs/rfcs/pull/734/files

## Backend Experimentals

The RFC flags above share switch semantics and precedence with backend experiments. Shared
`vize.config.*` enables them by default for tools that read project config, while direct Vite plugin
options can opt out with `false` or `null` for one plugin invocation.

```json
{
  "experimentals": {
    "serverScript": true,
    "vapor": true,
    "jsxVapor": true
  }
}
```

- `serverScript` resolves to the native `experimentalServerScript` compiler flag. Missing keys,
  `false`, and `null` leave that native flag disabled. Some runtimes keep this switch reserved until
  their compiler stage supports the experiment.
- `vapor` routes SFC compilation through experimental Vapor backend support only when direct
  `vize({ vapor })` and stable `compiler.vapor` are both unset. Use `vize({ vapor: false })` as a
  per-plugin opt-out.
- `jsxVapor` defaults JSX and TSX files to Vapor output only when direct `vize({ jsxMode })` and
  stable `compiler.jsxMode` are both unset. `jsxMode: "vdom"` is the explicit opt-out, and per-file
  `"use vue:vapor"` / `"use vue:vdom"` directives still win for that component.

Prefer stable `compiler.vapor` or `compiler.jsxMode: "vapor"` when those choices are no longer
experiments for the project.
