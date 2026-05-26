---
title: CLI
---

# CLI Reference

> **⚠️ Work in Progress:** Vize is under active development and the CLI surface is still evolving.

This page describes the Rust-native `vize` binary.
The npm `vize` package exposes shared config helpers plus NAPI-backed `build`, `fmt`, `lint`,
`check`, `clean`, `ready`, and `upgrade` commands. Install the Rust binary when you need LSP, IDE,
`check-server`, or Corsa project diagnostics.

For a higher-level explanation of the analysis pipeline, see [Static Analysis](./static-analysis.md).

## Installation

For v1 alpha, use the prebuilt GitHub release binaries or the Nix entry point. The Rust CLI is not a
supported crates.io install channel yet.

```bash
nix run github:ubugeeei/vize#vize -- --help
```

You can also download platform-specific binaries from
[GitHub Releases](https://github.com/ubugeeei/vize/releases).

For local development inside this repository, install the workspace build:

```bash
cargo install --path crates/vize --force --locked
```

## Rust CLI vs npm CLI

| Need                                                                   | Recommended entry point                 |
| ---------------------------------------------------------------------- | --------------------------------------- |
| Package scripts for build, format, lint, check, ready, and upgrade     | `vp exec vize ...` from the npm package |
| Project-backed type checking across `.vue`, `.ts`, `.tsx`, and `.d.ts` | Rust `vize check`                       |
| LSP, IDE setup, `check-server`, and profiling artifacts                | Rust `vize` binary                      |
| Shared Vite plugin, npm CLI, and Rust CLI settings                     | `vize.config.*`                         |

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
| `inspector`    | Create playground compiler inspector payloads   |
| `clean`        | Remove Vize-generated cache artifacts           |
| `ready`        | Run `fmt`, `lint`, `check`, and `build`         |
| `upgrade`      | Update the installed CLI                        |
| `check-server` | Start the Unix JSON-RPC typecheck server        |
| `musea`        | Musea subcommands and scaffolding               |
| `lsp`          | Start the language server                       |
| `ide`          | Install or manage editor integrations           |

All `--profile` terminal reports are rendered by the local-only `vize_curator` crate. The
instrumentation hooks remain in `vize_carton`, while curator owns the CLI report shape alongside
inspector and agent-facing artifacts.

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
| `--servers`         | Reserved Corsa server count; only `1` is supported |
| `--declaration`     | Emit `.d.ts` output                                |
| `--declaration-dir` | Output directory for emitted declarations          |

Use `--corsa-path` when you want to pin a custom Corsa executable while developing Vize or testing a
local `corsa-bind` checkout. The shared config key is `typeChecker.corsaPath`; `typeChecker.tsgoPath`
is kept only as a compatibility alias.

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

## Inspector

```bash
vize inspector src/App.vue
vize inspector "src/**/*.vue" --target ssr
vize inspector src --format json --output inspector-payload.json
vize inspector src --format agent --output inspector-agent.json
```

`vize inspector` packages one or more `.vue` files into the payload consumed by the playground
compiler inspector. The browser then inspects Vue output, Vize output, Virtual TS, VIR, and the
cross-file graph, then produces a permalink plus a prefilled pull request link.

Use `--format agent` when another local tool or AI agent needs the same repro without opening the
browser. The report contains the exact payload, playground URL, summary metrics, and import graph.
Payload, graph, and line diff metadata are built by the local-only `vize_curator` crate so CLI and
playground inspection stay aligned.

Key options:

| Option                | Description                              |
| --------------------- | ---------------------------------------- |
| `-f, --format`        | Output format: `url`, `json`, or `agent` |
| `--target`            | Compiler target: `dom` or `ssr`          |
| `--playground-url`    | Playground base URL for generated links  |
| `--max-files`         | Limit files included in a batch payload  |
| `--custom-renderer`   | Enable custom renderer comparison        |
| `--vue-parser-quirks` | Enable Vue parser compatibility quirks   |
| `-o, --output`        | Write the URL or JSON payload to a file  |

See [Compiler Inspector](./compiler-inspector.md) for the contributor workflow.

## Clean

```bash
vize clean
vize clean --dry-run
vize clean path/to/project
```

`vize clean` removes `node_modules/.vize` for the selected project root. Use it when profile
artifacts or materialized Corsa project files should be rebuilt from a blank cache directory.

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

By default, `vize upgrade` updates the npm package through Vite+:

```bash
vp install -D vize@latest
```

Use `--source cargo` only for explicit local Cargo installs.

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
