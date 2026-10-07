---
title: Compiler Configuration Reference
---

# Compiler Configuration Reference

For the canonical config location, see [Configuration](./configuration.md).
Standalone file discovery, scopes, and analysis settings remain in the
[standalone reference](./configuration-reference.md).

## Compiler Options

These options live under `compiler` in `defineConfig`; not every integration consumes every field.

| Option              | Values                                  | Common use                                                       |
| ------------------- | --------------------------------------- | ---------------------------------------------------------------- |
| `sourceMap`         | `boolean`                               | Enable source maps in the Vite plugin                            |
| `ssr`               | `boolean`                               | Compile for SSR when not relying on Vite's SSR build flag        |
| `vapor`             | `boolean`                               | Enable Vapor-mode compilation                                    |
| `jsxMode`           | `"vdom"` or `"vapor"`                   | Default output backend for `.jsx`/`.tsx` components              |
| `customRenderer`    | `boolean`                               | Treat lowercase non-HTML tags as custom renderer elements        |
| `customElements`    | `string[]`                              | Tag patterns compiled as custom elements (`Tres*` for TresJS)    |
| `templateSyntax`    | `"standard"`, `"strict"`, or `"quirks"` | Choose warning, error, or Vue-quirk handling for template syntax |
| `scriptExt`         | `"ts"` or `"js"`                        | Preserve TS output or downcompile to JS in the npm build command |
| `mode`              | `"module"` or `"function"`              | Lower-level compiler output mode                                 |
| `prefixIdentifiers` | `boolean`                               | Prefix template identifiers with `_ctx`                          |
| `hoistStatic`       | `boolean`                               | Control static node hoisting                                     |
| `cacheHandlers`     | `boolean`                               | Control event handler caching                                    |
| `isTs`              | `boolean`                               | Parse script blocks as TypeScript                                |
| `runtimeModuleName` | `string`                                | Override runtime import module                                   |
| `runtimeGlobalName` | `string`                                | Override runtime global for function/IIFE-style output           |

For Vite projects, direct plugin options override shared config:

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      vapor: true,
      sourceMap: true,
      customRenderer: true,
      templateSyntax: "standard",
    }),
  ],
});
```

Experimental Vue RFC and backend flags live under top-level `experimentals`; see
[Experimentals](./experimentals.md). Omitted keys, `false`, and `null` are off.

## Template Syntax

`compiler.templateSyntax` defaults to `"standard"`.

- `"standard"` accepts recoverable invalid syntax, emits warnings, and rewrites to valid output.
- `"strict"` reports invalid syntax as compilation errors.
- `"quirks"` preserves template syntax compatibility quirks without additional warnings.

The known cases are:

- `v-for` aliases with an unmatched edge parenthesis. Vue strips a leading `(` or trailing `)`
  from the alias before it splits `value`, `key`, and `index`; standard and strict modes report
  those aliases as malformed, while quirk mode mirrors Vue.
- Non-void HTML elements written with self-closing syntax, such as `<div />` or `<span />`.
  Standard mode warns and rewrites them as empty elements, strict mode errors, and quirk mode keeps
  them as self-closing leaves.

```text
<template>
  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="(item in items">{{ item }}</div>

  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="item) in items">{{ item }}</div>

  <!-- Standard warns and rewrites this as `<div></div>`. Strict errors. Quirk keeps it as a leaf. -->
  <div />
</template>
```

Vue upstream implementation:

- [`forAliasRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/utils.ts#L571)
- [`stripParensRE` in `parseForExpression`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts#L493-L530)

See [Troubleshooting](./troubleshooting.md) for the HTML strict-mode behavior behind invalid
self-closing tags.

## JSX & TSX Output Mode

> For the full authoring API, scoped styles, type-checking, editor support, and limitations, see the
> [JSX & TSX guide](./jsx.md). This section covers only the output-mode config keys.

Vize compiles `.jsx`/`.tsx` Vue components to either Virtual DOM or
[Vapor](https://blog.vuejs.org/posts/vue-vapor) output. `compiler.jsxMode` selects the **global
default** for components that do not opt in explicitly; it defaults to `"vdom"`.

```ts
// vize.config.ts
import { defineConfig } from "@vizejs/vite-plugin";

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

Because each component is routed independently, a **single module can mix both backends**:

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

A directive that begins with `"use vue:"` but does not name a known mode (a typo such as
`"use vue:vdomx"`) is reported as a compile error rather than silently ignored, and two conflicting
mode directives in one component (`"use vue:vapor"` followed by `"use vue:vdom"`) are likewise
diagnosed. Unrelated prologues such as `"use strict"` are left untouched.

## Vue Dialect

`dialect` selects the Vue dialect profile for standalone HTML documents (`.html`/`.htm`):

```json
{
  "dialect": "petite-vue"
}
```

- `"vue"` treats standalone HTML documents as plain Vue-from-CDN documents.
- `"petite-vue"` opts standalone HTML documents into the
  [petite-vue](https://github.com/vuejs/petite-vue) dialect (`v-scope`/`v-effect`
  completions and petite-vue-aware IDE features).

When the key is absent, the dialect is detected structurally per document: a `<script src>`
resolving to the petite-vue package, an inline ES import of `petite-vue`, or a `PetiteVue.createApp`
call. Mentions of petite-vue in comments or prose never switch the dialect, and single-file
components always use the standard Vue dialect.
