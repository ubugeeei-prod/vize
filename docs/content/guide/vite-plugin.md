---
title: Vite Plugin
---

# Vite Plugin

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. Test thoroughly before adopting in non-trivial projects.

> **Bundler status:** `@vizejs/vite-plugin` is currently the most stable bundler integration.
> For rollup / webpack / esbuild use `@vizejs/unplugin`, and for Rspack use `@vizejs/rspack-plugin`.
> Those non-Vite paths are still unstable and should be treated as experimental.

`@vizejs/vite-plugin` provides native-speed Vue SFC compilation for Vite projects. It is designed as a **drop-in replacement** for `@vitejs/plugin-vue` — your existing Vue components work without modification.

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the packages:

```bash
vp install -D @vizejs/vite-plugin
```

Add `vize` as a direct dependency only if your project imports shared config helpers from `"vize"`
or runs the npm CLI through `vp exec vize`.

## Basic Usage

```javascript
// vite.config.js
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

That's it. Replace `@vitejs/plugin-vue` with `@vizejs/vite-plugin` and your project compiles through Rust.

For most projects, keep direct plugin options small and put stable compiler settings in
`vize.config.ts`.

## Shared Config

The recommended shared entry point is `vize`. A single `vize.config.*` file is read by both the npm CLI and `@vizejs/vite-plugin`.

```bash
vp install -D vize
```

Supported config files:

- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.pkl`
- `vize.config.json`

TypeScript config:

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
    vueParserQuirks: false,
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

PKL config:

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}
```

JSON config with schema:

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "vite": {
    "scanPatterns": ["src/**/*.vue"]
  }
}
```

Importing `defineConfig` from `@vizejs/vite-plugin` still works for backward compatibility, but `import { defineConfig } from "vize"` is the shared path going forward.

See [Configuration](./configuration.md) for the full shared config shape.

## Compiler Options

Direct options passed to `vize()` override `vize.config.*`.

```ts
vize({
  sourceMap: true,
  ssr: false,
  vapor: false,
  customRenderer: false,
  vueParserQuirks: false,
  scanPatterns: ["src/**/*.vue"],
  ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
});
```

| Option                 | Where to set it                                           | Description                                                                                               |
| ---------------------- | --------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| `sourceMap`            | `compiler.sourceMap` or `vize({ sourceMap })`             | Generate source maps. Defaults to development on, production off.                                         |
| `ssr`                  | `compiler.ssr` or `vize({ ssr })`                         | Force SSR compilation when Vite's SSR build flag is not enough.                                           |
| `vapor`                | `compiler.vapor` or `vize({ vapor })`                     | Compile templates through the Vapor backend.                                                              |
| `customRenderer`       | `compiler.customRenderer` or `vize({ customRenderer })`   | Treat lowercase non-HTML tags as custom renderer elements. Useful for renderer ecosystems such as TresJS. |
| `vueParserQuirks`      | `compiler.vueParserQuirks` or `vize({ vueParserQuirks })` | Match Vue parser quirks for known edge cases.                                                             |
| `include`              | `vite.include` or `vize({ include })`                     | Files that the plugin should compile.                                                                     |
| `exclude`              | `vite.exclude` or `vize({ exclude })`                     | Files that the plugin should ignore.                                                                      |
| `scanPatterns`         | `vite.scanPatterns` or `vize({ scanPatterns })`           | Glob patterns used for startup pre-compilation.                                                           |
| `ignorePatterns`       | `vite.ignorePatterns` or `vize({ ignorePatterns })`       | Glob patterns skipped during startup pre-compilation.                                                     |
| `configMode`           | `vize({ configMode })`                                    | Use `"root"`, `"auto"`, or `false` for shared config loading.                                             |
| `configFile`           | `vize({ configFile })`                                    | Load a specific config file.                                                                              |
| `handleNodeModulesVue` | `vize({ handleNodeModulesVue })`                          | Compile `.vue` files imported from `node_modules` on demand.                                              |
| `debug`                | `vize({ debug })`                                         | Print plugin debug logs.                                                                                  |

Common recipes:

```ts
// Vapor-oriented build
vize({ vapor: true });

// TresJS or another custom renderer
vize({ customRenderer: true });

// Existing templates that rely on Vue's v-for alias edge-paren behavior
vize({ vueParserQuirks: true });

// Monorepo package with explicit scan roots
vize({
  root: import.meta.dirname,
  scanPatterns: ["src/**/*.vue", "examples/**/*.vue"],
});
```

## How It Works

The plugin intercepts `.vue` file requests and compiles them using Vize's Rust-native pipeline through Node.js NAPI bindings:

1. **Pre-compilation** — At `buildStart`, the plugin discovers all `.vue` files and compiles them in batch using `compileBatch`. This triggers Rayon-based parallel compilation on the Rust side, processing all files across all CPU cores simultaneously.

2. **On-demand compilation** — During development, if a `.vue` file is requested that isn't in the cache (e.g., dynamically imported), it's compiled on-the-fly via `compileFile`.

3. **HMR** — When a `.vue` file changes, only that file is recompiled. The plugin detects whether the change is style-only and applies a style-only HMR update when possible, avoiding a full component re-render.

4. **CSS extraction** — In production builds, all scoped CSS from Vue components is extracted and merged into `assets/vize-components.css`, eliminating per-component style injection overhead.

### Compilation Pipeline

```
.vue file
  → Armature (Parser)          — Tokenizes and parses the SFC structure
  → Croquis (Semantic Analysis) — Analyzes template expressions and bindings
  → Atelier (Compilation)       — Generates optimized JavaScript output
  → Vitrine (NAPI Binding)      — Delivers the result to Node.js
  → Vite module graph            — Served as a virtual module
```

The same semantic analysis layer is reused by linting and type checking. See
[Static Analysis](./static-analysis.md) for the diagnostic side of the pipeline.

## Comparison

| Feature               | @vitejs/plugin-vue | @vizejs/vite-plugin                |
| --------------------- | ------------------ | ---------------------------------- |
| Language              | JavaScript         | Rust (NAPI)                        |
| SFC Compilation       | Yes                | Yes                                |
| Template Compilation  | Yes                | Yes                                |
| Script Setup          | Yes                | Yes                                |
| CSS Scoping           | Yes                | Yes                                |
| SSR Support           | Yes                | Yes                                |
| HMR                   | Yes                | Yes (style-only optimization)      |
| Batch Pre-compilation | No                 | Yes (parallel via Rayon)           |
| CSS Extraction        | Per-component      | Merged single file                 |
| Vapor Mode            | Experimental       | First-class (`vize_atelier_vapor`) |

## Advanced Features

### Batch Pre-compilation

Unlike `@vitejs/plugin-vue`, which compiles each `.vue` file on first request, Vize pre-compiles all discovered `.vue` files at build start using multi-threaded batch compilation. This means:

- **Dev server startup** — All components are ready before the first page load
- **Production builds** — Maximum parallelism from the start

### Static Asset Rewriting

The plugin automatically rewrites static asset URLs in templates. For example:

```vue
<template>
  <img src="./logo.png" />
</template>
```

The `src` attribute is hoisted to an import statement, allowing Vite to process the asset through its asset pipeline (hashing, optimization, etc.).

### Define Replacement

Vite normally skips `import.meta.*` replacement for virtual modules (prefixed with `\0`). Vize's plugin manually applies define replacements to ensure `import.meta.env.*` values work correctly in compiled Vue components.

### Per-Environment Isolation

For Nuxt compatibility, the plugin isolates `define` values per Vite environment (client vs. server/SSR). This prevents client-side environment values from leaking into SSR output.

## Nuxt Compatibility

The plugin exposes a compatibility shim for tools that probe for `@vitejs/plugin-vue`'s API (like Nuxt). This means Vize works with Nuxt's built-in Vue integration without special configuration:

```typescript
// nuxt.config.ts — using the dedicated Nuxt module
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

See [Nuxt Integration](../integrations/nuxt.md) for more details.

## Notes

- The plugin requires `@vizejs/native` for Node.js NAPI bindings (installed automatically as a dependency)
- Vapor mode compilation is available via `vize_atelier_vapor` (Vue 3.6+)
- DOM (VDom) compilation uses `vize_atelier_dom`
- The plugin supports `virtual:vize-styles` for importing all compiled CSS as a module
- For experimental rollup / webpack / esbuild / Rspack support, see [Experimental Bundler Integrations](./unplugin.md)
