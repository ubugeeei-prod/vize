---
title: Vite Plugin
---

# Vite Plugin

> [!WARNING]
> Vize is under active development and is not yet ready for production use. Test thoroughly before adopting it in non-trivial projects.

> **Bundler status:** `@vizejs/vite-plugin` is currently the most stable bundler integration.
> For rollup / webpack / esbuild use `@vizejs/unplugin`, and for Rspack use `@vizejs/rspack-plugin`.
> Those non-Vite paths are still unstable and should be treated as experimental.

`@vizejs/vite-plugin` provides native-speed Vue SFC compilation for Vite projects. It is designed as a **drop-in replacement** for `@vitejs/plugin-vue` on Vue 3 SFCs — your existing `<script setup>` and Options API components work without modification.

## Start with Vite+

For combined compilation, native Vue checks, linting, and formatting, keep settings in
`vite.config.ts` with the [Vite+ helper](./vite-plus.md):

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({});
```

[Migration](./migration.md) shows exactly which import and plugin calls to replace.
The ordinary Vite path below replaces only the compiler.

## Drop-in Scope

The drop-in claim is deliberately bounded, so it is worth stating what it does and does not cover:

- **In scope** — Vue 3 single-file components compiled through Vite, in both `<script setup>` and
  Options API style, including scoped styles, CSS Modules, SSR, and HMR.
- **Incubating** — Vue 2 and 2.7 (`vue.version: "2"` / `"2.7"`, or `vize({ vueVersion: 2 })`). The legacy dialects are off by
  default and non-invasive: Vize does not compile `.vue` files in those modes, so your existing Vue
  compiler, `vue-loader`, or Nuxt 2 setup stays in charge. See
  [Compiler Options](#compiler-options).
- **Out of scope** — webpack, rollup, esbuild, and Rspack. `@vizejs/unplugin` and
  `@vizejs/rspack-plugin` exist but are experimental and carry no drop-in guarantee; see
  [Experimental Bundler Integrations](./unplugin.md).
- **Not complete yet** — plugin-option parity with `@vitejs/plugin-vue`. `include`, `exclude`, and
  `isProduction` are honored today; every other documented option, the reason it diverges, and the
  remaining gaps are tracked in
  [#3227](https://github.com/ubugeeei-prod/vize/issues/3227).

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the packages:

```bash
vp install -D @vizejs/vite-plugin
```

Add `vize` as a direct dependency only if your project imports shared config helpers from `"vize"`
or exposes package scripts such as `vize:lint` and `vize:check`.

## Basic Usage

```javascript
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
export default defineConfig({}).withVue({});
```

Replace `@vitejs/plugin-vue` with `@vizejs/vite-plugin` to compile through Rust. For combined Vite+ check, lint, format, and build tasks, [use the Vite+ `defineConfig()`](./vite-plus.md).

## TypeScript Vue Imports

Add the plugin package to `compilerOptions.types` to make direct `.vue` imports resolvable by
TypeScript without writing a local `env.d.ts` shim:

```json
{
  "compilerOptions": {
    "types": ["vite/client", "@vizejs/vite-plugin"]
  }
}
```

This does not require adding `vize` as a direct project dependency.

For Vite Plus projects, keep the Vite Plus client type and append the plugin package:

```json
{
  "compilerOptions": {
    "types": ["vite-plus/client", "@vizejs/vite-plugin"]
  }
}
```

## Shared native settings

Keep root settings in `defineConfig({ ... })` and add native Vue settings directly with
`.withVue({ ... })` from `@vizejs/vite-plugin/vite-plus`. Export that chain without storing it in
an intermediate variable. The CLI and editor read the same project settings.

Keep TypeScript `compilerOptions`, standard `extends` and `references` in
`tsconfig.json`; Vue template options belong in `vueCompilerOptions`.
Dedicated `vize.config.*` files and top-level `vize:` settings are retired.
See [Configuration](./configuration.md) for migration and
[Shared Vue configuration](./vue-configuration.md) for scoped entries.

## Compiler Options

Direct options passed to `vize()` take precedence, followed by inline `config`, then discovered project settings and defaults. Use `vue:` in the Vite configuration; dedicated `vize.config.*` files are no longer supported. `config: false` opts out of automatic discovery.

```ts
vize({
  vueVersion: 3,
  sourceMap: true,
  ssr: false,
  vapor: false,
  customRenderer: false,
  templateSyntax: "standard",
  scanPatterns: ["src/**/*.vue"],
  ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
});
```

| Option                              | Where to set it                                                                             | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ----------------------------------- | ------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vueVersion`                        | `vize({ vueVersion })`                                                                      | Set `0.11`, `1`, `2`, or `"legacy"` to run in non-invasive legacy Vue compatibility mode and leave SFC compilation to the host compiler.                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `sourceMap`                         | `vue.compiler.sourceMap` or `vize({ sourceMap })`                                           | Generate source maps whose `sources` name the authored `.vue` file. Defaults to development on, production off unless `build.sourcemap` is set.                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `ssr`                               | `vue.compiler.ssr` or `vize({ ssr })`                                                       | Force SSR compilation when Vite's SSR build flag is not enough.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `vapor`                             | `vue.compiler.vapor` or `vize({ vapor })`                                                   | Compile templates through the Vapor backend.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `jsxMode`                           | `vue.compiler.jsxMode` or `vize({ jsxMode })`                                               | Default output backend (`"vdom"` / `"vapor"`) for `.jsx`/`.tsx` components. Per-component `"use vue:*"` directives override it.                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `customRenderer`                    | `vue.compiler.customRenderer` or `vize({ customRenderer })`                                 | Treat lowercase non-HTML tags as custom renderer elements. Does not match PascalCase tags such as `<TresMesh>`.                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `customElements`                    | `vue.compiler.customElements` or `vize({ customElements })`                                 | Tag patterns compiled as custom elements instead of Vue components. Use `["Tres*"]` for TresJS PascalCase renderer tags.                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `templateSyntax`                    | `vue.compiler.templateSyntax` or `vize({ templateSyntax })`                                 | Choose `"standard"`, `"strict"`, or `"quirks"` template syntax handling.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `whitespace`                        | `vue.compiler.whitespace`, `vize({ whitespace })`, or `template.compilerOptions.whitespace` | Use Vue's `"condense"` (default) or `"preserve"` template whitespace mode. Vize also accepts `"vue2-line-breaks"` through its own compiler/Vite option to keep a newline after text or interpolation when Vue 3 would condense it to one space. This is a narrow migration mode, not full Vue 2 whitespace behavior. For Vue 2.7 `whitespace: "preserve"` output (as used by Vue 2, vue-loader 15 and Nuxt 2), use `"vue2-preserve"`: whitespace next to text and interpolations is kept as written, and whitespace-only runs between elements become one space. |
| `template.compilerOptions.comments` | `vue.compiler.comments` or `vize({ template: { compilerOptions: { comments } } })`          | Preserve authored template comments in development by default and omit them in production. Set `true` or `false` to override either default. Direct native calls use `templateComments` and default to `false`.                                                                                                                                                                                                                                                                                                                                                  |
| `experimentals`                     | `vue.experimentals` or `vize({ experimentals })`                                            | Fully opt-in Vue RFC and backend experiments. Missing keys, `false`, and `null` are off.                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `include`                           | `vue.vite.include` or `vize({ include })`                                                   | Files that the plugin should compile.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `exclude`                           | `vue.vite.exclude` or `vize({ exclude })`                                                   | Files that the plugin should ignore.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `scanPatterns`                      | `vue.vite.scanPatterns` or `vize({ scanPatterns })`                                         | Opt-in startup pre-compilation: glob patterns compiled at `buildStart`. Unset by default: each environment compiles the components its module graph imports, in batches.                                                                                                                                                                                                                                                                                                                                                                                         |
| `ignorePatterns`                    | `vue.vite.ignorePatterns` or `vize({ ignorePatterns })`                                     | Glob patterns skipped during startup pre-compilation.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `configMode`                        | `vize({ configMode })`                                                                      | Use `"root"`, `"auto"`, or `false` for shared config loading.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `configFile`                        | `vize({ configFile })`                                                                      | Load a specific config file.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `config`                            | `vize({ config })`                                                                          | Inline shared config for Vite Plus runtime settings.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `handleNodeModulesVue`              | `vize({ handleNodeModulesVue })`                                                            | Compile `.vue` files imported from `node_modules` on demand.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `debug`                             | `vize({ debug })`                                                                           | Print plugin debug logs.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |

For the full `experimentals` flag table, opt-in rules, aliases, and shared-config precedence, see
[Experimentals](./experimentals.md).

Common recipes:

```ts
// Vapor-oriented build
vize({ vapor: true });

// TresJS PascalCase renderer tags
vize({
  customRenderer: true,
  customElements: ["Tres*", "primitive"],
});

// Existing templates that rely on parser edge cases, such as
// v-for alias edge parens or `<div />` as a self-closing leaf
vize({ templateSyntax: "quirks" });

// Monorepo package with explicit scan roots
vize({
  root: import.meta.dirname,
  scanPatterns: ["src/**/*.vue", "examples/**/*.vue"],
});

// Legacy Vue / Nuxt 2 Bridge project with an existing host compiler plugin
vize({ vueVersion: 2 });
```

`vueVersion: 0.11`, `1`, `2`, and `"legacy"` are host-compiler compatibility modes. Vize does not
compile `.vue` files in these modes, does not expose the Vue 3 `vite:vue` API shim, and does not
inject Vue 3 bundler feature flags. Keep the existing Vue compiler plugin, `vue-loader`, or Nuxt 2's
own compiler configured normally.

## How It Works

The plugin intercepts `.vue` file requests and compiles them using Vize's Rust-native pipeline through Node.js NAPI bindings:

1. **Graph-driven batches** — Each environment compiles the `.vue` files its module graph imports, and no others. A bundler resolves a module's imports before it loads them, so the first component a module requests is compiled together with its siblings in one `compileBatch` call. That call runs Rayon-based parallel compilation on the Rust side. An environment that imports no component (a worker, Nuxt's Nitro pass) compiles nothing, and the dev server starts without compiling.

2. **Startup pre-compilation (opt-in)** — With `scanPatterns` set, the plugin also compiles every matching file in batches at `buildStart`, before anything is requested.

3. **HMR** — When a `.vue` file changes, only that file is recompiled. The plugin detects whether the change is style-only and applies a style-only HMR update when possible, avoiding a full component re-render.

4. **CSS extraction** — In production builds, all scoped CSS from Vue components is extracted and merged into `assets/vize-components.css`, eliminating per-component style injection overhead.

### Compilation Pipeline

```diagram
.vue file
  → Parser            — Tokenizes and parses the SFC structure
  → Semantic analysis — Resolves template expressions and bindings
  → Code generation   — Emits optimized JavaScript (VDOM, SSR, or Vapor)
  → Node.js binding   — Hands the result to Vite
  → Vite module graph — Served as a virtual module
```

The same semantic analysis layer is reused by linting and type checking. See
[Static Analysis](./static-analysis.md) for the diagnostic side of the pipeline.

## Comparison

| Feature               | @vitejs/plugin-vue      | @vizejs/vite-plugin                                                                                    |
| --------------------- | ----------------------- | ------------------------------------------------------------------------------------------------------ |
| Language              | JavaScript              | Rust (NAPI)                                                                                            |
| SFC Compilation       | Yes                     | Yes                                                                                                    |
| Template Compilation  | Yes                     | Yes                                                                                                    |
| Script Setup          | Yes                     | Yes                                                                                                    |
| CSS Scoping           | Yes                     | Yes                                                                                                    |
| SSR Support           | Yes                     | Yes                                                                                                    |
| HMR                   | Yes                     | Yes (style-only optimization)                                                                          |
| Batch Compilation     | No                      | Yes (parallel via Rayon)                                                                               |
| CSS Extraction        | Per-component           | Merged single file                                                                                     |
| Vapor Mode            | Experimental            | Yes, experimental                                                                                      |
| Plugin Options        | Full documented surface | `include` / `exclude` / `isProduction`; see [#3227](https://github.com/ubugeeei-prod/vize/issues/3227) |

## Advanced Features

### Batch Compilation

`@vitejs/plugin-vue` compiles each `.vue` file by itself on first request. Vize compiles the same files, the ones a module graph imports, but hands them to the native compiler in multi-threaded batches of up to `precompileBatchSize` files:

- **Dev server** — The server starts without compiling. A page's components are compiled together when the page first asks for one of them, and the components a hot update invalidates are compiled together when they are requested again.
- **Production builds** — Each environment compiles what it imports. A component nothing imports is never read.

To compile files before anything requests them, set `scanPatterns`. The plugin then pre-compiles every matching file at `buildStart`, in every environment, and keeps the results in `node_modules/.vize` for the next run.

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

### Style Preprocessors and Pug

`<style lang="scss">`, `sass`, `less` and `stylus` follow Vue's order: the preprocessor runs first, then Vize scopes its output and resolves `v-bind()`. `<style module>` class maps come from your bundler's CSS-module pipeline. Install the preprocessor package (`sass` or `sass-embedded`, `less`, `stylus`) in your project, as you would with `@vitejs/plugin-vue`. `<template lang="pug">` is compiled by Vize itself, so no extra package is needed. The legacy `<template lang="jade">` spelling is accepted as an alias on the same surfaces.

Every Vize tool resolves `lang` through one registry, ASCII case-insensitively: `jade` is `pug`, `styl` is `stylus`, and `postcss`, an empty `lang` or no `lang` is plain CSS. A template `lang` with no Vize grammar (for example `haml`) is never read as HTML: the compiler reports `TEMPLATE_LANG_UNSUPPORTED` at the `lang` value, and the linter and type checker leave the template out.

| Surface                                                 | scss / sass / less / stylus                                                                                                         | pug / jade                          |
| ------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| Vite                                                    | Vite preprocesses; Vize scopes the result                                                                                           | Native                              |
| `@vizejs/unplugin` (webpack, Rollup, Rolldown, esbuild) | webpack uses your loaders; Rollup, Rolldown and esbuild run your installed preprocessor                                             | Native                              |
| `@vizejs/rspack-plugin`                                 | Your loaders, with css-loader or native CSS                                                                                         | Native                              |
| `vize build`                                            | Runs your installed preprocessor through Node                                                                                       | Native                              |
| `vize fmt`                                              | Preprocessor blocks are left byte-identical                                                                                         | Formatted                           |
| `vize lint`                                             | Template rules apply; style blocks are not linted as CSS                                                                            | Template rules apply                |
| LSP                                                     | CSS completion and hover (indented syntax included); `v-bind()` go-to-definition; CSS-module class completion in every preprocessor | Tag completion and go-to-definition |

#### Supported Pug and Jade subset

Pug support is built into Vize; it does not execute the project's JavaScript or
require a local `pug` package. `lang="jade"` is read by the same implementation
under a jade 1.x profile. The supported deterministic subset includes:

- elements, components, class/id shorthand, attributes, Vue directives,
  interpolation, slots and text blocks;
- build-time `if` / `unless` / `else`, `each` / `for`, literal variables and
  values that can be evaluated without calling project code;
- mixin calls, mixin blocks, `attributes` and `&attributes` when their inputs
  are build-time values; and
- relative `.pug` includes plus raw-text includes. Included files are Vite
  dependencies, so editing one invalidates the owning SFC during HMR.

The compiler checks this subset against Pug-rendered HTML in DOM, SSR and Vapor
modes. Generated module source maps, diagnostics, type-check results and editor
ranges point back to the authored Pug or Jade source. The linter runs template
rules on that authored view. A pinned dho-web-client Pug SFC also checks its
rule, severity, count and upstream message prefix against eslint-plugin-vue
10.9.2 on the exact pug 3.0.4-rendered HTML twin; Patina separately pins its
full message with authored element context. The formatter intentionally keeps
the Pug program opaque and only normalizes its outer indentation. Completion,
hover, definition, references and rename are available inside supported inline templates.

Known gaps fail closed instead of compiling a partial template. Vize refuses
arbitrary executable JavaScript, `while` / `case`, template inheritance,
filters, filtered or rooted includes, and external `<template src>` Pug. Pug
template autofixes are withheld because a derived HTML edit cannot always be
mapped back losslessly, and there is no language-aware Pug printer. These
boundaries apply equally to `pug` and `jade`.

#### The jade profile

Vue's own toolchain renders `lang="jade"` with the legacy `jade` 1.x package,
not pug, and the two languages differ. Under `lang="jade"` Vize keeps the
author's intent and says so, never silently; `lang="pug"` behaves exactly as
pug 3 does.

| jade 1.x construct | Vize under `lang="jade"` |
| --- | --- |
| `- each item in items` / `- for …` | Iterates, with a warning and a fix that drops the dash (pug 3 refuses it). |
| a bodiless `mixin name(args)` | Calls the mixin, with a warning and a fix to `+name(args)`. |
| `each item of items` | Iterates as pug 3 does, warning that jade 1.x renders a literal `<each>` element. |
| `a(href="/users/#{id}")` | Keeps the text as written, warning that jade 1.x interpolates it at build time to `undefined`, with a fix to `` :href="`/users/${id}`" ``. |

The compiler reports each as a `JADE_PROFILE` warning at the construct. The
linter reports them, and the `lang` value itself, through
`vue/prefer-pug-lang`, whose fix renames the `lang` to `pug` once nothing
jade-only is left; configure or turn off that rule to keep jade.

The Real Project Matrix exercises Pug-heavy applications including
dho-web-client and Wave UI across compile, type-check, lint and format checks;
the same workflow carries the Pug LSP and SSR observers.

## Nuxt Compatibility

The plugin exposes a compatibility shim for tools that probe for `@vitejs/plugin-vue`'s API (like Nuxt). This means Vize works with Nuxt's built-in Vue integration without special configuration:

```ts
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
- Vapor Mode compilation is available for Vue 3.6+ (experimental)
- The plugin supports `virtual:vize-styles` for importing all compiled CSS as a module
- `.jsx`/`.tsx` Vue components are compiled automatically through the same plugin — see the [JSX & TSX](./jsx.md) guide
- `.tsrx` modules are compiled through the same TSRX compiler as `vize build` unless `exclude` matches them (`include` does not apply: no other plugin reads `.tsrx`). Their TypeScript is erased as in `.vue` scripts. A module the compiler refuses fails the build with every diagnostic at its line and column; the DOM target is emitted, while SSR builds, Vapor and source maps are refused for now
- For experimental rollup / webpack / esbuild / Rspack support, see [Experimental Bundler Integrations](./unplugin.md)

<span id="optional-standalone-shared-config"></span>
