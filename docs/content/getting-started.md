---
title: Getting Started
description: Add Vize to Vue with Vite+, one config file, and copyable commands.
---

# Getting Started

Add Vize to your Vue 3 app with [Vite+](https://viteplus.dev/guide/install).
Keep compilation, Vue linting, formatting, and type-checking settings in **`vite.config.ts`**.
Vize is under active development; review the [support status](./stability.md) before adoption.

## 1. Install the integration

From an existing Vue project with Vite+ installed:

```bash
vp install -D @vizejs/vite-plugin
```

The integration uses your project's Vite+ version (0.2.3 or newer).
For ordinary Vite, use the [Vite plugin migration](./guide/migration.md#vite-plugin).
For Nuxt, use the [Nuxt integration](./integrations/nuxt.md).

## 2. Update `vite.config.ts`

The Vite+ helper adds Vize's compiler and native check tasks.
Remove the old Vue plugin import and `vue()` from `plugins`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

Keep existing aliases, server settings, tests, and unrelated plugins.
The [migration guide](./guide/migration.md) shows the exact before/after diff
and how to carry supported Vue compiler options across.

## 3. Build and check your app

```bash
vp dev
vp build
vp run check
```

`vp run check` runs Vize's Vue type checker, linter, and formatter alongside
Oxlint and Oxfmt. Use focused tasks while working:

```bash
vp run typecheck
vp run lint
vp run fmt:check
```

To apply lint fixes and formatting, run `vp run check -- --fix`.

**Use `vp run` for combined Vize tasks.** Built-in `vp check`, `vp lint`, and
`vp fmt` run Vite+'s own tools. If a `check` script already exists, Vize preserves
it and generates `vize:check`; run `vp run vize:check`. The same rule applies to
existing `lint`, `fmt`, and `typecheck` scripts.
See [task names and overrides](./guide/vite-plus.md#tasks).

## Choose your next step

- [Configure a rule or compiler option](./guide/configuration.md) — one location per integration.
- [Migrate existing tools](./guide/migration.md) — source-to-target map and literal diffs.
- [Understand a lint diagnostic](./rules/all.md) — bad and good Vue examples.
- [Explore components](./guide/ui/index.md) — UI recipes and component reference.
- [Preview your components](./guide/musea.md) — Musea art files and gallery.
- [Set up your editor](./guide/vite-plus-editor.md) — editor responsibilities and native settings.

## Use Vize without Vite+

The [standalone CLI](./guide/cli.md) provides `vize lint`, `vize fmt`, and
`vize check`. Its settings belong in `vize.config.ts`; see
[standalone configuration](./guide/configuration.md#standalone-cli).

For an interactive setup of an existing Vite, Vite+, or Nuxt project, preview
proposed changes with `vpx vize init --dry-run`, then run `vpx vize init` to choose
features. [Project Setup](./guide/init.md) explains detection, flags, and editing limits.
