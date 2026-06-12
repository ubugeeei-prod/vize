---
title: JSX & TSX
---

# JSX & TSX

> **⚠️ Experimental:** JSX/TSX support is new and still moving. Type-checking is opt-in,
> HMR for `.jsx`/`.tsx` modules is not wired yet, and the authoring surface may change. Use it
> alongside `.vue` SFCs rather than as a full replacement today.

Vize compiles `.jsx` and `.tsx` Vue components through the **same compiler crates** as `.vue`
single-file components — the VDOM and Vapor backends, Croquis semantic analysis, Canon type
checking, Patina lint, and the Maestro language server. There is no separate Babel pipeline and no
runtime JSX factory shim: a JSX component is lowered straight to a Vue render function (or a Vapor
template) by the native compiler.

This means a `.tsx` Vue component gets the same Rust-native compilation, the same type checking, and
the same editor experience as an SFC — just authored as a typed function instead of a `<template>`.

## Enabling JSX/TSX

`.jsx` and `.tsx` files are routed through the Vize bundler plugins automatically — there is no
opt-in flag to compile them. Any project already using a Vize bundler integration picks up JSX/TSX
support:

- `@vizejs/vite-plugin`
- `@vizejs/unplugin` (rollup / webpack / esbuild)
- `@vizejs/rspack-plugin`
- `@vizejs/nuxt`

```ts
// vite.config.ts — nothing JSX-specific is required
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

Under the hood the plugins call the native/WASM `compileJsx` entry point (exposed from
`@vizejs/native` and `@vizejs/wasm`), which lowers the source and returns render code plus any
extracted scoped CSS.

## Authoring API

A Vize JSX/TSX component is a **plain function with typed parameters**. There are no macros and no
`defineComponent` wrapper in the common case — the types are read directly from the function
signature and erased from the runtime output (zero-cost).

- **Props** are the **typed first parameter**.
- **Emits and slots** are the **typed second parameter**, a Vize-provided `Ctx<Emits, Slots>`
  context (with `emit`, `slots`, and `attrs`, mirroring Vue's setup context).
- **Default prop values** come from **destructuring defaults** in the parameter pattern — the
  compiler extracts them from the destructuring.

```tsx
const Counter = (
  props: { label: string; start?: number },
  { emit }: Ctx<{ change: [value: number] }>,
) => <button onClick={() => emit("change", 1)}>{props.label}</button>;
```

Props-only components can omit the second parameter entirely:

```tsx
const Hello = (props: { name: string }) => <h1>Hello, {props.name}!</h1>;
```

Default values are written as destructuring defaults; no separate `props` option is needed:

```tsx
const Badge = ({ count = 0 }: { count?: number }) => <span class="badge">{count}</span>;
```

The component name is taken from the binding (`const Counter = …`) or the function declaration
(`function Card() { … }`), exactly as you would expect. Everything else is React-like JSX — element
nesting, fragments (`<>…</>`), expression children, and event props such as `onClick`. The only
Vue-specific addition is the `<style scoped>` element described [below](#scoped-styles).

> The type-only authoring form above is the supported common case. Synthesizing runtime `props`
> metadata, and the stateful `defineComponent(() => () => vnode)` setup form, are planned follow-ups.

## Output mode: VDOM vs Vapor

Each component compiles to either **Virtual DOM** output (Vue's default renderer) or
[**Vapor**](https://blog.vuejs.org/posts/vue-vapor) output. The default is chosen by configuration;
individual components can override it.

### Config default

`compiler.jsxMode` sets the global default backend for `.jsx`/`.tsx` components. It accepts `"vdom"`
or `"vapor"` and defaults to `"vdom"`.

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    // Default every .jsx/.tsx component to Vapor output.
    jsxMode: "vapor",
  },
});
```

`jsxMode` is independent of `compiler.vapor`: `vapor` toggles Vapor for `.vue` SFCs, while `jsxMode`
controls the default backend for JSX/TSX. A project can keep SFCs on VDOM while defaulting JSX to
Vapor, or vice versa. The Vite plugin also accepts `jsxMode` directly as a plugin option, which
overrides the shared config.

### Per-component directives

An individual component overrides the default with a directive prologue, mirroring `"use strict"`:

```tsx
// Compiled to Vapor regardless of the configured default.
const Fast = () => {
  "use vue:vapor";
  return <div class="fast" />;
};

// Compiled to Virtual DOM regardless of the configured default.
const Classic = () => {
  "use vue:vdom";
  return <div class="classic" />;
};
```

Because each component is routed independently, a **single file can mix both backends**:

```tsx
// vize.config: { compiler: { jsxMode: "vapor" } }

// No directive -> takes the configured default (Vapor here).
export const Dashboard = () => <main>{/* ... */}</main>;

// Opts back into Virtual DOM just for this component.
export const LegacyWidget = () => {
  "use vue:vdom";
  return <aside>{/* ... */}</aside>;
};
```

### Precedence

The output mode for a component resolves in this order:

1. A per-component `"use vue:vapor"` / `"use vue:vdom"` directive.
2. The `compiler.jsxMode` default from config (or the plugin's `jsxMode` option).
3. The built-in fallback, `"vdom"`.

### Diagnostics

Malformed or conflicting directives are reported rather than silently ignored:

- A directive that begins with `"use vue:"` but does not name a known mode (a typo such as
  `"use vue:vdomm"`) is a compile error.
- Two conflicting mode directives in one component (`"use vue:vapor"` followed by `"use vue:vdom"`)
  are diagnosed; the first directive still wins for the resolved mode.
- Unrelated prologues such as `"use strict"` are left untouched.

## Scoped styles

A `<style scoped>` element **inside the component** is the JSX equivalent of an SFC's
`<style scoped>` block. It is extracted at compile time — never rendered as a runtime `<style>`
vnode — its CSS is scope-rewritten with a generated `data-v-<hash>` scope id, that scope attribute
is injected onto the component's other elements, and the rewritten CSS is emitted through the
bundler plugin's CSS pipeline. This works in both the VDOM and Vapor backends, and both derive the
same scope id for a given component.

Idiomatically the `<style scoped>` element goes **last**, after the markup — matching an SFC's
`<template>` → `<style>` order — but the compiler extracts it wherever it appears.

```tsx
const Card = () => (
  <>
    <div class="box">hi</div>
    <style scoped>{`
      .box {
        color: red;
      }
    `}</style>
  </>
);
```

### Style-block interpolation (`v-bind` equivalent)

Template-literal interpolations `${expr}` inside the style block are the Vize equivalent of SFC CSS
`v-bind()`. They are recovered at compile time and type-checked against the component's scope:

```tsx
const Box = (props: { color: string }) => (
  <>
    <div class="box" />
    <style scoped>{`
      .box {
        color: ${props.color};
      }
    `}</style>
  </>
);
```

A `<style>` element **without** `scoped` is treated as a normal element and rendered as-is — it is
not extracted.

> The literal CSS `v-bind(...)` function syntax used inside an SFC `<style>` block is **not** a
> supported authoring form inside a JSX style block. Use `${expr}` interpolation instead.

## Type-checking

JSX/TSX type-checking is **opt-in** through `typeChecker.jsxTypecheck`, which defaults to **`false`**.
It is off by default on purpose: a repository may contain React `.tsx` files that must not be
type-checked as Vue JSX.

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  typeChecker: {
    enabled: true,
    jsxTypecheck: true,
  },
});
```

When enabled, `vize check` type-checks `.jsx`/`.tsx` Vue components — props, emits, and slots from
the typed parameters; directives and `v-model`; and the `${expr}` interpolations in `<style scoped>`
blocks. Diagnostics are reported at the **original source locations** (both as JSON for the CLI and
through the LSP), because the JSX is lowered to plain virtual TypeScript whose every byte maps back
to the source.

```bash
# Type-check a project including its .jsx/.tsx Vue components
vize check src
```

The JSX virtual TypeScript is plain `.ts` (never a TSX-format virtual document), and the LSP and the
CLI share the same lowering, so an error Corsa reports lands at the identical source range in the
editor and on the command line.

## Editor / LSP

Opening a `.jsx`/`.tsx` Vue component in an editor backed by `vize lsp` gives the same language
features as an SFC — **no SFC wrapper needed**:

- Diagnostics
- Hover
- Completion
- Go-to-definition
- References
- Rename
- Document symbols
- Semantic tokens
- Code actions
- Embedded CSS diagnostics for `<style scoped>` blocks

Structural features (document symbols, semantic tokens, scoped-style diagnostics, code actions) work
from the parsed document and are always available. Type-aware features (diagnostics, hover,
completion, go-to-definition, references, rename) are reached only when `typeChecker.jsxTypecheck` is
enabled, so React `.tsx` files are never treated as Vue JSX in the editor either.

## Linting

Vize's Patina lint rules run on JSX/TSX through a **zero-cost rule IR projected straight from the OXC
AST** — there is no synthetic template AST reconstructed for JSX. Semantic rules are backed by
Croquis, the same analysis layer used for SFCs. Diagnostics and fixes map to the JSX source ranges,
so the output lines up with what you wrote.

See [Static Analysis](./static-analysis.md) for the lint and type-check model, and
[Rules](../rules/index.md) for concrete rule output.

## Limitations

JSX/TSX support is experimental. Be aware of the current edges:

- **Type-checking is opt-in / experimental.** `typeChecker.jsxTypecheck` is `false` by default; turn
  it on per project once you are ready for Vue-JSX diagnostics on `.jsx`/`.tsx`.
- **HMR is not yet wired for `.jsx`/`.tsx` modules.** The JSX compiler currently emits a
  render-function module rather than a full component-object module, so there is no Vue HMR boundary
  to attach to. Full component-module output plus state-preserving HMR is a planned follow-up; until
  then, edits to a `.jsx`/`.tsx` component fall back to a normal reload.
- **Literal CSS `v-bind(...)` inside a JSX `<style scoped>` block is not supported.** Use `${expr}`
  template-literal interpolation, which is the supported, type-checked form.

## See also

- [Configuration](./configuration.md) — the `compiler.jsxMode` and `typeChecker.jsxTypecheck` keys,
  plus the full shared config shape.
- [Vite Plugin](./vite-plugin.md) — the recommended bundler integration.
- [Static Analysis](./static-analysis.md) — how lint and type checking share the compiler pipeline.
