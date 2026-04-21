---
title: Getting Started
---

# Getting Started

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. APIs and package boundaries may change without notice.

## What is Vize?

Vize (_/viːz/_) is an unofficial Vue.js toolchain written in Rust. The workspace contains shared
building blocks for:

| Area             | Main Rust crate(s)                   | User-facing package / command            |
| ---------------- | ------------------------------------ | ---------------------------------------- |
| Template compile | `vize_atelier_*`, `vize_atelier_sfc` | `@vizejs/vite-plugin`, Rust `vize build` |
| Lint             | `vize_patina`                        | `vize lint`, `oxlint-plugin-vize`        |
| Format           | `vize_glyph`                         | Rust `vize fmt`                          |
| Type check       | `vize_canon`                         | Rust `vize check`                        |
| Editor support   | `vize_maestro`                       | `vize lsp`, VS Code, Zed                 |
| Musea art tools  | `vize_musea`                         | `@vizejs/vite-plugin-musea`              |
| Bindings         | `vize_vitrine`                       | `@vizejs/native`, `@vizejs/wasm`         |

## Choose Your Entry Point

### 1. Vite Projects

Use the Vite plugin if you want native Vue compilation in an existing Vite project.

```bash
pnpm add -D vize @vizejs/vite-plugin
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

### 2. npm CLI + Shared Config

Use the `vize` npm package when you want shared config utilities and the native lint command.

```bash
pnpm add -D vize
pnpm exec vize lint src
```

The npm package currently focuses on config loading plus `lint`.

### 3. Full Rust CLI

Use the Rust binary when you want the full native CLI today.

```bash
cargo install vize
```

```bash
vize build src/**/*.vue
vize fmt --check src
vize lint --profile src
vize check --profile src
vize lsp
```

## Shared `vize.config.*`

The npm CLI and `@vizejs/vite-plugin` share config discovery:

- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.pkl`
- `vize.config.json`

TypeScript config:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  linter: {
    preset: "opinionated",
  },
  formatter: {
    printWidth: 100,
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
pnpm add -D @vizejs/vite-plugin
pnpm add @vizejs/native
pnpm add @vizejs/wasm
pnpm add @vizejs/unplugin
pnpm add @vizejs/rspack-plugin @rspack/core
pnpm add @vizejs/nuxt
pnpm add @vizejs/vite-plugin-musea
pnpm add @vizejs/musea-mcp-server
pnpm add -D oxlint oxlint-plugin-vize
```

Notes:

- `@vizejs/vite-plugin` is the recommended bundler integration today.
- `@vizejs/unplugin` and `@vizejs/rspack-plugin` are still experimental.
- `@vizejs/native` and `@vizejs/wasm` expose the Rust bindings directly.
- `@vizejs/vite-plugin-musea` provides the gallery and dev-server workflow for Musea.

## Oxlint Integration

Run Vize's Vue diagnostics inside Oxlint:

```bash
pnpm add -D oxlint oxlint-plugin-vize
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
pnpm exec oxlint-vize -c .oxlintrc.json -f stylish src
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

This repository uses `Nix + pnpm + vp` for local development.

```bash
nix develop
pnpm install --frozen-lockfile
vp build
vp run --workspace-root check
```
