---
title: Static Analysis
---

# Static Analysis

Vize's analysis stack is shared by the compiler, linter, type checker, editor server, and Musea
tooling. The goal is to parse a Vue SFC once, keep rich semantic information around, and reuse it
for diagnostics and code generation instead of treating each command as a separate tool.

The examples below assume the `vize` npm package is installed and called from project scripts, which
is the recommended workflow for applications.

## Pipeline

| Layer    | What it does                                                              | Used by                                        |
| -------- | ------------------------------------------------------------------------- | ---------------------------------------------- |
| Armature | Tokenizes and parses Vue templates and SFC structure                      | compiler, linter, formatter                    |
| Croquis  | Builds scopes, binding metadata, macro information, and cross-file graphs | compiler, lint, type-aware checks              |
| Patina   | Runs Vue, script, CSS, a11y, SSR, Vapor, Musea, and type-aware lint rules | `vize lint`, editor diagnostics, Oxlint bridge |
| Canon    | Generates virtual TypeScript and maps diagnostics back to Vue files       | `vize check`, editor type checking             |
| Maestro  | Exposes diagnostics and editor features through LSP                       | `vize lsp`, VS Code, Zed                       |

This means static analysis is not only linting. Template bindings, compiler macros, component
metadata, provide/inject relationships, reactivity flow, generated virtual TypeScript, and
component gallery metadata all depend on the same lower-level analysis work.

For the concrete rule names, defaults, and cross-file diagnostic codes that can be emitted, see
[Rules](../rules/index.md).

## Linting

Start with the default preset:

```json
{
  "scripts": {
    "vize:lint": "vize lint src"
  }
}
```

```bash
vp run vize:lint
```

Use `essential` for correctness-only CI, `happy-path` for the default recommended bundle,
`opinionated` when you want stronger conventions, `nuxt` for Nuxt-aware assumptions, and
`incremental` when you only want explicitly configured rules to run.

```json
{
  "scripts": {
    "vize:lint:ci": "vize lint --preset essential --max-warnings 0 src",
    "vize:lint:opinionated": "vize lint --preset opinionated --help-level short src",
    "vize:lint:fix": "vize lint --fix src",
    "vize:lint:json": "vize lint --format json src"
  }
}
```

```bash
vp run vize:lint:ci
vp run vize:lint:opinionated
vp run vize:lint:fix
vp run vize:lint:json
```

Opt into cross-file and type-aware checks only after the basic lint path is stable:

```json
{
  "scripts": {
    "vize:lint:cross-file": "vize lint --cross-file src",
    "vize:lint:cross-file-tree": "vize lint --cross-file --cross-file-tree src",
    "vize:lint:strict-reactivity": "vize lint --strict-reactivity src"
  }
}
```

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
vp run vize:lint:strict-reactivity
```

Cross-file linting analyzes relationships such as provide/inject and reactivity flow across a set of
Vue files. `--strict-reactivity` enables the native checker-backed reactivity-loss rule, so expect it
to be slower than ordinary template and script lint rules.

## Patina Rule Model

Patina is the lint rule layer. Rules are small visitors over the SFC source, template root,
template elements, directives, `v-for`, `v-if`, and interpolations. Each rule carries metadata for
its rule name, category, default severity, help text, and whether it is fixable. Presets are just
registries that decide which rules are enabled together.

| Area                | Example rules                                                                                | What they cover                                    |
| ------------------- | -------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| Vue correctness     | `vue/require-v-for-key`, `vue/valid-v-model`, `vue/no-use-v-if-with-v-for`                   | Template semantics that are local to one component |
| Vue security        | `vue/no-v-html`, `vue/no-unsafe-url`                                                         | XSS-prone HTML and URL sinks                       |
| Vue structure       | `vue/sfc-element-order`, `vue/require-scoped-style`, `vue/no-unused-components`              | SFC shape, component usage, and maintainability    |
| Script conventions  | `script/no-options-api`, `script/no-get-current-instance`, `script/prefer-import-from-vue`   | Vue Composition API and compiler macro conventions |
| CSS                 | `css/no-important`, `css/no-hardcoded-values`, `css/prefer-logical-properties`               | Style blocks and design-system friendly CSS        |
| Accessibility       | `a11y/img-alt`, `a11y/anchor-has-content`, `a11y/label-has-for`                              | Accessible markup and interaction patterns         |
| HTML                | `html/deprecated-element`, `html/id-duplication`, `html/no-empty-palpable-content`           | HTML validity and semantic markup                  |
| SSR                 | `ssr/no-browser-globals-in-ssr`, `ssr/no-hydration-mismatch`                                 | Server/client rendering hazards                    |
| Vapor               | `vapor/no-vue-lifecycle-events`, `vapor/no-inline-template`, `vapor/require-vapor-attribute` | Vapor-oriented template constraints                |
| Musea               | `musea/require-title`, `musea/valid-variant`, `musea/prefer-design-tokens`                   | Component gallery and variant authoring            |
| Type-aware analysis | `type/require-typed-props`, `type/require-typed-emits`, `type/no-reactivity-loss`            | Rules that need semantic or checker-backed context |

The built-in presets are meant to support adoption in stages:

| Preset        | Shape                                                                |
| ------------- | -------------------------------------------------------------------- |
| `essential`   | Error-focused Vue correctness, security, and minimal HTML checks     |
| `happy-path`  | Default bundle for correctness, security, a11y, SSR, semantic checks |
| `opinionated` | `happy-path` plus stronger conventions, script rules, and type rules |
| `nuxt`        | Opinionated rules adjusted for Nuxt auto-import assumptions          |
| `incremental` | Empty starting point for host-driven, rule-by-rule adoption          |

## Migration Pragmas and Custom Rules

Patina accepts existing ESLint disable pragmas for matching rule names, including
`eslint-disable`, `eslint-enable`, `eslint-disable-next-line`, and `eslint-disable-line`. This lets
projects migrate rules such as `vue/require-v-for-key` without rewriting every suppression comment
up front.

Project-local JavaScript rule modules are not a stable Vize runtime API yet. During migration, keep
those rules in ESLint or Oxlint and run them beside `vize lint`, or use the `incremental` preset to
enable only built-in Vize rules that already match your policy. The `rules` config object controls
built-in Vize rule severities by name.

## Cross-File Rules

Cross-file analysis lives in Croquis and is exposed to linting through Patina diagnostics. It is
opt-in because it builds a module registry, import graph, component-usage graph, and additional
indexes across all analyzed Vue files.

Today, `vize lint --cross-file` enables provide/inject matching, unique element ID checks,
reactivity tracking, and async race-condition analysis. `--cross-file-tree` prints the
provide/inject tree on top of those diagnostics.

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
```

The lower-level cross-file engine is broader than the current CLI surface:

| Cross-file option         | Intended diagnostics or facts                                               |
| ------------------------- | --------------------------------------------------------------------------- |
| `provide_inject`          | Unmatched injects, unused provides, string-key warnings, non-reactive flows |
| `unique_ids`              | Duplicate IDs and non-unique IDs introduced inside loops                    |
| `reactivity_tracking`     | Prop destructuring, aliasing, and cross-component reactivity loss           |
| `race_conditions`         | Async state updates that can race through provided or shared state          |
| `fallthrough_attrs`       | `$attrs`, `inheritAttrs`, and multi-root fallthrough hazards                |
| `component_emits`         | Undeclared emits, unused emits, and listeners without a producer            |
| `event_bubbling`          | Events that bubble through component boundaries without being handled       |
| `server_client_boundary`  | Browser API usage and hydration risks around SSR/client boundaries          |
| `error_suspense_boundary` | Async components without useful Suspense or error boundaries                |
| `circular_dependencies`   | Import cycles and deep import chains                                        |
| `component_resolution`    | Unregistered or unresolved component usage                                  |
| `props_validation`        | Missing required props and child prop type mismatches                       |

The direction is to keep single-file linting fast by default, expose cross-file groups explicitly as
they mature, and route high-confidence project facts into the same diagnostic stream used by the
CLI, Oxlint bridge, and editor server.

## Type Checking

`vize check` generates virtual TypeScript for Vue SFCs and asks Corsa project sessions for
diagnostics. It checks `.vue`, `.ts`, `.tsx`, and `.d.ts` inputs and maps diagnostics back to the
original source files.

```json
{
  "scripts": {
    "vize:check": "vize check",
    "vize:check:src": "vize check src",
    "vize:check:app": "vize check --tsconfig tsconfig.app.json",
    "vize:check:json": "vize check --format json --quiet",
    "vize:check:virtual-ts": "vize check --show-virtual-ts src/components/App.vue",
    "vize:check:profile": "vize check --profile src",
    "vize:check:single-server": "vize check --servers 1 src",
    "vize:check:declarations": "vize check --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check
vp run vize:check:src
vp run vize:check:app
vp run vize:check:json
```

When no paths are provided, `vize check` reads `tsconfig.json` `files`, `include`, and `exclude`
fields if a project config is available. Use `--show-virtual-ts` when debugging generated code and
`--profile` when you need timing and virtual-file artifacts under `node_modules/.vize`.

```bash
vp run vize:check:virtual-ts
vp run vize:check:profile
vp run vize:check:single-server
```

Declaration output is available from the materialized checker project:

```bash
vp run vize:check:declarations
```

Project-wide template values and generated declaration files should be visible through TypeScript
project configuration. Put ambient declarations under a path included by your `tsconfig` and pass
that project file to the checker when needed:

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
    $route: { path: string };
  }
}
```

```bash
vp run vize:check:app
```

## npm Package Scripts vs Rust CLI

The npm `vize` package is intended for package scripts and uses the packaged NAPI binding:

```json
{
  "scripts": {
    "vize:lint": "vize lint src",
    "vize:check": "vize check src --strict",
    "vize:ready": "vize ready src"
  }
}
```

```bash
vp run vize:lint
vp run vize:check
vp run vize:ready
```

The Rust CLI currently has the fuller project-backed type-checking surface:

```bash
nix run github:ubugeeei-prod/vize#vize -- check --tsconfig tsconfig.app.json --profile src
vize check --tsconfig tsconfig.app.json --profile src
vize lsp
```

Use npm package scripts when you want installable workflows in an application. Use the Rust CLI when
you need `check-server`, LSP, IDE management, or the Corsa-backed project diagnostics path across
Vue and TypeScript files.

## Oxlint

Use `oxlint-plugin-vize` when your team already runs Oxlint and wants Vue-aware diagnostics in the
same command:

```bash
vp install -D oxlint oxlint-plugin-vize
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "preset": "essential",
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn"
  }
}
```

## Adoption Path

1. Add a `vize:lint:ci` package script such as `vize lint --preset essential src` to CI.
2. Switch to `happy-path` or `opinionated` after correctness diagnostics are clean.
3. Add a `vize:check` package script with your project `tsconfig.json`.
4. Enable editor linting first, then type checking once CI output is stable.
5. Add cross-file and strict reactivity checks for projects that benefit from deeper analysis.

For a single quality gate, a `vize:ready` package script running `vize ready src` runs `fmt
--write`, `lint`, `check`, and `build` in order and stops at the first failing step.
