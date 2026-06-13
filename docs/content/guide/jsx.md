---
title: JSX & TSX
---

# JSX & TSX

> **Status:** JSX/TSX is covered across the compiler, linter, type checker, LSP, and formatter.
> Type-aware checks stay opt-in so React `.tsx` files are never treated as Vue JSX by accident.
> HMR for standalone `.jsx`/`.tsx` modules is still the main remaining integration gap.

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
import { computed, ref } from "vue";

type CounterProps = {
  label: string;
  start?: number;
};

type CounterEmits = {
  change: [value: number];
};

const Counter = ({ label, start = 0 }: CounterProps, { emit }: Ctx<CounterEmits>) => {
  const count = ref(start);
  const doubled = computed(() => count.value * 2);

  const increment = () => {
    count.value += 1;
    emit("change", count.value);
  };

  return (
    <section class="counter">
      <p>
        {label}: {count.value}
      </p>
      <p>Double: {doubled.value}</p>
      <button type="button" onClick={increment}>
        Increment
      </button>
    </section>
  );
};
```

Props-only components can omit the second parameter entirely:

```tsx
const Hello = ({ name }: { name: string }) => <h1>Hello, {name}!</h1>;
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
> metadata, and the `defineComponent(() => () => vnode)` setup form, are planned follow-ups.

## Supported JSX surface

The compiler lowers JSX to the same Relief IR used by SFC templates, then sends that IR to the VDOM
or Vapor backend. These forms are covered by the JSX/TSX test matrix:

- fragments and nested elements
- component tags, member-expression tags, and intrinsic HTML/SVG tags
- static attributes, dynamic `prop={expr}` bindings, boolean shorthand props, and spread props
- event handlers, including Vue-style option modifiers encoded in the prop name
- `v-if`, `v-else-if`, `v-else`, `v-show`, custom `v-*` directives, and `v-model`
- expression children, logical JSX branches, ternary JSX branches, and `.map(...)` list rendering
- slots written as object children or render-prop children
- TSX syntax: typed parameters, return annotations, generic JSX calls, casts, and non-null asserts
- `<style scoped>` extraction; template-literal `${expr}` interpolation is supported for advanced
  cases, but static classes and CSS variables are usually clearer

The canonical list form is idiomatic JSX:

```tsx
import { computed, ref } from "vue";

type Todo = {
  id: string;
  title: string;
  done: boolean;
};

type TodoListProps = {
  todos: Todo[];
  initialActiveId?: string;
};

const TodoList = ({ todos, initialActiveId }: TodoListProps) => {
  const activeId = ref(initialActiveId ?? todos[0]?.id);
  const activeTodo = computed(() => todos.find((todo) => todo.id === activeId.value));

  return (
    <section class="todo-panel">
      <header>
        <h2>{activeTodo.value?.title ?? "Select a todo"}</h2>
      </header>

      <ul class="todo-list">
        {todos.map((todo, index) => (
          <li
            key={todo.id}
            class={{ done: todo.done, active: todo.id === activeId.value }}
            data-index={index}
          >
            <button type="button" onClick={() => (activeId.value = todo.id)}>
              <span>{todo.title}</span>
              {todo.id === activeId.value ? <strong>Active</strong> : <em>{index + 1}</em>}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
};
```

The `.map(...)` callback aliases (`todo`, `index`) are kept in scope for generated type-checker and
LSP virtual TypeScript, so hover, completion, diagnostics, and rename operate on the same bindings
you authored.

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
  `"use vue:vdomx"`) is a compile error.
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
type CardProps = {
  title: string;
};

const Card = ({ title }: CardProps) => (
  <article class="card">
    <h2>{title}</h2>

    <style scoped>{`
      .card {
        border: 1px solid currentColor;
        padding: 12px;
      }
    `}</style>
  </article>
);
```

### Dynamic style values

Prefer normal class bindings, inline style objects, or CSS custom properties for dynamic styling in
JSX/TSX. Template-literal interpolations `${expr}` inside `<style scoped>` are supported and
type-checked, but they are an escape hatch rather than the main authoring style:

```tsx
type BoxProps = {
  color: string;
  gap: number;
};

const Box = ({ color, gap }: BoxProps) => (
  <section
    class="box"
    style={{
      "--box-color": color,
      "--box-gap": `${gap}px`,
    }}
  >
    <p>content</p>

    <style scoped>{`
      .box {
        color: var(--box-color);
        gap: var(--box-gap);
      }
    `}</style>
  </section>
);
```

A `<style>` element **without** `scoped` is treated as a normal element and rendered as-is — it is
not extracted.

`<style scoped>{`.box { color: ${color}; }`}</style>` also works and is covered by the type checker,
but keep it for cases where a scoped stylesheet really needs to reference a component expression.
The literal CSS `v-bind(...)` function syntax used inside an SFC `<style>` block is not a supported
authoring form inside a JSX style block.

## Formatting

Glyph formats JSX/TSX script content with the OXC parser and formatter. In `.vue` files,
`<script lang="jsx">`, `<script lang="tsx">`, and `<script setup lang="tsx">` are parsed as JSX/TSX
instead of falling back to plain TypeScript, so JSX children and TSX annotations are formatted as
real syntax:

```vue
<script setup lang="tsx">
type CardProps = {
  title: string;
  items: string[];
};

const Card = ({ title, items }: CardProps) => (
  <section class="card">
    <h2>{title}</h2>
    {items.map((item) => (
      <span key={item}>{item}</span>
    ))}
  </section>
);
</script>
```

Standalone `.jsx`/`.tsx` modules are discovered by `vize fmt` alongside `.vue` files and formatted
with the same JSX/TSX source-type handling:

```bash
# Formats .vue, .jsx, and .tsx files by default
vize fmt src --write
```

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

When enabled, `vize check` type-checks `.jsx`/`.tsx` Vue components through Canon. The generated
virtual file is plain TypeScript, not TSX, and it preserves the authored component contract:

- the typed first parameter remains the props type;
- `Ctx<Emits, Slots>` remains visible to the setup body and JSX expressions;
- event handlers, bound props, `v-if`/`v-show`, custom directives, and scoped-style interpolation
  expressions, when used, are re-emitted as normal TypeScript reads;
- `v-model` targets are re-emitted as writable self-assignments, so readonly or non-lvalue bindings
  are diagnosed at the binding;
- `.map(...)` list bodies are re-emitted inside the generated callback, so value/index aliases keep
  their inferred element types.

Diagnostics are reported at the **original source locations** (both as JSON for the CLI and through
the LSP), because every meaningful virtual-TS range maps back to the source range you wrote.

```tsx
type FieldProps = {
  model: {
    readonly value: string;
  };
};

const Field = ({ model }: FieldProps) => <input v-model={model.value} />;
```

In the example above, `model.value` is checked as an assignment target. If it is readonly, the
diagnostic lands on `model.value` in the TSX source, not in generated code.

```bash
# Type-check a project including its .jsx/.tsx Vue components.
# .jsx/.tsx files are collected only when typeChecker.jsxTypecheck is enabled.
vize check src
```

Standalone JSX/TSX components lower to plain virtual TypeScript for checking. SFCs that contain
`<script lang="jsx">`, `<script lang="tsx">`, or matching `script setup` blocks are materialized as
`.vue.tsx` virtual files so TypeScript parses JSX syntax in the script block. The LSP and CLI share
the same lowering, so a Corsa diagnostic lands at the identical source range in the editor and on the
command line.

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
AST**. Markup-oriented rules do not reconstruct a synthetic SFC template; they read JSX elements and
attributes directly. Rules that need the Vue template shape, such as `.map(...)` list key checks, run
over the lowered Relief tree. Semantic rules are backed by Croquis, the same analysis layer used for
SFCs.

This means JSX/TSX linting catches the same classes of issues without relying on partial string
matching:

```tsx
const BrokenMedia = () => (
  <article>
    <img src="/avatar.png" />
    <button accessKey="s" autoFocus>
      Save
    </button>
  </article>
);
```

The example above is linted as JSX source:

- `a11y/img-alt` reports the missing `alt`;
- `a11y/no-access-key` reports `accessKey`;
- `a11y/no-autofocus` reports `autoFocus`.

List key rules understand the idiomatic JSX `.map(...)` shape:

```tsx
const KeyedList = ({ rows }: { rows: Array<{ id: string; label: string }> }) => (
  <ul>
    {rows.map((row) => (
      <li key={row.id}>{row.label}</li>
    ))}
  </ul>
);
```

Diagnostics and fixes map to JSX source ranges, so CLI output and editor decorations point at the
element or prop that should change.

```bash
# Lint .vue, .html, .jsx, and .tsx files
vize lint src
```

See [Static Analysis](./static-analysis.md) for the lint and type-check model, and
[Rules](../rules/index.md) for concrete rule output.

## Limitations

Be aware of the current edges:

- **Type-checking is opt-in.** `typeChecker.jsxTypecheck` is `false` by default so mixed Vue/React
  repositories do not accidentally route React TSX through the Vue JSX checker.
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
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx) —
  focused JSX/TSX source examples for compiler, linter, type checker, LSP, and formatter coverage.
