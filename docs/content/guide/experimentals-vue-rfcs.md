---
title: Vue RFC Experimental Details
---

# Vue RFC Experimental Details

This page expands the Vue-RFC part of [Experimentals](./experimentals.md). It documents Vize's
shipped opt-in contract and implementation limits. The upstream pull requests remain the design
sources, but no RFC feature is enabled unless the matching Vize flag is explicitly on.

## Opt-in Contract

Every RFC flag is independent. Enabling one proposal never enables another proposal.

| RFC | Vize config flag | Direct compiler field | Applies to | When the flag is off |
| --- | --- | --- | --- | --- |
| [#823](https://github.com/vuejs/rfcs/pull/823) patterned templates | `experimentals.patternedTemplate` | `experimentalPatternedTemplate` | DOM, SSR, Vapor, SFC, WASM compile APIs | `v-match`, `v-when`, and `v-case` report that the opt-in is required |
| [#831](https://github.com/vuejs/rfcs/pull/831) in-tag comments | `experimentals.inTagComment` | `experimentalInTagComments` | Parser, DOM, SSR, Vapor, SFC, WASM compile APIs | `//` inside an opening tag is parsed as invalid tag syntax |
| [#833](https://github.com/vuejs/rfcs/pull/833) self references | `experimentals.selfComponent` | `experimentalSelfComponent` | DOM, SSR, Vapor, SFC, WASM compile APIs | `<Self>` is an ordinary component tag |
| [#734](https://github.com/vuejs/rfcs/pull/734) strict slot children | `experimentals.strictSlotChildren` | `experimentalStrictSlotChildren` | `vize check`, LSP, and virtual-TS project APIs | no extra child-type assertions are generated |

Use `experimentals` in shared config and plugin config. Use the direct compiler fields only in an
integration that has already resolved aliases, precedence, and `{}` switch objects to final booleans.

```ts
import { defineConfig } from "vize";

export default defineConfig({
  experimentals: {
    inTagComment: true,
    patternedTemplate: true,
    selfComponent: true,
    strictSlotChildren: true,
  },
});
```

```ts
vize({
  experimentals: {
    patternedTemplate: false,
    inTagComment: true,
  },
});
```

```ts
compileTemplate(source, {
  experimentalPatternedTemplate: true,
  experimentalInTagComments: true,
  experimentalSelfComponent: true,
});
```

In the second example, the direct plugin value disables patterned templates for that plugin instance
even if shared config enables it, while in-tag comments stay on. The direct `compileTemplate` fields
do not understand aliases such as `pattenedTemplate` or `intagComment`.

## Patterned Templates

`patternedTemplate` implements the long-form `v-match` / `v-when` syntax from RFC #823. The subject
expression is evaluated once, direct branch children are tested in source order, and only the first
matching branch renders.

```vue
<template v-match="result">
  <ArticleView v-when="{ status: 'success', data: const article }" :article="article" />
  <RetryBanner
    v-when="{ status: 'error', error: const error } if (error.retriable)"
    :error="error"
  />
  <ErrorBanner v-when="{ status: 'error', error: const error }" :error="error" />
  <LoadingSpinner v-when="{ status: 'loading' }" />
  <p v-when="_">Unknown result.</p>
</template>
```

Supported branch patterns:

| Pattern | Example | Runtime check |
| --- | --- | --- |
| Literal | `v-when="'ready'"`, `v-when="404"` | strict equality; `NaN` uses `Number.isNaN` |
| Value | `v-when="Status.Ready"` | compares with an identifier or member expression |
| Wildcard | `v-when="_"` | always matches and introduces no binding |
| Const binding | `v-when="const value"` | always matches and binds `value` in this branch |
| Object | `v-when="{ kind: 'ok', value: const data }"` | open structural object match |
| Object rest | `v-when="{ kind: 'error', ...const payload }"` | binds remaining own enumerable properties |
| Array rest | `v-when="[const first, ...const rest]"` | requires an array and collects extra items |
| Or | `v-when="'idle' | 'loading'"` | tries alternatives from left to right |
| As binding | `v-when="{ kind: 'ok' } as const whole"` | binds the matched value as `whole` |
| Guard | `v-when="{ error: const e } if (e.retriable)"` | runs after the pattern succeeds |

Bindings are branch-local. They are visible to the element carrying `v-when`, its attributes and
directives, its children, and the guard expression. They are not visible to sibling branches.

### Patterned Diagnostics

Without the flag, Vize reports the required opt-in instead of passing unknown directives through:

```vue
<template v-match="status">
  <p v-when="'ready'">Ready</p>
</template>
```

`v-when` must be a direct child of the `v-match` container:

```vue
<template v-match="status">
  <section>
    <p v-when="'ready'">Ready</p>
  </section>
</template>
```

An unguarded top-level fallback must be unique and last:

```vue
<template v-match="status">
  <p v-when="_">Other</p>
  <p v-when="'ready'">Ready</p>
  <p v-when="(_)">Again</p>
</template>
```

`let` and `var` bindings are rejected. `const` is the only binding declaration Vize accepts today.

```vue
<template v-match="entry">
  <p v-when="{ data: let article }">{{ article }}</p>
</template>
```

### Patterned Deferred Syntax

Vize accepts `v-case` only as a compatibility alias for older experiments. New code should use
`v-when`. The shorthand candidates discussed in the RFC are not public Vize syntax: `?=`, `|=`, and
`~=` are not parsed as branch attributes. Binding inside an or-pattern alternative is also deferred,
so split those cases into separate branches when a branch needs a binding.

## In-Tag Comments

`inTagComment` implements compile-time-only `//` comments inside an opening tag's attribute list from
RFC #831. Vize keeps their source text for parser/tooling fidelity and does not emit runtime code.

```vue
<template>
  <LegacySelect
    :options="options"
    // @vue-expect-error legacy API accepts string IDs
    :selected-id="selectedId"
  />
</template>
```

Allowed placements are after the tag name, between complete attributes/directives, and after the
last attribute before `>` or `/>`. The comment is not a child node, so it does not behave like
`<!-- ... -->` and does not depend on the template `comments` option.

The parser still treats `//` inside an attribute value as text:

```vue
<template>
  <a href="https://example.test/a//b">Link</a>
</template>
```

This is intended for SFC and tooling pipelines that parse the template themselves. Raw in-DOM
templates are parsed by the browser first, so do not use in-tag comments there.

## Self Component

`selfComponent` reserves the exact tag `<Self>` for the current component from RFC #833.

```vue
<template>
  <li>
    {{ node.id }}
    <ul>
      <Self v-for="child in node.children" :key="child.id" :node="child" />
    </ul>
  </li>
</template>
```

Resolution rules:

| Context | How Vize resolves `<Self>` |
| --- | --- |
| SFC build with compiler metadata | uses the current component name from metadata |
| SFC build without metadata | derives the component name from the file name |
| Direct template API | uses `componentName` when provided |
| No name available | keeps normal component resolution semantics |

The tag is exact and case-sensitive. `<Self>` is reserved only when the flag is enabled; `<self>` is
not special. If the flag is off, a local component binding named `Self` keeps its ordinary meaning.

## Strict Slot Children

`strictSlotChildren` implements the tooling side of RFC #734. It adds virtual TypeScript assertions
for the child nodes a parent provides to a component slot. It does not change template codegen or
runtime rendering.

```ts
import TabItem from "./TabItem.vue";

declare const Tabs: {
  readonly __vizeSlots?: {
    default: () => [typeof TabItem, HTMLButtonElement];
    footer: () => [HTMLButtonElement];
  };
};
```

```vue
<template>
  <Tabs>
    <TabItem />
    <button>Open</button>

    <template #footer>
      <button>Save</button>
    </template>
  </Tabs>
</template>
```

With the flag enabled, Vize represents the provided default slot as
`__VizeProvidedSlotChildren<[typeof TabItem, HTMLButtonElement]>` and asks TypeScript to compare it
with the component's `__vizeSlots.default` return type.

Child mapping:

| Provided child | Virtual TypeScript type |
| --- | --- |
| Native element such as `<button>` | `HTMLButtonElement` |
| Imported component such as `<TabItem>` | `typeof TabItem` |
| Text or interpolation | `string` |
| Named `<template #footer>` | checked against the named slot |
| Comments and structural-only wrappers | skipped when they do not contribute a child value |

Advanced component libraries can expose `__vizeResolveSlots` when the slot contract depends on props.
Components that use `defineSlots` export a `__vizeSlots` marker for parents. Required slot-name
checks are separate; this flag adds child node type checks on top of the existing virtual-TS slot
model.

## Verification Checklist

Before documenting a new experimental behavior, pin all of these facts in tests:

- the flag defaults to off
- shared config and direct plugin options resolve with the documented precedence
- direct compiler fields are final booleans
- bad syntax produces diagnostics instead of silently compiling
- DOM, SSR, Vapor, SFC, WASM, or `vize check` coverage matches the surface matrix
- docs include both a good example and a flag-off or invalid-placement example
