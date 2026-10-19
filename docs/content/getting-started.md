---
title: Getting Started
description: Add Vize to Vue with your Vite config and TypeScript project.
---

# Getting Started

Add Vize to your Vue 3 app with [Vite+](https://viteplus.dev/guide/install). Keep tool settings in `vite.config.ts` and the TypeScript project in `tsconfig.json`. Vize is under active development; review the [support status](./stability.md) before adoption.

## Choose your setup

| Your project                     | Start with                                              |
| -------------------------------- | ------------------------------------------------------- |
| Vue app on Vite+                 | The three steps below                                   |
| Vue app on plain Vite            | [Replace only the compiler](./guide/migration.md#vite-plugin) |
| Nuxt                             | [Nuxt integration](./integrations/nuxt.md)              |
| Any project, terminal or CI only | [Use Vize without Vite+](#use-vize-without-vite)        |
| Existing Oxlint setup            | [Oxlint plugin](./guide/oxlint.md)                      |
| Just the editor                  | [Language server](./guide/language-server.md)           |

## 1. Install the integration

Run this in an existing Vue project with Vite+ installed:

```bash
vp install -D @vizejs/vite-plugin
```

The integration uses your project's Vite+ version (0.2.3 or newer). For ordinary Vite, use the [Vite plugin migration](./guide/migration.md#vite-plugin); for Nuxt, use the [Nuxt integration](./integrations/nuxt.md).

## 2. Update `vite.config.ts`

The helper adds Vize's compiler and native check tasks. Remove the old Vue plugin import and `vue()` from `plugins`; preserve aliases, server settings, tests and unrelated plugins. The [migration guide](./guide/migration.md) shows complete before/after examples.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

## 3. Build and check your app

```bash
vp dev
vp build
vp run check
```

`vp run check` combines Vue type checking, lint and formatting with Oxlint and Oxfmt. Use focused tasks during development:

```bash
vp run typecheck
vp run lint
vp run fmt:check
```

Apply fixes with `vp run check -- --fix`.

> [!WARNING]
> Use `vp run check`, not `vp check`. The built-in `vp check`, `vp lint` and `vp fmt` run Vite+'s own tools and skip Vize's Vue checks.

If a script already exists, Vize preserves it and generates `vize:<name>`; see [task names and overrides](./guide/vite-plus.md#tasks).

## Choose your next step

- [Configure a rule or compiler option](./guide/configuration.md).
- [Migrate existing tools](./guide/migration.md) with complete before/after examples.
- [Understand a lint diagnostic](./rules/all.md) with bad and good Vue examples.
- [Type-check Vue files](./guide/type-checking.md) and compare with `vue-tsc`.
- [Try Vapor Mode](./guide/vapor.md) one component at a time.
- [Explore components](./guide/ui/index.md) with recipes and the component reference.
- [Preview your components](./guide/musea.md) with Musea art files and the gallery.
- [Set up your editor](./guide/language-server.md) for VS Code, Zed, Neovim and more.
- [Use the JavaScript APIs through JSR](./guide/jsr.md) from Node, Deno or Bun.

> [!NOTE]
> The CLI, Vite integration and editor share `vue:` settings in `vite.config.*` and use the TypeScript project and `vueCompilerOptions` from `tsconfig.json`. Dedicated `vize.config.*` files and top-level `vize:` settings are no longer supported; see the [configuration guide](./guide/configuration.md) for migration.

## Use Vize without Vite+

Try the [standalone CLI](./guide/cli.md) on any Vue project without installing anything:

```bash
vpx vize lint src
vpx vize check
vpx vize fmt --check src
```

It provides `vize lint`, `vize fmt` and `vize check`. Run from the target package root and keep the TypeScript project in `tsconfig.json`. Native commands can share top-level `vue` settings in `vite.config.*`; see [CLI configuration](./guide/configuration.md#standalone-cli) for discovery and its current scope.

Preview interactive setup with `vpx vize init --dry-run`, then run `vpx vize init` to select features. [Project Setup](./guide/init.md) explains detection and editing limits. Init uses project settings and defaults; it does not create a dedicated Vize config.
