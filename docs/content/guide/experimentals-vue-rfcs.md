---
title: Vue RFC Experimental Details
---

# Vue RFC Experimental Details

This page expands [Experimentals](./experimentals.md) with opt-in contracts, examples, and tooling boundaries.
Upstream RFCs remain the design sources; each feature requires its matching Vize flag.

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
compile(templateSource, { experimentalPatternedTemplate: true });
compileVapor(templateSource, { experimentalSelfComponent: true, componentName: "TreeNode" });
parseTemplate(templateSource, { experimentalInTagComments: true });
compileSfc(sfcSource, { filename: "TreeNode.vue", experimentalStrictSlotChildren: true });
```

Direct native fields on `compile`, `compileVapor`, `parseTemplate`, and `compileSfc*` do not
understand aliases such as `pattenedTemplate` or `intagComment`, and they do not accept `{}` switch
objects. Direct Vite plugin values still win over shared config, including explicit opt-outs.

## Current Scope

| RFC | Vize ships today | Boundary to keep explicit |
| --- | --- | --- |
| #823 | parser support, runtime lowering, branch-local bindings, guards, rest/as patterns, diagnostics for flag-off and invalid placement | the upstream type-tooling acceptance contract for narrowing and exhaustiveness is not yet certified by `vize check` |
| #831 | parser support for in-tag `//`, source text in `root.comments`, and compile pipelines that preserve the AST comment | no runtime output, no child comment node, and no browser in-DOM template support |
| #833 | exact `<Self>` current-component resolution in DOM, SSR, and Vapor compilation | no render-function or JSX macro; a local/imported component named `Self` is shadowed when the flag is enabled |
| #734 | virtual TypeScript assertions for provided default and named slot children | no template codegen change; open slot contracts and `any` degrade to TypeScript's own permissive checks |

For entry-point and proof checklists, see [Experimentals Reference](./experimentals-reference.md).

## Patterned Templates

`patternedTemplate` implements the long-form `v-match` / `v-when` syntax from RFC #823. The subject
expression is evaluated once, direct branch children are tested in source order, and only the first
matching branch renders.

Inline HTML SFCs accept outer `<template v-match>` with nested-match semantics in DOM, SSR, and Vapor.
Header-only subject edits invalidate HMR; parsed `template.content` remains the original block body.
External `src`, preprocessors, and descriptors missing original source metadata are rejected.
Canon narrowing/exhaustiveness remains unsupported; assembled SFC source maps remain script-only.

```vue
<script setup lang="ts">
type Result =
  | { status: "success"; data: { title: string; published: boolean } }
  | { status: "error"; error: { message: string; retriable: boolean } }
  | { status: "loading" }
  | { status: "empty" };

const result = ref<Result>({ status: "loading" });
</script>

<template v-match="result">
  <ArticleView
    v-when="{ status: 'success', data: const article } if (article.published)"
    :article="article"
  />
  <RetryBanner
    v-when="{ status: 'error', error: const error } if (error.retriable)"
    :error="error"
  />
  <p v-when="{ status: 'empty' } | { status: 'loading' }">Waiting for content.</p>
  <ErrorBanner v-when="{ status: 'error', error: const error }" :error="error" />
  <p v-when="_">Unpublished article.</p>
</template>
```

Bindings are branch-local. They are visible to the element carrying `v-when`, its attributes and
directives, its guard expression, and its children. Sibling branches cannot read them.

Supported branch patterns:

| Pattern | Example | Runtime check |
| --- | --- | --- |
| Literal | `v-when="'ready'"`, `v-when="404"` | strict equality; `NaN` uses `Number.isNaN` |
| Value | `v-when="Status.Ready"` | compares with an identifier or member expression |
| Wildcard | `v-when="_"` | always matches and introduces no binding |
| Const binding | `v-when="const value"` | always matches and binds `value` in this branch |
| Object | `v-when="{ kind: 'ok', value: const data }"` | open structural object match; extra properties are allowed |
| Object shorthand | `v-when="{ kind: 'ok', const data }"` | `{ const data }` means `{ data: const data }` |
| Object rest | `v-when="{ kind: 'error', ...const payload }"` | binds remaining own enumerable properties; lone `...` is accepted |
| Array / tuple | `v-when="[const first, ...const rest]"` | requires `Array.isArray`; rest allows additional items |
| Or | `v-when="'idle' | 'loading'"` | tries alternatives from left to right |
| As binding | `v-when="{ kind: 'ok' } as const whole"` | binds the matched value as `whole` |
| Guard | `v-when="{ error: const e } if (e.retriable)"` | runs after the pattern succeeds |

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

`v-match` must have at least one direct branch, and `v-when` cannot use directive arguments or
modifiers:

```vue
<template v-match="status"></template>

<template v-match="status">
  <p v-when:ready="'ready'">Ready</p>
  <p v-when.once="'ready'">Ready again</p>
</template>
```

`let` and `var` bindings are rejected. `const` is the only binding declaration Vize accepts today.
`v-when` also rejects directive arguments and modifiers. `v-case` and `v-case.default` are kept only
as compatibility aliases for older Vize experiments; new templates should use `v-when` and `_`.

### Patterned Type Boundary

RFC #823 requires branch narrowing and exhaustiveness; `vize check` does not yet certify these,
unreachable branches, or future union members. Use `v-when="_"` for runtime fallback and manual union coverage tests.
Bindings in or-pattern alternatives are deferred: split cases needing bindings into separate branches.
The RFC shorthand candidates `?=`, `|=`, and `~=` are not public Vize branch syntax.

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
last attribute before `>` or `/>`. Put the closing delimiter on the next line after a trailing
comment: `//` consumes the closing delimiter when `>` or `/>` stays on the same line. The syntax is
not valid inside a tag name, attribute name, directive name, argument, modifier, or attribute value.

The comment is stored on the template root `comments` list with `CommentKind::InTag`. It is not an
element prop, not a child `<!-- ... -->` node, and not controlled by the normal template `comments`
option; `comments: false` still preserves in-tag comments for tooling. `//` inside an attribute value
remains text:

```vue
<template>
  <a href="https://example.test/a//b">Link</a>
</template>
```

This syntax is intended for SFC and tooling pipelines that parse templates themselves. Raw in-DOM
templates are parsed by the browser first, so do not rely on in-tag comments there.

## Self Component

`selfComponent` reserves the exact tag `<Self>` for the current component from RFC #833.

```vue
<template>
  <li>
    {{ node.id }}
    <ul v-if="node.children.length">
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

When the flag is enabled, `<Self>` is resolved before local, imported, or global components named
`Self`. Avoid naming a component import `Self`; rename the import if you need the other component.
Props, attrs, events, directives, refs, `v-if`, `v-for`, `v-show`, and slots behave like any other
component usage. Recursive templates still need a termination condition.

```vue
<script setup lang="ts">
import OtherSelf from "./OtherSelf.vue";
</script>

<template>
  <Self />
  <OtherSelf />
</template>
```

The tag is exact and case-sensitive. `<Self>` is reserved only when the flag is enabled; `<self>` is
not special. Render functions and JSX are outside this flag.

Low-level template calls must provide a name explicitly when no SFC filename is available:

```ts
import { compile } from "@vizejs/native";

compile("<Self :node=\"node\" />", {
  componentName: "TreeNode",
  experimentalSelfComponent: true,
});
```

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

The same public contract can come from `defineSlots` in an SFC:

```ts
defineSlots<{
  default(): [typeof TabItem, HTMLButtonElement];
  footer(): [HTMLButtonElement];
}>();
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
with the component's `__vizeSlots.default` return type. A mismatched child becomes a TypeScript
diagnostic:

```vue
<template>
  <Tabs>
    <div />
  </Tabs>
</template>
```

Child mapping:

| Provided child | Virtual TypeScript type |
| --- | --- |
| Native element such as `<button>` | `HTMLButtonElement` |
| SVG or MathML element | `SVGElement` or `MathMLElement` fallback when no narrower DOM type is known |
| Imported component such as `<TabItem>` | `typeof TabItem` |
| Text or interpolation | `string` |
| Named `<template #footer>` | checked against the named slot |
| Comments and structural-only wrappers | skipped when they do not contribute a child value |

RFC #734's return shapes are preserved as TypeScript cardinality contracts:

| Slot return contract | Meaning in Vize virtual TS |
| --- | --- |
| `() => HTMLInputElement` | one input child, accepting the single-child shorthand |
| `() => HTMLInputElement[]` | zero or more input children |
| `() => [HTMLInputElement, HTMLInputElement]` | exactly two input children in tuple order |
| `() => (typeof TabItem)[]` | zero or more `TabItem` component children |
| `() => [typeof TabItem, HTMLButtonElement]` | one `TabItem` child followed by one button child |

`__VizeProvidedSlotChildren<__T>` accepts a single child as either `Only` or `[Only]` when the slot
contract has one accepted child type, and preserves tuple/array shapes for multi-child contracts.
Open slot index signatures and `any` degrade to `any`, so TypeScript will not produce strict child
diagnostics there. Built-ins, dynamic components, `KeepAlive`, `Teleport`, `Transition`,
`TransitionGroup`, and `Suspense` do not currently contribute strict component child types.

Advanced component libraries can expose `__vizeResolveSlots` when the slot contract depends on props.
Components that use `defineSlots` export a `__vizeSlots` marker for parents. Required slot-name
checks are separate; this flag adds child node type checks on top of the existing virtual-TS slot
model.

## Verification Checklist

Before documenting a new experimental behavior, pin all of these facts in tests:

- the flag defaults to off
- shared config and direct plugin options resolve with the documented precedence
- direct compiler fields are final booleans
- low-level docs name the real entry points: `compile`, `compileVapor`, `parseTemplate`, and `compileSfc*`
- bad syntax produces diagnostics instead of silently compiling
- DOM, SSR, Vapor, SFC, WASM, or `vize check` coverage matches the surface matrix
- docs include a good example, a flag-off or invalid-placement example, and an implementation boundary
