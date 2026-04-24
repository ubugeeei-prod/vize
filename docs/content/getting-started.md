---
title: Getting Started
---

# Getting Started

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. APIs and package boundaries may change without notice.

## What is Vize?

Vize (_/viːz/_) is an unofficial Vue.js toolchain written in Rust. The workspace contains shared
building blocks for:

| Area            | Main Rust crate(s)                                                                                                                                                                                                                                                                                                                                                                                                                                                     | User-facing package / command            |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| Compilation     | [`vize_atelier_core`](https://github.com/ubugeeei/vize/tree/main/crates/vize_atelier_core), [`vize_atelier_dom`](https://github.com/ubugeeei/vize/tree/main/crates/vize_atelier_dom), [`vize_atelier_vapor`](https://github.com/ubugeeei/vize/tree/main/crates/vize_atelier_vapor), [`vize_atelier_ssr`](https://github.com/ubugeeei/vize/tree/main/crates/vize_atelier_ssr), [`vize_atelier_sfc`](https://github.com/ubugeeei/vize/tree/main/crates/vize_atelier_sfc) | `@vizejs/vite-plugin`, Rust `vize build` |
| Lint            | [`vize_patina`](https://github.com/ubugeeei/vize/tree/main/crates/vize_patina)                                                                                                                                                                                                                                                                                                                                                                                         | `vize lint`, `oxlint-plugin-vize`        |
| Format          | [`vize_glyph`](https://github.com/ubugeeei/vize/tree/main/crates/vize_glyph)                                                                                                                                                                                                                                                                                                                                                                                           | Rust `vize fmt`                          |
| Type check      | [`vize_canon`](https://github.com/ubugeeei/vize/tree/main/crates/vize_canon)                                                                                                                                                                                                                                                                                                                                                                                           | Rust `vize check`                        |
| Editor support  | [`vize_maestro`](https://github.com/ubugeeei/vize/tree/main/crates/vize_maestro)                                                                                                                                                                                                                                                                                                                                                                                       | `vize lsp`, VS Code, Zed                 |
| Musea art tools | [`vize_musea`](https://github.com/ubugeeei/vize/tree/main/crates/vize_musea)                                                                                                                                                                                                                                                                                                                                                                                           | `@vizejs/vite-plugin-musea`              |
| Bindings        | [`vize_vitrine`](https://github.com/ubugeeei/vize/tree/main/crates/vize_vitrine)                                                                                                                                                                                                                                                                                                                                                                                       | `@vizejs/native`, `@vizejs/wasm`         |

This guide recommends [Vite+](https://viteplus.dev/) (`vp`) for JavaScript package management and project commands. It keeps the install and exec flow consistent across package managers while still using the workspace's underlying tool.

If you do not have `vp` yet, install it once and open a new shell:

```bash
curl -fsSL https://vite.plus | bash
```

See the [Vite+ docs](https://viteplus.dev/) and the [Installing Dependencies guide](https://viteplus.dev/guide/install) for more.

## Choose Your Entry Point

### 1. Vite Projects

Use the Vite plugin if you want native Vue compilation in an existing Vite project.

```bash
vp install -D vize @vizejs/vite-plugin
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

Use the `vize` npm package when you want shared config utilities and native lint/check commands
available in package scripts.

```bash
vp install -D vize
vp exec vize lint src
vp exec vize check
```

The npm `vize check` command uses the packaged NAPI checker. Use the Rust CLI when you need the
Corsa-backed project diagnostics path across Vue, TS, TSX, and `.d.ts` inputs.

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

## Native Type Checking

`vize check` is powered by `vize_canon`, which now leans on [`corsa-bind`](https://github.com/ubugeeei/corsa-bind) project sessions for native TypeScript diagnostics. Vize generates virtual TypeScript for Vue SFCs, asks Corsa for project-aware diagnostics, and then maps the results back onto the original `.vue`, `.ts`, `.tsx`, and `.d.ts` files.

This path is still maturing, so editor type checking remains an opt-in capability for now. If you are developing Vize alongside Corsa, `vize check --corsa-path /path/to/corsa` lets you point at a custom executable.

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
