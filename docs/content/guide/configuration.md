---
title: Configuration
---

# Configuration

Vize uses `vize.config.*` for shared npm package commands, Vite plugin, and Rust CLI settings.

## Config Files

The npm package commands and `@vizejs/vite-plugin` load these files from the project root in this
priority order:

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

The Rust CLI reads the same config file names in the order above for command-native settings such as
`check`, `lint`, `lsp`, and `fmt`.

## TypeScript Config

```ts
import { defineConfig } from "vize";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  compiler: {
    sourceMap: mode !== "production",
    ssr: isSsrBuild,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: {
    include: [/\.vue$/],
    exclude: [/node_modules/],
    scanPatterns: ["src/**/*.vue"],
    ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
  },
  linter: {
    enabled: command !== "build",
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
  },
  formatter: {
    printWidth: 100,
    singleQuote: false,
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
}));
```

## Experimental Flat Entries

Monorepos can describe root defaults and package-scoped overrides with `entries`. Plain object
configs are normalized to one entry internally, and array exports are accepted by `defineConfig` for
ESLint-flat-config-style authoring.

```ts
export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  entries: [
    {
      name: "web app",
      basePath: "apps/web",
      files: ["src/**/*.vue"],
      typeChecker: {
        tsconfig: "tsconfig.app.json",
      },
    },
    {
      name: "ui package",
      basePath: "packages/ui",
      files: ["src/**/*.vue"],
      formatter: {
        singleQuote: true,
      },
    },
  ],
});
```

## PKL Config

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
  vapor = false
  customRenderer = false
  templateSyntax = "standard"
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}

linter {
  preset = "happy-path"
}

typeChecker {
  enabled = true
  strict = true
}

entries = new Listing {
  new ConfigEntry {
    name = "web app"
    basePath = "apps/web"
    files = new Listing { "src/**/*.vue" }
    typeChecker {
      tsconfig = "tsconfig.app.json"
    }
  }
}

lsp {
  lint = true
  typecheck = false
  editor = false
  formatting = false
}
```

## JSON Config

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "compiler": {
    "sourceMap": true,
    "vapor": false,
    "customRenderer": false,
    "templateSyntax": "standard"
  },
  "vite": {
    "scanPatterns": ["src/**/*.vue"]
  },
  "linter": {
    "preset": "happy-path"
  },
  "typeChecker": {
    "enabled": true,
    "strict": true
  },
  "musea": {
    "include": ["src/**/*.art.vue"],
    "basePath": "/__musea__"
  }
}
```

## Compiler Options

These options live under `compiler`. They are schema-backed and shared through `defineConfig`; not
every integration consumes every field yet.

| Option              | Values                                  | Common use                                                       |
| ------------------- | --------------------------------------- | ---------------------------------------------------------------- |
| `sourceMap`         | `boolean`                               | Enable source maps in the Vite plugin                            |
| `ssr`               | `boolean`                               | Compile for SSR when not relying on Vite's SSR build flag        |
| `vapor`             | `boolean`                               | Enable Vapor-mode compilation                                    |
| `jsxMode`           | `"vdom"` or `"vapor"`                   | Default output backend for `.jsx`/`.tsx` components              |
| `customRenderer`    | `boolean`                               | Treat lowercase non-HTML tags as custom renderer elements        |
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
`"use vue:vdomm"`) is reported as a compile error rather than silently ignored, and two conflicting
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

When the key is absent, the dialect is detected structurally per document: a
`<script src>` resolving to the petite-vue package, an inline ES import of
`petite-vue`, or a `PetiteVue.createApp` call. Mentions of petite-vue in
comments or prose never switch the dialect. Single-file components always use
the standard Vue dialect.

## Static Analysis Options

Use `linter` for the npm lint path:

```ts
export default defineConfig({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
});
```

Use `typeChecker` for the npm check path:

```ts
export default defineConfig({
  typeChecker: {
    enabled: true,
    strict: true,
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    // Resolve Vue 3 Options API template bindings (data/computed/methods/
    // inject/setup/props). Officially supported in Vue 3 and default-on
    // (matches vue-tsc). Set to `false` to opt out.
    optionsApi: true,
  },
});
```

`typeChecker.optionsApi` opts the type checker into resolving Vue 3 Options API
template bindings (`data`/`computed`/`methods`/`inject`/`setup`/`props` declared
on a normal `<script> export default { ... }`). Options API is officially
supported in Vue 3, so this lives in the standard build (it is **not** the
`legacy` feature). It is **on by default** (matching `vue-tsc`); set
`optionsApi: false` to opt out. The bridge only runs for non-`<script setup>`
components, so the common `<script setup>` path stays zero-cost. Legacy Vue 2.7 /
Nuxt 2 support — `typeChecker.legacyVue2`,
which additionally adds the Nuxt 2 template globals — remains a separate opt-in
that requires a `legacy` build.

`typeChecker.tsconfig` and `typeChecker.corsaPath` are part of the shared schema, but the
project-backed Corsa path is the Rust CLI surface today. `typeChecker.corsaPath` is shared by
`vize check`, type-aware `vize lint`, and `vize lsp`; `typeChecker.tsgoPath` is a deprecated alias
for older configs. Use package scripts such as `vize:check:app` or `vize:check:corsa` when you need
command-line overrides for `--tsconfig` or `--corsa-path`.

The runtime stack is `@typescript/native-preview`, the Corsa/corsa-bind API layer, and the installed
`tsgo` executable name. Keep ambient declarations, generated auto-import files, path aliases, and
Vue `ComponentCustomProperties` declarations in your project `tsconfig.json`; use a package script
that runs `vize check --tsconfig ...` to select that project file.

```json
{
  "typeChecker": {
    "corsaPath": "./node_modules/.bin/tsgo",
    "servers": 1
  },
  "lsp": {
    "typecheck": false
  }
}
```

`typeChecker.servers` is reserved for future Corsa worker pools. The direct project-session runner
currently supports only `1`; larger values fail fast instead of pretending to tune concurrency.

## Musea Options

Shared config currently covers the gallery file set and route:

```ts
export default defineConfig({
  musea: {
    include: ["src/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
    basePath: "/__musea__",
    storybookCompat: false,
    inlineArt: false,
  },
});
```

Pass presentation-focused options such as `previewCss`, `previewSetup`, `tokensPath`, `theme`, and
`storybookOutDir` directly to `musea()` in `vite.config.ts`.
