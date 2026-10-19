---
title: CLI
---

# CLI Reference

Use the CLI to compile, format, lint, and type-check Vue files from a terminal or CI job.
For apps, install the `vize` npm package and start with the [package scripts below](#application-package-scripts). For LSP, IDE management, `check-server`, and profiling, use the [Rust binary installation](#rust-binary-installation).

The npm package also exports shared configuration helpers. See the [command comparison](#npm-package-scripts-vs-rust-cli) to choose an entry point, or follow [User Workflows](./workflows.md) for a task-oriented walkthrough.

## Application Package Scripts

For apps, install from npm and wire stable commands into project scripts:

```bash
vp install -D vize
```

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
vp run vize:lint
vp run vize:check
vp run vize:ready
```

Use `vp exec vize ...` for one-off debugging; prefer named scripts for documented workflows and CI.

## Rust Binary Installation

For v1 alpha, use the prebuilt GitHub release binaries or the Nix entry point. The Rust CLI is not a
supported crates.io install channel yet.

```bash
nix run github:ubugeeei-prod/vize#vize -- --help
```

You can also download platform-specific binaries from
[GitHub Releases](https://github.com/ubugeeei-prod/vize/releases).

For local development inside this repository, install the workspace build:

```bash
cargo install --path crates/vize --force --locked
```

## npm Package Scripts vs Rust CLI

| Need                                                                   | Recommended entry point                          |
| ---------------------------------------------------------------------- | ------------------------------------------------ |
| Package scripts for build, format, lint, check, ready, and upgrade     | `vp run vize:*` from the npm package             |
| Project-backed type checking across `.vue`, `.ts`, `.tsx`, and `.d.ts` | Rust `vize check`                                |
| LSP, IDE setup, `check-server`, and profiling artifacts                | Rust `vize` binary                               |
| Shared Vite plugin, npm package command, and Rust CLI settings         | [Vite config](./configuration.md#standalone-cli) |

## Commands

```bash
vize [COMMAND]
```

When invoked without a command, `vize` defaults to `build`.

| Command          | Description                                                       |
| ---------------- | ----------------------------------------------------------------- |
| `build`          | Compile Vue SFC files                                             |
| `fmt`            | Format Vue SFC files                                              |
| `lint`           | Lint Vue SFC files                                                |
| `check`          | Type check Vue SFC, TS, TSX, and `.d.ts` inputs                   |
| `explain`        | Print the documentation page for a diagnostic code                |
| `doctor`         | Analyze whole-application health                                  |
| `inspector`      | Create playground compiler inspector payloads                     |
| `clean`          | Remove Vize-generated cache artifacts                             |
| `lib`            | Pull versioned UI/composable sources ([guide](./lib-pull.md))     |
| `ready`          | Run `fmt`, `lint`, `check`, and `build`                           |
| `upgrade`        | Update the installed CLI                                          |
| `check-server`   | Start the Unix JSON-RPC typecheck server                          |
| `content-mapper` | Start the [TypeScript content-mapper](./content-mapper.md) server |
| `musea`          | Musea subcommands and scaffolding                                 |
| `lsp`            | Start the language server                                         |
| `ide`            | Install or manage editor integrations                             |

## Build

```bash
vize build src/**/*.vue
vize build --ssr
vize build --profile src
```

Key options:

| Option                        | Description                                                            |
| ----------------------------- | ---------------------------------------------------------------------- |
| `-o, --output`                | Source-relative output below the common input root; rejects collisions |
| `-f, --format`                | Output format: `js`, `json`, `stats`                                   |
| `--ssr`                       | Enable SSR compilation                                                 |
| `--vapor`                     | Compile in Vapor mode                                                  |
| `--template-syntax`           | `standard` (default), `strict`, or `quirks` handling of invalid syntax |
| `--custom-renderer`           | Treat lowercase non-HTML tags as custom renderer elements              |
| `--custom-elements <PATTERN>` | Tag patterns compiled as custom elements; repeat for multiple patterns |
| `--script-ext`                | `preserve` or `downcompile`                                            |
| `--declaration`               | Emit `.d.ts` files for the built SFCs (alias: `--dts`)                 |
| `--declaration-dir`           | Declaration output directory (default: the build output directory)     |
| `-j, --threads`               | Thread count override                                                  |
| `--profile`                   | Print timing profile                                                   |
| `--continue-on-error`         | Keep compiling and report failures at the end                          |

`<style lang="scss">`, `sass`, `less` and `stylus` blocks are compiled with the preprocessor installed in your project (`sass` or `sass-embedded`, `less`, `stylus`) before they are scoped, as Vite does. Preprocessor errors point to the line in the `.vue` file. If Node or the package is missing, the block is emitted as written and the build reports a warning.

## Format

```bash
vize fmt --check src
vize fmt --write src
```

Default discovery formats Vue, JavaScript/TypeScript and JSON/JSONC files. YAML and Markdown formatting is not implemented: `.yaml`, `.yml`, `.md` and `.markdown` are excluded from defaults, and explicit selection reports an error without changing their bytes. Key options:

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
| `--preset`                         | Use Prettier's Vue layout with `prettier` |
| `--profile`                        | Print timing profile                                 |

## Lint

```bash
vize lint src
vize lint --preset opinionated src
vize lint --help-level short src
```

Key options:

| Option                               | Description                                                                                                                             |
| ------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| `--fix`                              | Apply safe autofixes from rules that provide text edits, then report remaining diagnostics                                              |
| `-f, --format`                       | Output format: `text`, `rich` (localize with `--locale ja` or `zh`), `ansi`, `plain`, `json`, `stylish`, `markdown`, `html`, or `agent` |
| `--max-warnings`                     | Fail when warnings exceed the limit                                                                                                     |
| `--report-unused-disable-directives` | Report `eslint-disable` template comments that suppressed nothing, as `vue/comment-directive` errors                                    |
| `-q, --quiet`                        | Show summary only                                                                                                                       |
| `--help-level`                       | `full`, `short`, or `none`                                                                                                              |
| `--locale`                           | Message language: `en`, `ja`, or `zh`                                                                                                   |
| `--preset`                           | `happy-path`, `ecosystem`, `opinionated`, `essential`, `incremental`, or `nuxt`                                                         |
| `--cross-file`                       | Enable opt-in cross-file checks                                                                                                         |
| `--cross-file-tree`                  | Print the provide/inject tree when cross-file linting is enabled                                                                        |
| `--cross-file-complexity`            | Print the cross-file complexity score and hotspots when cross-file linting is enabled                                                   |
| `--type-aware`                       | Enable the type-aware rules configured in your lint settings                                                                            |
| `--strict-reactivity`                | Enable native checker-backed reactivity-loss linting                                                                                    |
| `--profile`                          | Print timing profile                                                                                                                    |
| `--slow-threshold`                   | Slow file threshold for profile output                                                                                                  |

Presets are intended for staged adoption:

| Preset        | Use it when                                                            |
| ------------- | ---------------------------------------------------------------------- |
| `essential`   | You want correctness-oriented diagnostics in CI                        |
| `happy-path`  | You want the default recommended bundle                                |
| `ecosystem`   | You want `happy-path` plus Router, Pinia, Nuxt, and i18n [ecosystem rules](../rules/ecosystem.md) |
| `opinionated` | You want stronger conventions, script rules, and type-aware candidates |
| `incremental` | You only want explicitly configured rules                              |
| `nuxt`        | You want opinionated rules with Nuxt component assumptions             |

Examples:

```bash
vize lint --preset essential --max-warnings 0 src
vize lint --preset opinionated --help-level short src
vize lint --cross-file --cross-file-tree src
vize lint --strict-reactivity src
vize lint --format agent src
```

## Check

```bash
vize check
vize check src
vize check --tsconfig tsconfig.app.json
vize check --profile src
```

`vize check` type-checks Vue SFCs with [TypeScript's native compiler](https://github.com/microsoft/typescript-go)
(Project Corsa, the Go port that ships as TypeScript 7). Vize generates virtual TypeScript for each
SFC, runs project diagnostics natively through [`corsa-bind`](https://github.com/ubugeeei/corsa-bind),
and maps every result back to the original `.vue` location. Flags and config keys that mention
"Corsa" refer to that native TypeScript compiler.


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
| `--save-virtual-ts-for` | Save one file's virtual TypeScript next to it; repeatable |
| `--max-warnings`    | Fail when warnings exceed the limit                |
| `--no-check-props`  | Skip prop type checks                              |
| `--no-check-emits`  | Skip emit type checks                              |
| `--no-check-template-bindings` | Skip template binding checks            |
| `-q, --quiet`       | Show summary only                                  |
| `--profile`         | Write profile artifacts under `node_modules/.vize` |
| `--corsa-path`      | Override the native TypeScript (Corsa) executable path |
| `--servers`         | Parallel Corsa CLI servers (`1`–`32`); auto-tuned when omitted |
| `--checkers`        | Checker workers per Corsa process (default `1`)    |
| `--declaration`     | Emit `.d.ts` output                                |
| `--declaration-dir` | Output directory for emitted declarations          |

Use `--corsa-path` to pin a local Corsa build. The shared config key is `typeChecker.corsaPath`;
`typeChecker.tsgoPath` remains only as a compatibility alias.

Project-wide template values and Vue ambient types should be visible through TypeScript project
configuration. Include generated files such as `auto-imports.d.ts`, `components.d.ts`, or your own
`declare module "vue"` augmentations in the `tsconfig.json` `include` globs, then select that project
with `--tsconfig` when needed:

```json
{
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue", "src/**/*.d.ts"]
}
```

## Doctor

```bash
vize doctor
vize doctor src
vize doctor src packages/shared --format json
```

`vize doctor` builds one application graph across Vue SFCs and script modules and reports
whole-app problems that a single-file linter cannot see:

- dependency injection (`provide` / `inject`) and cross-component prop contracts
- duplicate element IDs and server/client boundaries
- reactivity flow, asynchronous mutation risks, and setup-context ownership
- circular imports

Findings point at the authored `.vue` source. Discovery follows your ignore files, stays inside the
workspace root, stops with an error when a file cannot be parsed, and never writes files.

Key options:

| Option         | Description                                                                     |
| -------------- | ------------------------------------------------------------------------------- |
| `--root`       | Workspace boundary and base for report paths; defaults to the current directory |
| `-f, --format` | Output format: `text` (default), versioned `json`, or `sarif` for code hosts    |
| `--exit-zero`  | Return success even when a proven error would normally block                    |

Exit status `0` means the gate passed, `1` means a certain or high-confidence error, and `2` means
discovery, parsing, analysis, serialization, or output failed. Warnings and lower-confidence findings
affect the health score but do not block by default. See [Doctor diagnostics](doctor.md) for precise
filter, changed-file, scoring, and automation contracts. The deterministic `--format json` report
carries explicit format and scoring versions and is the preferred interface for CI and AI tooling:

```bash
vize doctor src --format json > doctor-report.json
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

Key options:

| Option              | Description                              |
| ------------------- | ---------------------------------------- |
| `-f, --format`      | Output format: `url`, `json`, `agent`, or `compare` |
| `--target`          | Compiler target: `dom`, `ssr`, or `vapor` |
| `--playground-url`  | Playground base URL for generated links  |
| `--max-files`       | Limit files included in a batch payload  |
| `--custom-renderer` | Enable custom renderer comparison        |
| `--template-syntax` | Choose `standard`, `strict`, or `quirks` |
| `-o, --output`      | Write the URL or JSON payload to a file  |

See [Compiler Inspector](./compiler-inspector.md) for the contributor workflow.

## Clean

```bash
vize clean
vize clean --dry-run
vize clean --scope node-modules
vize clean --scope project
vize clean --force
vize clean path/to/project
```

`vize clean` deletes the caches and reports Vize writes under `.vize` and `node_modules/.vize`:
profile output, Musea reports and snapshots, lint sessions, config schemas, LSP logs, and the
generated TypeScript project used by `vize check`.

| Option                              | Description                                               |
| ----------------------------------- | --------------------------------------------------------- |
| `--dry-run`                         | Print what would be removed                               |
| `--scope node-modules` / `project` | Clean only `node_modules/.vize` or only `.vize`           |
| `--force`                           | Also remove unknown entries by deleting the whole folder |

## Ready

```bash
vize ready src
vize ready --output dist src
```

`vize ready` runs `fmt --write`, `lint`, `check`, and `build` in order, stopping at the first failing step.
Interactive stderr shows numbered stages and elapsed completion times; CI and redirected stderr keep plain stage logs. Progress honors the terminal's color and Unicode profile.

Key options:

| Option         | Description                         |
| -------------- | ----------------------------------- |
| `-o, --output` | Output directory for the build step |
| `--ssr`        | Enable SSR compilation for build    |
| `--script-ext` | `preserve` or `downcompile`         |
| `--tsconfig`   | `tsconfig.json` for the check step  |
| `--no-check-props`, `--no-check-emits`, `--no-check-template-bindings` | Skip those type checks |

## Upgrade

```bash
vize upgrade
vize upgrade --dry-run
```

By default, `vize upgrade` updates the npm package through Vite+, which uses your project's package manager:

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

The `musea` subcommand currently focuses on scaffolding and experimental entry points. For
day-to-day gallery development, prefer `@vizejs/vite-plugin-musea`. The npm package also exposes a
convenience `vize musea` command that runs Vite with the Musea plugin installed (`vp exec vize
musea`, or add `--build` for a production build).

## LSP and IDE

```bash
vize lsp
vize lsp --port 9527
vize ide vscode
vize ide zed
```

`vize lsp` starts the language server directly. `vize ide` adds editor-specific install and
management commands for the VS Code and Zed integrations.

## Global Options

```bash
vize --help
vize --version
vize <command> --help
```
