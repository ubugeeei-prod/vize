---
title: Configuration
---

# Configuration

Vize uses `vize.config.*` for shared npm CLI and Vite plugin settings. Some Rust CLI commands also
read config directly, but their supported file formats and field names are currently narrower.

## Config Files

The npm CLI and `@vizejs/vite-plugin` load these files from the project root:

- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.pkl`
- `vize.config.json`

The Rust CLI currently reads `vize.config.pkl` first and then `vize.config.json` for command-native
settings such as `check`, `lsp`, and `fmt`.

## TypeScript Config

```ts
import { defineConfig } from "vize";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  compiler: {
    sourceMap: mode !== "production",
    ssr: isSsrBuild,
    vapor: false,
    customRenderer: false,
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

## PKL Config

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
  vapor = false
  customRenderer = false
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
    "customRenderer": false
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
    }),
  ],
});
```

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
project-backed Corsa path is the Rust CLI surface today. Use `vize check --tsconfig ...` and
`vize check --corsa-path ...` when you need those controls.

The Rust `vize check` command reads its own `check` block from `vize.config.pkl` or
`vize.config.json` for command-native settings such as worker count. Keep ambient declarations,
generated auto-import files, path aliases, and Vue `ComponentCustomProperties` declarations in your
project `tsconfig.json`; use `vize check --tsconfig ...` to select that project file.

```json
{
  "check": {
    "servers": 4
  },
  "lsp": {
    "typecheck": false
  }
}
```

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
