---
title: CLI
---

# CLI Reference

> **⚠️ Work in Progress:** Vize is under active development and the CLI surface is still evolving.

This page describes the Rust-native `vize` binary.
The npm `vize` package exposes shared config helpers plus NAPI-backed `build`, `fmt`, `lint`,
`check`, `ready`, and `upgrade` commands. Install the Rust binary when you need LSP, IDE,
`check-server`, or Corsa project diagnostics.

For a higher-level explanation of the analysis pipeline, see [Static Analysis](./static-analysis.md).

## Installation

```bash
cargo install vize
```

Or run the current workspace build:

```bash
nix run github:ubugeeei/vize#vize -- --help
```

## Rust CLI vs npm CLI

| Need                                                                   | Recommended entry point                  |
| ---------------------------------------------------------------------- | ---------------------------------------- |
| Package scripts for build, format, lint, check, ready, and upgrade     | `vp exec vize ...` from the npm package  |
| Project-backed type checking across `.vue`, `.ts`, `.tsx`, and `.d.ts` | Rust `vize check`                        |
| LSP, IDE setup, `check-server`, and profiling artifacts                | Rust `vize` binary                       |
| Shared Vite plugin and npm CLI settings                                | `vize.config.ts` through the npm package |
| Rust command-native settings                                           | `vize.config.pkl` or `vize.config.json`  |

## Commands

```bash
vize [COMMAND]
```

When invoked without a command, `vize` defaults to `build`.

| Command        | Description                                     |
| -------------- | ----------------------------------------------- |
| `build`        | Compile Vue SFC files                           |
| `fmt`          | Format Vue SFC files                            |
| `lint`         | Lint Vue SFC files                              |
| `check`        | Type check Vue SFC, TS, TSX, and `.d.ts` inputs |
| `ready`        | Run `fmt`, `lint`, `check`, and `build`         |
| `upgrade`      | Update the installed CLI                        |
| `check-server` | Start the Unix JSON-RPC typecheck server        |
| `musea`        | Musea subcommands and scaffolding               |
| `lsp`          | Start the language server                       |
| `ide`          | Install or manage editor integrations           |

## Build

```bash
vize build src/**/*.vue
vize build --ssr
vize build --profile src
```

Key options:

| Option                | Description                                   |
| --------------------- | --------------------------------------------- |
| `-o, --output`        | Output directory                              |
| `-f, --format`        | Output format: `js`, `json`, `stats`          |
| `--ssr`               | Enable SSR compilation                        |
| `--script-ext`        | `preserve` or `downcompile`                   |
| `-j, --threads`       | Thread count override                         |
| `--profile`           | Print timing profile                          |
| `--continue-on-error` | Keep compiling and report failures at the end |

## Format

```bash
vize fmt --check src
vize fmt --write src
```

Key options:

| Option                             | Description                                          |
| ---------------------------------- | ---------------------------------------------------- |
| `--check`                          | Report files that would change                       |
| `-w, --write`                      | Write formatted output                               |
| `--single-quote`                   | Toggle string quote style                            |
| `--print-width`                    | Maximum line width                                   |
| `--tab-width`                      | Indentation width                                    |
| `--use-tabs`                       | Toggle tabs vs spaces                                |
| `--no-semi`                        | Omit semicolons                                      |
| `--sort-attributes`                | Sort template attributes                             |
| `--single-attribute-per-line`      | Put one attribute per line                           |
| `--max-attributes-per-line`        | Wrap after a given attribute count                   |
| `--normalize-directive-shorthands` | Normalize `v-bind:` / `v-on:` / `v-slot:` shorthands |
| `--profile`                        | Print timing profile                                 |

## Lint

```bash
vize lint src
vize lint --preset opinionated src
vize lint --help-level short src
```

Key options:

| Option                | Description                                                                               |
| --------------------- | ----------------------------------------------------------------------------------------- |
| `--fix`               | Reserved for future autofix support                                                       |
| `-f, --format`        | Output format: `text`, `ansi`, `plain`, `json`, `stylish`, `markdown`, `html`, or `agent` |
| `--max-warnings`      | Fail when warnings exceed the limit                                                       |
| `-q, --quiet`         | Show summary only                                                                         |
| `--help-level`        | `full`, `short`, or `none`                                                                |
| `--preset`            | `happy-path`, `opinionated`, `essential`, `incremental`, or `nuxt`                        |
| `--cross-file`        | Enable opt-in cross-file checks                                                           |
| `--cross-file-tree`   | Print the provide/inject tree when cross-file linting is enabled                          |
| `--strict-reactivity` | Enable native checker-backed reactivity-loss linting                                      |
| `--profile`           | Print timing profile                                                                      |
| `--slow-threshold`    | Slow file threshold for profile output                                                    |

Presets are intended for staged adoption:

| Preset        | Use it when                                                            |
| ------------- | ---------------------------------------------------------------------- |
| `essential`   | You want correctness-oriented diagnostics in CI                        |
| `happy-path`  | You want the default recommended bundle                                |
| `opinionated` | You want stronger conventions, script rules, and type-aware candidates |
| `incremental` | You only want explicitly configured rules                              |
| `nuxt`        | You want opinionated rules with Nuxt component assumptions             |

Examples:

```bash
vize lint --preset essential --max-warnings 0 src
vize lint --preset opinionated --help-level short src
vize lint --cross-file --cross-file-tree src
vize lint --strict-reactivity src
vize lint --format ansi src
vize lint --format plain src
vize lint --format agent src
vize lint --format markdown src
```

## Check

```bash
vize check
vize check src
vize check --tsconfig tsconfig.app.json
vize check --profile src
```

`vize check` is backed by `vize_canon` and Corsa project sessions exposed through [`corsa-bind`](https://github.com/ubugeeei/corsa-bind). Vize generates virtual TypeScript for Vue SFCs, runs project diagnostics on a native path, and maps the results back to the original source locations.

When no explicit paths are given, `vize check` uses `tsconfig.json` `files` / `include` /
`exclude` if available. Explicit inputs may be files, directories, or globs and can include `.vue`,
`.ts`, `.tsx`, and `.d.ts`.

Key options:

| Option              | Description                                        |
| ------------------- | -------------------------------------------------- |
| `-s, --socket`      | Connect to a running `check-server`                |
| `--tsconfig`        | Override `tsconfig.json`                           |
| `-f, --format`      | Output format: `text` or `json`                    |
| `--show-virtual-ts` | Print generated virtual TypeScript                 |
| `-q, --quiet`       | Show summary only                                  |
| `--profile`         | Write profile artifacts under `node_modules/.vize` |
| `--corsa-path`      | Override the Corsa executable path                 |
| `--servers`         | Parallel Corsa worker count                        |
| `--declaration`     | Emit `.d.ts` output                                |
| `--declaration-dir` | Output directory for emitted declarations          |

Use `--corsa-path` when you want to pin a custom Corsa executable while developing Vize or testing a local `corsa-bind` checkout.

Useful patterns:

```bash
vize check --tsconfig tsconfig.app.json src
vize check --show-virtual-ts src/components/App.vue
vize check --profile src
vize check --declaration --declaration-dir dist/types
```

Project-wide template values and Vue ambient types should be visible through TypeScript project
configuration. Include generated files such as `auto-imports.d.ts`, `components.d.ts`, or your own
Vue declarations in `tsconfig.json`, then select that project with `--tsconfig` when needed:

```json
{
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue", "src/**/*.d.ts"]
}
```

```ts
// src/types/vue-app.d.ts
declare module "vue" {
  interface ComponentCustomProperties {
    $t: (key: string) => string;
  }
}
```

```bash
vize check --tsconfig tsconfig.app.json src
```

## Ready

```bash
vize ready src
vize ready --output dist src
```

`vize ready` runs `fmt --write`, `lint`, `check`, and `build` in order. The command stops at the
first failing step.

Key options:

| Option         | Description                         |
| -------------- | ----------------------------------- |
| `-o, --output` | Output directory for the build step |
| `--ssr`        | Enable SSR compilation for build    |
| `--script-ext` | `preserve` or `downcompile`         |

## Upgrade

```bash
vize upgrade
vize upgrade --dry-run
```

The Rust CLI upgrades through Cargo by running `cargo install vize --force --locked`.
The npm CLI upgrades the npm package through the detected package manager.

## Musea

```bash
vize musea --help
vize musea serve --port 6006
vize musea new
```

The `musea` subcommand currently focuses on scaffolding and experimental entry points.
For day-to-day gallery development, the recommended workflow today is
`@vizejs/vite-plugin-musea`.

The npm CLI also exposes a convenience `vize musea` command that runs Vite with the Musea plugin
installed in your project:

```bash
vp exec vize musea
vp exec vize musea --build
```

## LSP and IDE

```bash
vize lsp
vize lsp --port 9527
vize ide vscode
vize ide zed
```

`vize lsp` starts the language server directly.
`vize ide` adds editor-specific install and management commands for the VS Code and Zed
integrations.

## Global Options

```bash
vize --help
vize --version
vize <command> --help
```
