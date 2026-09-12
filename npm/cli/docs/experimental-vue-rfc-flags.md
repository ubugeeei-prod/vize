# Experimental Vue RFC Flags

Every flag in this page is fully opt-in. Vize keeps these RFC surfaces disabled
unless the matching `experimentals` key is `true` or `{}` in `vize.config.*`.
Use `false` or `null` to keep a flag disabled.

```json
{
  "experimentals": {
    "intagComment": true,
    "patternedTemplate": true,
    "selfComponent": true,
    "strictSlotChildren": true
  }
}
```

`pattenedTemplate` remains accepted as a historical alias for
`patternedTemplate`, but new configs should use `patternedTemplate`.

## In-Tag Comments

`experimentals.intagComment` enables compile-time-only `//` comments inside
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
template `comments` compiler option does not enable or disable this syntax.

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

The check is intentionally virtual-TS-only because the RFC describes a tooling
constraint, not a runtime compiler transform.

Upstream reference: https://github.com/vuejs/rfcs/pull/734/files
