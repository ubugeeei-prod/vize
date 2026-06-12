---
title: Getting Started
---

# Getting Started

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. APIs and package boundaries may change without notice.

## What is Vize?

Vize (_/viːz/_) is a Vue.js toolchain written in Rust. The workspace contains shared
building blocks for:

| Area            | Main Rust crate(s)                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | User-facing entry point                        |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| Compilation     | [`vize_atelier_core`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_core), [`vize_atelier_dom`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_dom), [`vize_atelier_vapor`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_vapor), [`vize_atelier_ssr`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_ssr), [`vize_atelier_sfc`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_sfc) | `@vizejs/vite-plugin`, npm `vize:build` script |
| Lint            | [`vize_patina`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_patina)                                                                                                                                                                                                                                                                                                                                                                                                             | npm `vize:lint` script, `oxlint-plugin-vize`   |
| Format          | [`vize_glyph`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_glyph)                                                                                                                                                                                                                                                                                                                                                                                                               | npm `vize:fmt` script                          |
| Type check      | [`vize_canon`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_canon)                                                                                                                                                                                                                                                                                                                                                                                                               | npm `vize:check` script                        |
| Editor support  | [`vize_maestro`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_maestro)                                                                                                                                                                                                                                                                                                                                                                                                           | VS Code, Zed, Rust `vize lsp`                  |
| Musea art tools | [`vize_musea`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_musea)                                                                                                                                                                                                                                                                                                                                                                                                               | `@vizejs/vite-plugin-musea`                    |
| Bindings        | [`vize_vitrine`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_vitrine)                                                                                                                                                                                                                                                                                                                                                                                                           | `@vizejs/native`, `@vizejs/wasm`               |

This guide recommends [Vite+](https://viteplus.dev/) (`vp`) for JavaScript package management and project commands. It keeps the install and exec flow consistent across package managers while still using the workspace's underlying tool.

If you do not have `vp` yet, install it once and open a new shell:

```bash
curl -fsSL https://vite.plus | bash
```

See the [Vite+ docs](https://viteplus.dev/) and the [Installing Dependencies guide](https://viteplus.dev/guide/install) for more.

## What Vize Does

At a high level, Vize is split into a few reusable pipelines:

| Pipeline          | Package or script                        | What you get                                                                               |
| ----------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------ |
| Compile           | `@vizejs/vite-plugin`, `vize:build`      | Rust-native Vue SFC compilation, SSR output, Vapor mode, scoped CSS handling               |
| Static analysis   | `vize:lint`, `oxlint-plugin-vize`        | Vue template, script, CSS, a11y, SSR, Vapor, Musea, cross-file, and type-aware diagnostics |
| Type check        | `vize:check`                             | Virtual TypeScript generation, project diagnostics, Vue-to-source diagnostic mapping       |
| Format            | `vize:fmt`                               | Vue SFC formatting with project and CLI options                                            |
| Component gallery | `@vizejs/vite-plugin-musea`, `musea-vrt` | Art files, component variants, preview setup, design tokens, a11y, VRT                     |
| Editor support    | VS Code, Zed, Rust `vize lsp`            | Opt-in diagnostics and editor features                                                     |

See [Static Analysis](./guide/static-analysis.md) for the lint and type-checking model,
[Rules](./rules/index.md) for concrete rule output, and
[Configuration](./guide/configuration.md) for shared config and compiler options.

Authoring components in JSX/TSX instead of `.vue` SFCs? See the [JSX & TSX](./guide/jsx.md) guide —
`.jsx`/`.tsx` Vue components compile through the same Rust pipeline.

## Choose Your Entry Point

### 1. Vite Projects

Use the Vite plugin if you want native Vue compilation in an existing Vite project.

```bash
vp install -D @vizejs/vite-plugin
```

Install `vize` as a direct dependency only when you want to import shared config helpers from
`"vize"` or add Vize package scripts such as `vize:lint` and `vize:check`.

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

Add compiler options in `vize.config.ts` when you want the same settings available to package
scripts and the plugin:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

### 2. Nuxt Projects

Use the Nuxt module when you want Vize to run inside Nuxt's own Vite pipeline.

```bash
vp install @vizejs/nuxt
```

Add the module to `nuxt.config.ts`:

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

Run your Nuxt dev server as usual. The module registers `@vizejs/vite-plugin` for Vue SFC
compilation while preserving Nuxt auto-imports, components, middleware, and SSR transforms.

See the [Nuxt Integration](./integrations/nuxt.md) guide for Musea setup and Nuxt-specific notes.

### 3. npm Package Scripts + Shared Config

Use the `vize` npm package when you want shared config utilities and native commands available from
project scripts.

```bash
vp install -D vize
```

Recommended package scripts:

```json
{
  "scripts": {
    "vize:build": "vize build src",
    "vize:fmt": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path src",
    "vize:check": "vize check src",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:fmt
vp run vize:lint
vp run vize:check
vp run vize:build
vp run vize:ready
```

The npm package's `vize check` command uses the packaged NAPI checker and can emit Vue component
declarations with `--declaration --declaration-dir dist/types`. Use the Rust CLI when you need
`check-server`, LSP, IDE management, or project diagnostics across Vue, TS, TSX, and `.d.ts` inputs.

### 4. Full Rust CLI

Most application workflows should use the npm package scripts above. Use the Rust binary when you
need the full native CLI today: LSP, IDE management, profiling, or `check-server`. For v1 alpha, the
supported public channels are GitHub release binaries and the Nix entry point; the Rust CLI is not
published through crates.io yet.

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

```bash
vize build src/**/*.vue
vize fmt --check src
vize lint --profile src
vize check --profile src
vize ready src
vize lsp
```

## Native Type Checking

`vize check` is powered by `vize_canon`, which now leans on [`corsa-bind`](https://github.com/ubugeeei/corsa-bind) project sessions for native TypeScript diagnostics. Vize generates virtual TypeScript for Vue SFCs, asks Corsa for project-aware diagnostics, and then maps the results back onto the original `.vue`, `.ts`, `.tsx`, and `.d.ts` files.

This path is still maturing, so editor type checking remains an opt-in capability for now. The
runtime stack is the `@typescript/native-preview` package, Corsa/corsa-bind is the API layer Vize
talks to, and the executable installed by the TypeScript native preview is still commonly named
`tsgo`. Use `typeChecker.corsaPath`, or a package script that runs
`vize check --corsa-path /path/to/tsgo`, when you want to pin that runtime.
`typeChecker.tsgoPath` remains a deprecated compatibility alias.

Useful package-script targets:

```json
{
  "scripts": {
    "vize:check": "vize check",
    "vize:check:app": "vize check --tsconfig tsconfig.app.json",
    "vize:check:virtual-ts": "vize check --show-virtual-ts src/components/App.vue",
    "vize:check:declarations": "vize check --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check
vp run vize:check:app
vp run vize:check:virtual-ts
vp run vize:check:declarations
```

## Shared `vize.config.*`

The npm package commands and `@vizejs/vite-plugin` share config discovery:

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

TypeScript config:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  linter: {
    preset: "opinionated",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    corsaPath: "./node_modules/.bin/tsgo",
  },
  formatter: {
    printWidth: 100,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
});
```

PKL config:

```pkl
amends "node_modules/vize/pkl/vize.pkl"

linter {
  preset = "opinionated"
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

JSON config with schema:

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "linter": {
    "preset": "opinionated"
  }
}
```

## Packages

```bash
vp install -D @vizejs/vite-plugin
vp install @vizejs/native
vp install @vizejs/wasm
vp install @vizejs/unplugin
vp install @vizejs/rspack-plugin @rspack/core
vp install @vizejs/nuxt
vp install @vizejs/vite-plugin-musea
vp install @vizejs/musea-mcp-server
vp install -D oxlint oxlint-plugin-vize
```

Notes:

- `@vizejs/vite-plugin` is the recommended bundler integration today.
- `@vizejs/unplugin` and `@vizejs/rspack-plugin` are still experimental.
- `@vizejs/native` and `@vizejs/wasm` expose the Rust bindings directly.
- `@vizejs/vite-plugin-musea` provides the gallery and dev-server workflow for Musea.

## Musea Component Gallery

Use Musea when you want Vue-native component examples, documentation, tokens, VRT, and a11y checks:

```bash
vp install -D @vizejs/vite-plugin @vizejs/vite-plugin-musea vize
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [
    vize(),
    musea({
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
      previewCss: ["src/styles/main.css"],
    }),
  ],
});
```

Run your Vite dev server and open `/__musea__`. See [Musea](./guide/musea.md) for art files,
preview setup, design tokens, VRT, and generated variants.

## Oxlint Integration

Run Vize's Vue diagnostics inside Oxlint:

```bash
vp install -D oxlint oxlint-plugin-vize
```

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  },
  "settings": {
    "vize": {
      "preset": "general-recommended",
      "helpLevel": "short"
    }
  }
}
```

For terminal-first usage, prefer:

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

## Editor Support

For day-to-day Vue editing, keep using `vuejs/language-tools` for now.
Vize editor features are designed for incremental opt-in.

VS Code starting point:

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

Zed starting point:

```json
{
  "languages": {
    "Vue": {
      "language_servers": ["vize", "..."]
    }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true
      }
    }
  }
}
```

## Local Development

This repository uses `Nix + Vite+ (vp)` for local development. In this workspace, `vp` will use `pnpm` automatically.

```bash
nix develop
vp install --frozen-lockfile
vp check
vp fmt
vp dev
vp build
```
