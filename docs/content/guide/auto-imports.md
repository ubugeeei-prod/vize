---
title: Auto-imports for @vizejs/ui and @vizejs/composable
---

# Auto-imports

Auto-imports let you use components from `@vizejs/ui` and helpers from `@vizejs/composable`
without repeating import statements. Use them when those libraries appear throughout your app;
ordinary explicit imports remain useful for a small number of call sites.

Choose the [Nuxt setup](#nuxt) or the [Vite resolvers](#vite)
below, then try a component such as `<VzDialogTrigger>` in a template. If you copied library
sources into the app, use [local source resolution](#pulled-sources) instead.

Each resolver chooses a direct package subpath, such as `@vizejs/ui/dialog` or
`@vizejs/composable/use-toggle`, so bundlers can tree-shake it like a handwritten import.
Resolvers run at build time.

## Nuxt

Both integrations are opt-in options of `@vizejs/nuxt`:

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    ui: { prefix: "Vz", exclude: ["tooltip"] },
    composables: true,
  },
});
```

- `ui` registers every `@vizejs/ui` component with `addComponent` (`<VzDialogTrigger>`,
  `<VzButton>`, ...) and auto-imports the `use*` helpers the families export (`useFieldWiring`,
  `useSafeAreaInsets`, ...). Options: `prefix`, per-family `include` / `exclude`,
  `composables: false` to skip the helpers, and the [local mode](#pulled-sources) options.
- `composables` auto-imports every runtime export of `@vizejs/composable` with `addImports`.
  Options: `include` / `exclude` entries (`"use-toggle"`), `names`, and the local mode options.
  When both libraries export the same name, the `@vizejs/ui` registration wins.

Nuxt generates the typed `components.d.ts` / `imports.d.ts` declarations, so templates and scripts are
fully typed. The packages are resolved from your project, so the installed versions drive what is
registered.

<span id="vite"></span>

## Vite (`unplugin-vue-components` / `unplugin-auto-import`)

```ts
// vite.config.ts
import Components from "unplugin-vue-components/vite";
import AutoImport from "unplugin-auto-import/vite";
import { VizeUiResolver, VizeUiComposablesResolver } from "@vizejs/ui/resolver";
import { VizeComposableResolver } from "@vizejs/composable/resolver";

export default defineConfig({
  plugins: [
    vue(),
    Components({ resolvers: [VizeUiResolver({ prefix: "Vz" })] }),
    AutoImport({ resolvers: [VizeUiComposablesResolver(), VizeComposableResolver()] }),
  ],
});
```

Prefer presets? `vizeUiImports()` and `vizeComposableImports()` return
`{ "<module>": ["name", ...] }` maps for `AutoImport({ imports: [...] })`. For custom setups,
`createVizeUiComponentDeclarations()` and `createVizeComposableDeclarations()` render the
`GlobalComponents` / global `.d.ts` declarations the plugins would otherwise generate.

## Pulled sources

When you copy families or composables into the project with [`vize lib pull`](/guide/lib-pull), set
`source: "local"` so the pulled copies win:

```ts
VizeUiResolver({ source: "local" }); // reads ./vize-lib.lock.json
VizeComposableResolver({ source: "local", root: projectRoot, lockfile: "vize-lib.lock.json" });
```

Items recorded in `vize-lib.lock.json` resolve to `<root>/<dir>/<entry>` (for example
`src/components/vize/families/actions/button/button.ts`); everything else falls back to the package
unless `fallback: false`. The same options exist on the Nuxt `ui` and `composables` settings.

## Keeping the catalogs in sync

The `@vizejs/ui` resolver manifest is generated from the family catalog with
`pnpm --filter @vizejs/ui generate:resolver`; a test fails when a catalogued family is missing from it.
The composable resolver reads `COMPOSABLE_CATALOG` directly.
