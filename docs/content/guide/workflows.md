---
title: User Workflows
---

# User Workflows

This guide gives a compact path through the common Vize workflows: install it, connect config,
format, lint, type-check, compile, and run the same gates in CI.

## Install

Install the npm package in the project that owns your Vue dependencies:

```bash
vp install -D vize
```

For monorepos, install it at the workspace root when packages share one lockfile. Install it in a
package only when that package has its own lockfile and dependency graph.

## Add Package Scripts

Prefer named scripts over one-off commands so local and CI runs share the same entry points:

```json
{
  "scripts": {
    "vize:fmt": "vize fmt --check src",
    "vize:fmt:fix": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path --max-warnings 0 src",
    "vize:check": "vize check src",
    "vize:build": "vize build src",
    "vize:ready": "vize ready src"
  }
}
```

`vize ready` is the broad local gate. In larger repositories, keep the individual commands too so
developers can isolate formatting, lint, type-checking, and compiler failures.

## Configure Once

Create `vize.config.ts` at the project root when defaults are not enough:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  linter: {
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    tsconfig: "tsconfig.json",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

See [Configuration](./configuration.md) for flat monorepo entries, PKL, JSON, compiler options, and
Vue type resolution details.

## Format

Use check mode in CI and write mode locally:

```bash
vp run vize:fmt
vp run vize:fmt:fix
```

For one-off migration work, `vize fmt --write` can target a file, directory, or glob.

## Lint

Start with `happy-path` for correctness and low-noise Vue diagnostics:

```bash
vize lint --preset happy-path --max-warnings 0 src
```

Use `--help-level short` when CI output should stay compact, and `--format json` when another tool
will consume the diagnostics. See [CLI](./cli.md) and [Rules](../rules/index.md) for the full rule
surface.

## Type Check

Run `vize check` from the project root so the active `tsconfig`, Vue version, framework packages,
and ambient types come from the same dependency graph:

```bash
vize check src
```

For package-specific monorepo checks, run from the package directory or set `typeChecker.tsconfig`
in a scoped config entry.

## Compile

Use `vize build` when you need compiler output outside the Vite plugin path:

```bash
vize build src --output dist/vize
```

For Vite applications, prefer `@vizejs/vite-plugin` and let Vite own build orchestration. See
[Vite Plugin](./vite-plugin.md).

## CI

Use the same package scripts in CI:

```yaml
- run: vp install --frozen-lockfile
- run: vp run vize:fmt
- run: vp run vize:lint
- run: vp run vize:check
```

Keep `vize:build` in the gate only when the project consumes Vize compiler output directly. For
Vite applications, the normal app build exercises the plugin.

## Debug Failures

When a failure is unclear:

- rerun with `--format json` to inspect stable diagnostic fields;
- use `--profile` on `check`, `lint`, or `build` to find slow phases;
- create an inspector payload with `vize inspector` for compiler mismatches;
- include the smallest `.vue` file or project slice when reporting an issue.

The [Testing & Feedback](./testing.md) and [Troubleshooting](./troubleshooting.md) pages cover
reporting, real-world fixtures, and common environment problems.
