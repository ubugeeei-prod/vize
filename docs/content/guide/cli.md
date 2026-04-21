---
title: CLI
---

# CLI Reference

> **⚠️ Work in Progress:** Vize is under active development and the CLI surface is still evolving.

This page describes the Rust-native `vize` binary.
The npm `vize` package currently exposes shared config helpers plus `vize lint`; for the full CLI,
install the Rust binary with `cargo install vize`.

## Installation

```bash
cargo install vize
```

Or run the current workspace build:

```bash
nix run github:ubugeeei/vize#vize -- --help
```

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

| Option           | Description                                         |
| ---------------- | --------------------------------------------------- |
| `--fix`          | Reserved for future autofix support                 |
| `-f, --format`   | Output format: `text` or `json`                     |
| `--max-warnings` | Fail when warnings exceed the limit                 |
| `-q, --quiet`    | Show summary only                                   |
| `--help-level`   | `full`, `short`, or `none`                          |
| `--preset`       | `happy-path`, `opinionated`, `essential`, or `nuxt` |
| `--profile`      | Print timing profile                                |

## Check

```bash
vize check
vize check src
vize check --tsconfig tsconfig.app.json
vize check --profile src
```

`vize check` is backed by `vize_canon` and Corsa project sessions exposed through [`corsa-bind`](https://github.com/ubugeeei/corsa-bind). Vize generates virtual TypeScript for Vue SFCs, runs project diagnostics on a native path, and maps the results back to the original source locations.

When no explicit paths are given, `vize check` uses `tsconfig.json` `files` / `include` /
`exclude` if available.

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

## Musea

```bash
vize musea --help
vize musea new
```

The `musea` subcommand currently focuses on scaffolding and experimental entry points.
For day-to-day gallery development, the recommended workflow today is
`@vizejs/vite-plugin-musea`.

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
