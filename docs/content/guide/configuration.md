---
title: Configuration
description: Put Vite+ settings in vite.config.ts; use vize.config.ts for standalone commands.
---

# Configuration

For Vite+ projects, configure Vize in **`vite.config.ts`** using the integration
helper. Start with the options you need; the defaults require no extra file.

## Vite+ configuration

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: { sourceMap: true },
  lint: { vize: { preset: "essential" } },
  fmt: { vize: { printWidth: 100 } },
  typecheck: { strict: true },
});
```

Settings are grouped by the tool that consumes them:

| Location | Controls | Run |
| --- | --- | --- |
| `compiler` | Vue compilation | `vp dev`, `vp build` |
| `lint.vize` | Vize's Vue rules | `vp run lint` |
| Other `lint` fields | Oxlint | `vp run lint` |
| `fmt.vize` | Vize's Vue formatting | `vp run fmt:check` |
| Other `fmt` fields | Oxfmt | `vp run fmt:check` |
| `typecheck` | Native Vue type checking | `vp run typecheck` |
| `pack.vize` | Vue library declarations | `vp run pack` |

`vp run check` combines the checks above. Built-in `vp check`, `vp lint`, and
`vp fmt` retain Vite+'s own behavior; use the generated tasks for Vize.
Existing scripts can rename generated tasks to `vize:<name>`.
See [Vite+ integration](./vite-plus.md#tasks) for task overrides and all integration options.

## Change one rule

Keep Vize rule names under `lint.vize.rules` and Oxlint rules under `lint.rules`:

```ts
export default defineConfig({
  lint: {
    vize: { rules: { "vue/no-v-html": "error" } },
    rules: { "no-debugger": "error" },
  },
});
```

[Browse rules and examples](../rules/all.md). Vize's native linter already runs
alongside Oxlint in this integration; no `oxlint-plugin-vize` registration is needed.

## Choose which features to adopt

Set `compiler`, `typecheck`, `lint.vize`, or `fmt.vize` to `false` to disable that
part of the integration. For example, `compiler: false` keeps your existing Vue
compiler plugin. Other Vite+ settings stay in the same `vite.config.ts`.

Vize formats Vue files and Oxfmt handles other files. `fmt.ignorePatterns`
excludes files from both. See [ownership and conflict handling](./vite-plus.md#lint-and-formatter-ownership).

## Ordinary Vite

Use plugin options directly in `vite.config.ts`:

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize({ sourceMap: true })],
});
```

See [Vite plugin options](./vite-plugin.md#compiler-options). A standalone shared
config is optional for settings also consumed by CLI or LSP commands.

## Standalone CLI

Install `vize` when running standalone commands or importing its config helper:

```bash
vp install -D vize
vp exec vize check
```

Its configuration belongs in **`vize.config.ts`**:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  linter: { preset: "essential" },
  formatter: { printWidth: 100 },
  typeChecker: { strict: true },
});
```

The standalone names `linter`, `formatter`, and `typeChecker` differ from the
Vite+ integration's `lint.vize`, `fmt.vize`, and `typecheck`.
Use [CLI commands](./cli.md) for this path; it does not require Vite+ tasks.

## Detailed reference

[Standalone configuration reference](./configuration-reference.md) preserves
file discovery and precedence, JSON/PKL examples, scoped entries, all compiler
options, template syntax modes, project Vue type resolution, and LSP/Musea settings.
For library declarations or editor setup, use the [Vite+ integration guide](./vite-plus.md).
