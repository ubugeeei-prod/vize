---
title: Babel JSX Compatibility
---

# Babel JSX Compatibility

> **Status:** opt-in and off by default. `compiler.jsxCompat` is read by the config loader and
> honoured by the native/WASM `compileJsx` bindings and the Vize bundler plugins.

Vize compiles `.jsx` and `.tsx` through its own compiler crates, so the output is
template-compiler shaped: a block tree, `v-if` / `v-for` lowered out of the JavaScript, and patch
flags on every node. [`@vue/babel-plugin-jsx`](https://github.com/vuejs/babel-plugin-jsx) does none
of that — it emits bare `createVNode` calls, never opens a block, leaves `&&`, `?:` and `.map()` as
plain JavaScript, and by default emits no patch flags at all.

Most of that difference is invisible at runtime. The rest is what this switch exists for: a project
migrating off the babel plugin needs a way to ask for the plugin's semantics instead of Vize's.
`compiler.jsxCompat: "babel"` is that switch.

This page is about **compatibility semantics**. For the authoring API, the type surface, and the
Vapor/VDOM output selector, see the [JSX & TSX guide](./jsx.md).

## Enabling it

```json
{
  "compiler": {
    "jsxCompat": "babel"
  }
}
```

The key accepts `"native"` (the default) and `"babel"`. Any other value falls back to `"native"`
rather than failing the build, matching how an unrecognised `jsxMode` is handled: a stray config
value must never block compilation.

The same value is accepted directly by the `compileJsx` bindings:

```js
import { compileJsx } from "@vizejs/native";

const result = compileJsx(source, {
  filename: "App.tsx",
  lang: "tsx",
  jsxCompat: "babel",
});
```

`@vizejs/wasm` exposes the same `jsxCompat` option. The Vite, unplugin, Rspack, and Nuxt entry
points forward their configured `jsxCompat` value to `compileJsx`, and their option types accept
`jsxCompat` directly alongside `jsxMode` and `vapor`.

## Why it is opt-in and project-level

**Off by default.** `"native"` is the default and has to stay the default. Flipping it would
silently change the emitted output for every existing Vize project, none of which asked for babel
semantics.

**Project-level, with no per-component form.** `jsxMode` can be selected per component with a
`"use vue:vapor"` / `"use vue:vdom"` prologue, because VDOM and Vapor components coexist happily in
one module — each is an independent render function. Compatibility mode is not like that. It
changes **module-level** output shape: the babel plugin rewrites the JSX expression in place, so
`const A = () => <div />` stays a `const A = …`, while Vize emits a standalone `render` export. A
module compiled half in compat mode and half out of it would emit two mutually incompatible module
shapes from a single file. Compat is therefore configured once for the project and deliberately has
no directive prologue.

## Plugin option mapping

The babel plugin's own options have no config-file spelling in Vize. Each one is a parameter of a
`compile_jsx_with_babel_*` entry point on the
[`vize_atelier_jsx`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_jsx) crate,
and every one of them is inert unless `jsxCompat` is `"babel"`.

| `@vue/babel-plugin-jsx` | Vize entry point                            |
| ----------------------- | ------------------------------------------- |
| `transformOn`           | `BabelJsxOptions::transform_on`             |
| `pragma`                | `compile_jsx_with_babel_pragma`             |
| `mergeProps`            | `compile_jsx_with_babel_merge_props`        |
| `isCustomElement`       | `BabelJsxCustomizations::is_custom_element` |
| `enableObjectSlots`     | `compile_jsx_with_babel_object_slots`       |
| any combination         | `compile_jsx_with_babel_customizations`     |

Two plugin options are not in that table:

- **`optimize`** has no Vize equivalent, because Vize's output is always optimized — which is what
  the plugin's `optimize: true` produces. The plugin's default is `optimize: false`, and its own
  README warns that turning it on "may skip certain re-renders", so the gap compat mode has to
  close is the _unoptimized_ direction: emitting patch-flag-free output.
- **`resolveType`** is not implemented; see "What is deferred" below.

`enableObjectSlots` defaults to `true` in the plugin and in Vize's compat lane: a lone identifier or
call expression passed as a component's only child may already be a slots object, so it is checked
at runtime. Passing `false` always treats that value as the raw default-slot child.

## Where the mode does not apply

**Vapor output.** `@vue/babel-plugin-jsx` is a vdom-era plugin: every output shape it defines is a
`createVNode` tree, and it has no Vapor equivalent. `jsxCompat: "babel"` combined with
`jsxMode: "vapor"` therefore has no defined meaning, and is rejected with a diagnostic rather than
silently ignored:

```text
compiler.jsxCompat: "babel" is not supported with Vapor output: @vue/babel-plugin-jsx has no
Vapor equivalent. Use jsxMode "vdom" for babel compatibility, or drop jsxCompat to use Vize's own
Vapor semantics.
```

**SSR output.** The plugin's options describe client vnode trees. SSR compilation therefore
withholds the whole babel lane — the `transformOn` and `enableObjectSlots` helpers, the
`isCustomElement` predicate, `mergeProps: false`, and every babel-only lowering — and uses Vize's
own SSR semantics instead of emitting a half-applied mixture.

Both are deliberate answers, recorded in the crate so they are not relitigated.

## What is deferred

Two corpus rows are recorded as `deferred` rather than divergent, because each is waiting on
unrelated compiler work rather than on compat mode itself:

| Row                       | What babel does                          | What it is waiting on                                                                                                                                                                                 |
| ------------------------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `options/resolve_type_on` | appends `{ props: { … }, name: "A" }`    | type-driven props/emits inference, which needs the type resolution tracked on [#1497](https://github.com/ubugeeei-prod/vize/issues/1497) / [#1502](https://github.com/ubugeeei-prod/vize/issues/1502) |
| `slots/dynamic_slot_name` | emits a computed key, `{ [n]: () => … }` | dynamic-slot lowering; Vize currently warns and drops the slot                                                                                                                                        |

## How compatibility is measured

Compatibility is measured against the **real plugin**, not from memory. The corpus is compiled by a
pinned `@vue/babel-plugin-jsx`, its output is recorded as committed ground truth, and the Rust suite
snapshots that recording beside Vize's output with an explicit verdict per row.

| Artifact                                                          | Role                                                     |
| ----------------------------------------------------------------- | -------------------------------------------------------- |
| `crates/vize_atelier_jsx/tests/babel_compat/fixtures/corpus.json` | the inputs, and the plugin options each is compiled with |
| `crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs`           | runs the corpus through the real plugin                  |
| `crates/vize_atelier_jsx/tests/babel_compat_oracle.rs`            | snapshots babel's output beside Vize's, per row          |
| `crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md`         | the prose form of the verdict table, and the totals      |

The row-by-row verdicts, the global divergences that hold for nearly every row (module shape, block
tree, patch flags, un-lowered control flow), and the current totals all live in
[`BABEL_COMPAT_INVENTORY.md`](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md).
Those totals are pinned by the `babel_compat_verdict_totals` test, so they cannot drift from the
corpus — which is why this page quotes none of them. Read them at the source.

To regenerate or verify the recording locally:

```bash
node crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs --check
cargo test -p vize_atelier_jsx --test babel_compat_oracle
node --test tests/tooling/babel-jsx-oracle.test.ts
```

## See also

- [JSX & TSX](./jsx.md) — the authoring API, typed props and emits, scoped styles, and `jsxMode`.
- [Configuration](./configuration.md) — every `compiler.*` key and the config file lookup order.
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx) — a runnable JSX/TSX project.
