---
title: Vue RFC Experiments
---

# Vue RFC Experiments

Vize can try four Vue RFCs before they land in Vue. Each one is off until you turn on its own flag,
and turning on one never enables another. The RFC on GitHub is the design source; this page shows
what Vize implements today and where it stops. To try one, start with
[Enable an experiment](#enable-an-experiment).

> [!WARNING]
> These features follow open proposals. Syntax and behavior can change with the RFC, and code that
> uses them only compiles with Vize.

## Enable an experiment

| RFC | What it adds | Flag | Compiler field | Works in |
| --- | --- | --- | --- | --- |
| [#823](https://github.com/vuejs/rfcs/pull/823) | `v-match` / `v-when` pattern matching in templates | `patternedTemplate` | `experimentalPatternedTemplate` | DOM, SSR, Vapor, `vize check`, LSP diagnostics |
| [#831](https://github.com/vuejs/rfcs/pull/831) | `//` comments inside an opening tag | `inTagComment` | `experimentalInTagComments` | Parser and every compile target; no runtime output |
| [#833](https://github.com/vuejs/rfcs/pull/833) | `<Self>` for the current component | `selfComponent` | `experimentalSelfComponent` | DOM, SSR, Vapor |
| [#734](https://github.com/vuejs/rfcs/pull/734) | Type checks for the children passed to a slot | `strictSlotChildren` | `experimentalStrictSlotChildren` | `vize check` and the language server only |

Turn flags on under `vue.experimentals` in `vite.config.ts`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    experimentals: {
      inTagComment: true,
      patternedTemplate: true,
      selfComponent: true,
      strictSlotChildren: true,
    },
  },
});
```

With a flag off, Vize treats the syntax as it would without the RFC: `v-match`, `v-when`, and
`v-case` report that the opt-in is required, `//` in a tag is invalid syntax, `<Self>` is an
ordinary component tag, and no extra slot child types are generated.

Integrations that call the compiler directly pass the resolved booleans instead:

```ts
compile(templateSource, { experimentalPatternedTemplate: true });
compileVapor(templateSource, { experimentalSelfComponent: true, componentName: "TreeNode" });
parseTemplate(templateSource, { experimentalInTagComments: true });
compileSfc(sfcSource, { filename: "TreeNode.vue", experimentalStrictSlotChildren: true });
```

These direct fields on `compile`, `compileVapor`, `parseTemplate`, and `compileSfc*` take final
booleans only. They do not understand aliases such as `pattenedTemplate` or `intagComment`, or `{}`
switch objects. A value passed directly to the Vite plugin still wins over shared config, including an
explicit `false`. [Experimentals](./experimentals.md) covers precedence, and the
[Experimentals Reference](./experimentals-reference.md) lists every entry point.

## Patterned Templates

`patternedTemplate` adds the long-form `v-match` / `v-when` syntax from RFC #823. Put `v-match` on a
container and `v-when` on each direct child. Vize evaluates the subject once, tries the branches in
order, and renders only the first one that matches.

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

How it behaves:

- Bindings declared with `const` are branch-local. The branch element, its attributes, its guard,
  and its children can read them; sibling branches cannot.
- Each property is read once per attempted branch, and guards see the same values as the rendered
  branch. A guard only runs after its pattern matched.
- `v-match` renders no extra fragment or list marker in DOM, SSR, or Vapor. A single-root match keeps
  attribute fallthrough, works inside `Transition` and `KeepAlive`, and hydrates node for node.
- A branch's `:key` is evaluated with that branch's bindings.
- An outer `<template v-match>` works in SFCs too. External `src` templates and preprocessors are
  rejected.

| Pattern | Example | Matches when |
| --- | --- | --- |
| Literal | `v-when="'ready'"`, `v-when="404"` | strictly equal |
| Value | `v-when="Status.Ready"` | equal to an identifier or member expression (SameValueZero, so `NaN` matches) |
| Wildcard | `v-when="_"` | always; binds nothing |
| Const binding | `v-when="const value"` | always; binds `value` |
| Object | `v-when="{ kind: 'ok', value: const data }"` | the listed properties match; extra properties are allowed |
| Object shorthand | `v-when="{ kind: 'ok', const data }"` | same as `{ data: const data }` |
| Object rest | `v-when="{ kind: 'error', ...const payload }"` | binds the remaining own enumerable properties |
| Array / tuple | `v-when="[const first, ...const rest]"` | the value is an array (`Array.isArray`); rest allows more items |
| Or | `v-when="'idle' \| 'loading'"` | any alternative matches, tried left to right |
| As binding | `v-when="{ kind: 'ok' } as whole"` | the pattern matches; binds the whole value as `whole` |
| Guard | `v-when="{ error: const e } if (e.retriable)"` | the pattern matches and the guard is truthy |

### Patterned Diagnostics

Vize reports misuse instead of passing unknown directives through. Without the flag, it says the
opt-in is required:

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

An unguarded fallback must be unique and last:

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

Also rejected:

- `let` and `var` bindings are rejected; `const` is the only binding declaration.
- `v-when` on the same element as `v-if`, `v-else-if`, `v-else`, `v-for`, or `v-match`.
- The old `as const name` spelling; write `pattern as name`.
- `v-case` and `v-case.default` are kept only as aliases for older Vize experiments; use `v-when`
  and `_` in new templates.

### Patterned Type Boundary

With the flag on, `vize check` understands patterns:

- Branch bindings and the subject are narrowed to the matched shape.
- Missing coverage is an error anchored to the authored subject and fails `vize check`; add an
  unguarded `v-when="_"` when coverage cannot be proven.
- Unreachable branches are warnings and do not fail the check.
- Guards never prove coverage. Optional properties keep `undefined` in the remaining cases.
- Finite object and tuple unions, readonly arrays, and rest bindings are supported. Open primitives,
  `any`, `unknown`, and union-valued value patterns stay conservative.
- Binding inside an or-pattern alternative is also deferred; split cases that need bindings into
  separate branches. The RFC shorthands `?=`, `|=`, and `~=` are not supported.

| Tool | Support today |
| --- | --- |
| Compiler (DOM, SSR, Vapor) | Full RFC syntax |
| `vize check` | Narrowing, coverage errors, unreachable-branch warnings |
| Language server | Structural and coverage diagnostics, hover and go to definition for root-pattern bindings; restart the server after changing the flag |
| Playground | Croquis and Canon checkboxes; full type diagnostics use the Monaco TypeScript worker |
| VS Code grammar | Highlights `v-when` patterns without the flag |
| Not yet | Content Mapper routing, and full rename and completion for guards, nested closures, and or-pattern bindings ([#6176](https://github.com/ubugeeei-prod/vize/issues/6176)) |

The full upstream type-tooling acceptance contract for narrowing and exhaustiveness also includes
editor navigation and completion, which are still in progress. Virtual-TS hosts other than Vize must
register `virtualTsHelpers` once as an ambient declaration file.

## In-Tag Comments

`inTagComment` allows `//` comments inside an opening tag's attribute list (RFC #831). They exist only
at compile time: Vize keeps their text for tools and emits no runtime code.

```vue
<template>
  <LegacySelect
    :options="options"
    // @vue-expect-error legacy API accepts string IDs
    :selected-id="selectedId"
  />
</template>
```

- Put them after the tag name, between complete attributes, or after the last attribute.
- Put `>` or `/>` on the next line after a trailing comment: `//` consumes the closing delimiter when
  `>` or `/>` stays on the same line.
- They are not valid inside a tag name, attribute name, directive name or argument, modifier, or
  attribute value. `//` inside an attribute value stays text:

```vue
<template>
  <a href="https://example.test/a//b">Link</a>
</template>
```

For tools, the comment is stored in `root.comments` with `CommentKind::InTag`. It is not an element
prop or a `<!-- -->` child, and `comments: false` does not remove it.
Raw in-DOM templates are parsed by the browser first, so do not rely on in-tag comments there.

## Self Component

`selfComponent` reserves the exact tag `<Self>` for the current component (RFC #833), which makes
recursive components easy to write:

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

| Context | What `<Self>` resolves to |
| --- | --- |
| SFC build with compiler metadata | the current component name from metadata |
| SFC build without metadata | a name derived from the file name |
| Direct template API | `componentName` when provided |
| No name available | normal component resolution |

With the flag on, `<Self>` wins over a local/imported component named `Self` (which is shadowed).
Avoid naming a component import `Self`; rename it instead:

```vue
<script setup lang="ts">
import OtherSelf from "./OtherSelf.vue";
</script>

<template>
  <Self />
  <OtherSelf />
</template>
```

`<Self>` is exact and case-sensitive; `<self>` is not special. Props, attrs, events, directives,
refs, `v-if`, `v-for`, `v-show`, and slots work as on any component, and recursion still needs an end
condition. Render functions and JSX are not covered.

Low-level template calls without an SFC file name must pass the name:

```ts
import { compile } from "@vizejs/native";

compile("<Self :node=\"node\" />", {
  componentName: "TreeNode",
  experimentalSelfComponent: true,
});
```

## Strict Slot Children

`strictSlotChildren` adds type checks for the children a parent passes into a slot (RFC #734). It only
affects `vize check` and the language server; compiled output does not change.

A component declares what each slot accepts:

```ts
import TabItem from "./TabItem.vue";

declare const Tabs: {
  readonly __vizeSlots?: {
    default: () => [typeof TabItem, HTMLButtonElement];
    footer: () => [HTMLButtonElement];
  };
};
```

The same contract can come from `defineSlots` in an SFC:

```ts
defineSlots<{
  default(): [typeof TabItem, HTMLButtonElement];
  footer(): [HTMLButtonElement];
}>();
```

Children are then checked against it:

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

Vize represents the default slot above as
`__VizeProvidedSlotChildren<[typeof TabItem, HTMLButtonElement]>` and lets TypeScript compare it with
the slot's return type. A wrong child becomes a TypeScript error:

```vue
<template>
  <Tabs>
    <div />
  </Tabs>
</template>
```

| Child | Checked as |
| --- | --- |
| Native element such as `<button>` | `HTMLButtonElement` |
| SVG or MathML element | `SVGElement` or `MathMLElement` when no narrower type is known |
| Imported component such as `<TabItem>` | `typeof TabItem` |
| Text or interpolation | `string` |
| Named `<template #footer>` | against the named slot |
| Comments and wrappers that add no child | skipped |

| Slot return type | Meaning |
| --- | --- |
| `() => HTMLInputElement` | one input child |
| `() => HTMLInputElement[]` | any number of input children |
| `() => [HTMLInputElement, HTMLInputElement]` | exactly two input children, in order |
| `() => (typeof TabItem)[]` | any number of `TabItem` children |
| `() => [typeof TabItem, HTMLButtonElement]` | one `TabItem`, then one button |

Limits:

- `__VizeProvidedSlotChildren<__T>` accepts a single child as either `Only` or `[Only]` when the slot
  accepts one child type.
- Open slot index signatures and `any` degrade to `any`, so there is no strict check there.
- Built-ins, dynamic components, `KeepAlive`, `Teleport`, `Transition`, `TransitionGroup`, and
  `Suspense` do not contribute strict child types yet.
- Advanced component libraries can expose `__vizeResolveSlots` when the contract depends on props.
  Components using `defineSlots` export a `__vizeSlots` marker for parents.
- Required slot-name checks are separate; this flag only adds child type checks.
