---
title: Configuration
---

# Configuration

Vize uses `vize.config.*` for shared npm CLI, Vite plugin, and Rust CLI settings.

## Config Files

The npm CLI and `@vizejs/vite-plugin` load these files from the project root in this priority
order:

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
    vueParserQuirks: false,
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
  vueParserQuirks = false
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
    "vueParserQuirks": false
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

| Option              | Values                     | Common use                                                       |
| ------------------- | -------------------------- | ---------------------------------------------------------------- |
| `sourceMap`         | `boolean`                  | Enable source maps in the Vite plugin                            |
| `ssr`               | `boolean`                  | Compile for SSR when not relying on Vite's SSR build flag        |
| `vapor`             | `boolean`                  | Enable Vapor-mode compilation                                    |
| `customRenderer`    | `boolean`                  | Treat lowercase non-HTML tags as custom renderer elements        |
| `vueParserQuirks`   | `boolean`                  | Match Vue parser quirks for known edge cases                     |
| `scriptExt`         | `"ts"` or `"js"`           | Preserve TS output or downcompile to JS in the npm build command |
| `mode`              | `"module"` or `"function"` | Lower-level compiler output mode                                 |
| `prefixIdentifiers` | `boolean`                  | Prefix template identifiers with `_ctx`                          |
| `hoistStatic`       | `boolean`                  | Control static node hoisting                                     |
| `cacheHandlers`     | `boolean`                  | Control event handler caching                                    |
| `isTs`              | `boolean`                  | Parse script blocks as TypeScript                                |
| `runtimeModuleName` | `string`                   | Override runtime import module                                   |
| `runtimeGlobalName` | `string`                   | Override runtime global for function/IIFE-style output           |

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
      vueParserQuirks: false,
    }),
  ],
});
```

## Vue Parser Quirks

`compiler.vueParserQuirks` defaults to `false`. Keep strict mode unless you need to compile
existing templates that Vue accepts through parser edge-case behavior.

The compatibility cases are:

- `v-for` aliases with an unmatched edge parenthesis. Vue strips a leading `(` or trailing `)`
  from the alias before it splits `value`, `key`, and `index`; strict Vize reports those aliases as
  malformed.
- Non-void HTML elements written with self-closing syntax, such as `<div />` or `<span />`. Strict
  Vize follows HTML tree construction and ignores the self-closing flag, while quirk mode keeps the
  element as a self-closing leaf to match Vue parser compatibility.

```text
<template>
  <!-- Strict mode rejects this. Quirk mode compiles it as `item in items`. -->
  <div v-for="(item in items">{{ item }}</div>

  <!-- Strict mode rejects this. Quirk mode compiles it as `item in items`. -->
  <div v-for="item) in items">{{ item }}</div>

  <!-- Strict mode treats this as an open `<div>` start tag. Quirk mode keeps it as a leaf. -->
  <div />
</template>
```

Vue upstream implementation:

- [`forAliasRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/utils.ts#L571)
- [`stripParensRE` in `parseForExpression`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts#L493-L530)

See [Troubleshooting](./troubleshooting.md) for the HTML strict-mode behavior behind invalid
self-closing tags.

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
  },
});
```

`typeChecker.tsconfig` and `typeChecker.corsaPath` are part of the shared schema, but the
project-backed Corsa path is the Rust CLI surface today. `typeChecker.corsaPath` is shared by
`vize check`, type-aware `vize lint`, and `vize lsp`; `typeChecker.tsgoPath` is a deprecated alias
for older configs. Use `vize check --tsconfig ...` and `vize check --corsa-path ...` when you need
command-line overrides.

The runtime stack is `@typescript/native-preview`, the Corsa/corsa-bind API layer, and the installed
`tsgo` executable name. Keep ambient declarations, generated auto-import files, path aliases, and
Vue `ComponentCustomProperties` declarations in your project `tsconfig.json`; use
`vize check --tsconfig ...` to select that project file.

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
