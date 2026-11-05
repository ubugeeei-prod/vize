---
title: Static Analysis
---

# Static Analysis

Use linting to catch Vue mistakes and enforce your team's rules, then add type checking to validate component props, events and template expressions. Both report findings against the original source, so the same code can be checked in CI and in the editor.

The examples below assume the `vize` npm package is installed in the project. Add the commands to `package.json` and run them through your package manager.

<span id="pipeline"></span>

## Choose a check

| What you want to check | Command or integration | Start here |
| --- | --- | --- |
| Vue rules, accessibility, script and CSS conventions | `vize lint src` | [Lint execution settings](./lint-execution.md) |
| Component props, events and template types | The content mapper or `vize check` | [Type checking](./type-checking.md) |
| Relationships between components | `vize lint --cross-file src` | [Component trees and complexity](./cross-file-complexity.md) |
| Findings while editing | Vize's language server | [Editor setup](./language-server.md) |
| Vue lint rules in an existing Oxlint run | `oxlint-plugin-vize` | [Oxlint integration](./oxlint.md) |

The [rule reference](../rules/index.md) lists each rule's default severity, presets, options and examples.

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

Use `essential` for correctness-only CI, `happy-path` for the default recommended bundle, `ecosystem` to add Router, Pinia, Nuxt, and i18n rules on top,
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

## Reactivity Overlay

For each SFC, the analyzer records where reactive values come from, where `.value` is needed, where
reactivity is lost, and which effects depend on which values, all mapped to the source. The same
data feeds diagnostics, reports, the editor, and the Playground's **Reactivity** tab.

<span id="patina-rule-model"></span>

## Lint rules and presets

Each lint rule checks a particular problem and has its own severity, options and examples. A preset selects a group of rules; an explicit rule setting can change that selection. Rules with automatic fixes can be applied with `vize lint --fix src`; review the resulting diff before committing it.

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
| `ecosystem`   | `happy-path` plus Router, Pinia, Nuxt, and i18n rules                |
| `opinionated` | `happy-path` plus stronger conventions, script rules, and type rules |
| `nuxt`        | Opinionated rules adjusted for Nuxt auto-import assumptions          |
| `incremental` | Empty starting point for host-driven, rule-by-rule adoption          |

## Migration Pragmas and Custom Rules

Vize accepts existing ESLint disable comments for matching rule names, including
`eslint-disable`, `eslint-enable`, `eslint-disable-next-line`, and `eslint-disable-line`. This lets
projects migrate rules such as `vue/require-v-for-key` without rewriting every suppression comment
up front.

Project-local JavaScript rule modules are not a stable Vize runtime API yet. During migration, keep
those rules in ESLint or Oxlint and run them beside `vize lint`, or use the `incremental` preset to
enable only built-in Vize rules that already match your policy. The `rules` config object controls
built-in Vize rule severities by name.

For the common case of forbidding a runtime-environment global (typical sidecar ESLint rules such as
`no-access-process`, `no-access-local-storage`, or `no-restricted-globals` against `localStorage` /
`sessionStorage`), enable the opt-in built-in `script/no-restricted-globals` rule instead of keeping
ESLint installed for those alone. Its default deny list is `process`, `localStorage`, and
`sessionStorage`, reported on each bare reference.

Several rules accept project-local configuration under `linter.ruleOptions`; see
[Rule Options](../rules/options.md) for the full typed inventory. The script example below shows
`script/no-restricted-globals` replacing the built-in `process` / `localStorage` / `sessionStorage`
deny list and `script/no-restricted-members` flagging configured `<object>.<property>` accesses.

```json
{
  "linter": {
    "rules": {
      "script/no-restricted-globals": "error",
      "script/no-restricted-members": "error"
    },
    "ruleOptions": {
      "script/no-restricted-globals": {
        "globals": [
          { "name": "process", "message": "Read env via a typed helper." },
          { "name": "alert" }
        ]
      },
      "script/no-restricted-members": {
        "members": [
          { "object": "window", "property": "localStorage", "message": "Use authStorage." }
        ]
      }
    }
  }
}
```

## Cross-File Rules

Cross-file linting checks relationships that a single component cannot establish on its own. It is off by default. Enable it when the run includes the related Vue files you want to check.

Today, `vize lint --cross-file` enables provide/inject matching, unique element ID checks,
reactivity tracking, and async race-condition analysis. `--cross-file-tree` prints the
provide/inject tree on top of those diagnostics.

```bash
vp run vize:lint:cross-file
vp run vize:lint:cross-file-tree
```

See [component trees and complexity](./cross-file-complexity.md) for the reports and [cross-file rule examples](../rules/cross-file.md) for the checks currently exposed to users.

## Type Checking

> [!TIP]
> Type-check with the [content mapper](./content-mapper.md). TypeScript 7.1 runs `tsc --runExternalCode --noEmit`. tsrs is a drop-in of the same `tsconfig`: `tsrs --runExternalCode --noEmit`. The setup and the `vue-tsc` map are in [Type Checking](./type-checking.md).

```json
{
  "scripts": {
    "typecheck": "tsc --runExternalCode --noEmit",
    "typecheck:tsrs": "tsrs --runExternalCode --noEmit"
  }
}
```

`vize check` checks the project's `.vue`, `.ts`, `.tsx` and `.d.ts` inputs and reports diagnostics at their original source locations.

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

A directly installed Rust CLI exposes the same commands:

```bash
nix run github:ubugeeei-prod/vize#vize -- check --tsconfig tsconfig.app.json --profile src
vize check --tsconfig tsconfig.app.json --profile src
vize lsp
```

Use npm package scripts to manage the tool and runtime with your application dependencies. The npm
launcher calls the Rust CLI through NAPI. Its shared resolver uses your project's installed native
TypeScript executable, respecting explicit settings such as `CORSA_PATH`. Vize does not bundle
or download that compiler. A direct Rust CLI installation uses the same resolution rules and is
also available through Nix and other installation methods.

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
