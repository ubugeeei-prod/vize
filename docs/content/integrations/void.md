---
title: Void
description: Use Vize in a Void app with @void/vue pages, workerd SSR and Void's generated types.
---

# Void

[Void](https://void.cloud) builds full-stack apps on Vite and runs them on Cloudflare's workerd. Its Vue adapter, `@void/vue`, renders `pages/` on the server and hydrates them in the browser. Add `vize()` next to it and Vize compiles every `.vue` file; Void keeps its routing, loaders, islands and bindings.

> [!WARNING]
> Vize is under active development. Compare build output and diagnostics on your own app before you adopt it, and check the [support status](../stability.md).

## Set up

### 1. Install the plugin

```bash
npm install -D @vizejs/vite-plugin
```

### 2. Add `vize()` to `vite.config.ts`

```ts
import { voidVue } from "@void/vue/plugin";
import { vize } from "@vizejs/vite-plugin";
import { defineConfig } from "vite";
import { voidPlugin } from "void";

export default defineConfig({
  plugins: [voidPlugin(), vize(), voidVue()],
});
```

`voidVue()` ships its own `@vitejs/plugin-vue`. You do not need to remove it: `vize()` replaces it so that one compiler owns each SFC. If you set `exclude`, plugin-vue stays and compiles the files Vize skips.

### 3. Run Void as usual

```bash
npx vite dev
npx vite build
```

No other change is needed:

- **Server rendering on workerd.** Vue and `vue/server-renderer` are bundled into the worker, since workerd has no `node_modules` to import from.
- **Route CSS and preloads.** The client manifest is keyed by source path (`pages/index.vue`), the key Void uses to add a route's stylesheet to the server-rendered HTML.
- **HMR in `vite dev`.** A template edit keeps component state. Loader props stay in place.
- **Page props.** `defineProps<Props>()` with `Props` imported from a `.server.ts` loader compiles as with plugin-vue.

## Type checking

Void generates `.void/tsconfig.json` with route, environment and binding types; your `tsconfig.json` extends it. Run `void prepare` once (or start the dev server) so the files exist, then check:

```bash
npx void prepare
vize check
```

`vize check` reads the generated `files` and `paths`, so in `.vue` files these are typed like they are in `vue-tsc`:

- loader props from `InferProps<typeof loader>`,
- `fetch` from `void/client` (route paths, methods and response bodies),
- `env` from `void/env` and binding modules such as `void/kv`.

## Lint and format

`vize lint` and `vize fmt` find your files from the project root and skip what `.gitignore` lists. Void's `.gitignore` covers `dist`, `.void`, `.wrangler` and `.void-wrangler.jsonc`, so generated files are not linted or reformatted.

## Editor

The [language server](../guide/language-server.md) uses the same `tsconfig.json`. Run `void prepare` before opening the project so that the generated types exist.

## How this is tested

CI builds a pinned Void 1.1.12 app with `@void/vue` (`tools/support/compat/void/fixtures/void-vue`) on every change to the Vite plugin or native compiler. It checks:

- production build, and SSR in `vite preview` on workerd (including route CSS and a KV binding);
- dev SSR, hydration, and HMR that keeps state;
- that `vize check` and `vue-tsc` report the same errors;
- `vize lint` and `vize fmt --check`.

Nuxt on Void (`voidPlugin()` in `nuxt.config.ts`) is not covered by this fixture yet. Use the [Nuxt integration](./nuxt.md) there.
