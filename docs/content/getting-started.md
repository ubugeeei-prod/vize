---
title: Getting Started
---

# Getting Started

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. APIs and features may change without notice.

## What is Vize?

Vize (_/viːz/_) is an unofficial, high-performance Vue.js toolchain written entirely in Rust. It provides a unified set of tools for Vue.js development:

| Tool         | Purpose                     | Replaces            |
| ------------ | --------------------------- | ------------------- |
| `vize build` | SFC compilation             | `@vue/compiler-sfc` |
| `vize lint`  | Template and script linting | `eslint-plugin-vue` |
| `vize fmt`   | Code formatting             | `prettier`          |
| `vize check` | Type checking               | `vue-tsc`           |
| `vize musea` | Component gallery           | Storybook           |
| `vize lsp`   | Editor integration          | Volar               |

All of these share a single parser, a single AST representation, and a single configuration surface — eliminating the overhead and inconsistencies of maintaining separate tools.

## Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 9+ (recommended) or npm

For building from source:

- [Rust](https://rustup.rs/) 1.80+

## Installation

### CLI

Install the Vize CLI globally:

```bash
# via npm (recommended)
npm install -g vize

# via Cargo
cargo install vize
```

### npm Packages

Vize is distributed as multiple npm packages, each serving a specific integration point:

```bash
# Main package (includes CLI)
npm install vize

# Native bindings (Node.js) — used by the Vite plugin
npm install @vizejs/native

# WASM bindings (Browser) — for playgrounds and in-browser compilation
npm install @vizejs/wasm

# Vite plugin — drop-in replacement for @vitejs/plugin-vue
npm install @vizejs/vite-plugin vize

# Experimental unplugin integration — rollup / webpack / esbuild
npm install @vizejs/unplugin

# Experimental Rspack integration — dedicated path
npm install @vizejs/rspack-plugin @rspack/core

# Nuxt module — first-class Nuxt integration
npm install @vizejs/nuxt

# Musea (component gallery)
npm install @vizejs/vite-plugin-musea

# MCP server (AI assistant integration)
npm install @vizejs/musea-mcp-server
```

> **Bundler status:** `@vizejs/vite-plugin` is the recommended integration today.
> `@vizejs/unplugin` and `@vizejs/rspack-plugin` are available for non-Vite build systems, but they are still unstable.
> Rspack intentionally uses the dedicated `@vizejs/rspack-plugin` path because its loader and CSS integration are Rspack-specific.

## Quick Start

### Using the CLI

Once installed, you can compile Vue SFC files immediately:

```bash
# Compile all .vue files in current directory to ./dist
vize

# Custom input/output
vize build src/**/*.vue -o out

# SSR mode
vize build --ssr

# Format check
vize fmt --check

# Lint with auto-fix
vize lint --fix

# Type check
vize check --strict
```

### Using the Vite Plugin

Add Vize to your Vite project for native-speed Vue compilation. This is a drop-in replacement for `@vitejs/plugin-vue` — no code changes to your components are required:

```javascript
// vite.config.js
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

The plugin handles SFC compilation, `<script setup>`, scoped CSS, HMR, and SSR — all through Rust-native NAPI bindings. See [Vite Plugin](./guide/vite-plugin.md) for configuration options.

### Shared `vize.config.*`

The npm CLI and `@vizejs/vite-plugin` share the same config discovery. Vize looks for these files in the project root:

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
  },
  linter: {
    preset: "opinionated",
  },
});
```

PKL config:

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
}

linter {
  preset = "opinionated"
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

Today the npm CLI applies shared config to `vize lint`, and the Vite plugin reads the same file for compiler and plugin options.

### Using Other Bundlers (Experimental)

For rollup, webpack, or esbuild, use `@vizejs/unplugin`.
For Rspack, use `@vizejs/rspack-plugin`.

These integrations are still unstable and should be treated as experimental.
Vite remains the recommended path if you need the most complete and best-tested experience today.

See [Experimental Bundler Integrations](./guide/unplugin.md) for setup details and caveats.

### Using with Nuxt

Vize provides a dedicated Nuxt module with first-class support:

```typescript
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
    musea: {
      include: ["**/*.art.vue"],
    },
  },
});
```

See [Nuxt Integration](./integrations/nuxt.md) for more details.

### Using WASM in the Browser

```javascript
import init, { compileSfc } from "@vizejs/wasm";

await init();
const { code } = compileSfc(`<template><div>{{ msg }}</div></template>`, { filename: "App.vue" });
```

See [WASM Bindings](./guide/wasm.md) for the full API.

## Development Setup

For contributing to Vize itself:

### With Nix

```bash
nix develop
vp env install
vp install
vp run --workspace-root cli             # Enable vize CLI command
vp run --workspace-root dev:playground  # Start playground
```

### Without Nix

```bash
git clone https://github.com/ubugeeei/vize.git
cd vize
vp env install
vp install

# Build CLI
cargo build --release -p vize

# Run playground
vp run --workspace-root dev:playground
```

### Project Structure

```
vize/
├── crates/               # Rust crates (compiler, linter, formatter, etc.)
│   ├── vize/             # CLI binary
│   ├── vize_armature/    # Parser
│   ├── vize_relief/      # AST definitions
│   ├── vize_croquis/     # Semantic analysis
│   ├── vize_atelier_*/   # Compilation backends (dom, vapor, sfc, ssr)
│   ├── vize_patina/      # Linter
│   ├── vize_glyph/       # Formatter
│   ├── vize_canon/       # Type checker
│   ├── vize_maestro/     # LSP
│   ├── vize_musea/       # Component gallery
│   ├── vize_vitrine/     # NAPI + WASM bindings
│   └── ...
├── npm/                  # npm packages (vite-plugin, wasm, native, etc.)
├── playground/           # Development playground
├── docs/                 # Documentation (this site)
└── tests/                # Integration tests
```

## Next Steps

- [Philosophy](./philosophy.md) — Design principles and vision
- [CLI Reference](./guide/cli.md) — Full command documentation
- [Vite Plugin](./guide/vite-plugin.md) — Configuration options
- [Experimental Bundler Integrations](./guide/unplugin.md) — rollup / webpack / esbuild / Rspack status
- [Musea](./guide/musea.md) — Component gallery guide
- [Architecture](./architecture/overview.md) — How Vize works internally
