---
title: Nuxt
description: Add Vize to a Nuxt project as one module for compiling, linting and previewing Vue components.
---

# Nuxt

`@vizejs/nuxt` adds Vize to a Nuxt project as a single module. It compiles `.vue` files with Vize's Rust compiler instead of Nuxt's default Vue compiler, generates a Nuxt-aware lint configuration, and can serve the [Musea](../guide/musea.md) component gallery. Auto-imports, components, middleware and SSR keep working through Nuxt.

> [!WARNING]
> Vize is under active development and is not ready for production. Compare build output and diagnostics on your own project before you adopt it, and check the [support status](../stability.md).

## Set up

### 1. Install the module

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the module:

```bash
vp install @vizejs/nuxt
```

### 2. Register the module

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

`compiler: true` is the default, so `modules: ["@vizejs/nuxt"]` alone is enough. The `vize` key is where every option on this page goes.

### 3. Start Nuxt

Start the dev server as usual:

```bash
vp run dev
```

The module adds `@vizejs/vite-plugin` to Nuxt's Vite config and keeps Nuxt's own transforms in the pipeline. Pages, layouts, auto-imported composables and components behave as before.

### 4. Check that Vize compiles your components

Turn on compiler debug logging to see which files Vize handles:

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: { debug: true },
  },
});
```

Nuxt compiles components on demand, so the log prints `[vize] load: on-demand compiling ...` only for the files a request actually reaches. Remove `debug` once you have confirmed the setup.

## Options

All options live under `vize` in `nuxt.config.ts`. Most projects only need the first four.

| Option                | Type                                    | Default                    | What it does                                                                                                                          |
| --------------------- | --------------------------------------- | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `compiler`            | `boolean \| VizeNuxtCompilerOptions`    | `true`                     | Compiles Vue SFCs with Vize. Pass an object to forward options to `@vizejs/vite-plugin`.                                              |
| `lint`                | `boolean \| VizeNuxtLintOptions`        | `true`                     | Generates the Nuxt-aware Oxlint configuration. See [Lint](#lint).                                                                     |
| `checker`             | `boolean \| VizeNuxtLintCheckerOptions` | `false`                    | Runs lint beside the dev server and reports in the terminal and browser overlay. See [Lint while developing](#lint-while-developing). |
| `musea`               | `boolean \| MuseaOptions`               | `false`                    | Serves the Musea gallery. See [Musea](#musea).                                                                                        |
| `ui`                  | `boolean \| VizeNuxtUiOptions`          | `false`                    | Auto-registers `@vizejs/ui` components. See [auto-imports](../guide/auto-imports.md#nuxt).                                            |
| `composables`         | `boolean \| VizeNuxtComposablesOptions` | `false`                    | Auto-imports `@vizejs/composable` helpers. See [auto-imports](../guide/auto-imports.md#nuxt).                                         |
| `bridge`              | `boolean \| VizeNuxtBridgeOptions`      | `true`                     | Re-applies Nuxt's auto-import, component, i18n and async-data key transforms to the modules Vize compiles.                            |
| `unocss`              | `boolean \| VizeNuxtUnoCssOptions`      | `true`                     | Lets UnoCSS read the original `.vue` source. `originalSource: false` turns this off; `maxBytes` caps the size read per file.          |
| `dev.stylesheetLinks` | `boolean`                               | `true`                     | Cleans up duplicate or unsafe stylesheet links in dev SSR HTML.                                                                       |
| `compatibility`       | `VizeNuxtCompatibilityOptions`          | detected                   | Overrides the detected Nuxt and Vue major versions. See [Nuxt 2 and Vue 2](#nuxt-2-and-vue-2).                                        |
| `nuxtMusea`           | `NuxtMuseaOptions`                      | `{ route: { path: "/" } }` | Describes the route, runtime config and fetch mocks that Musea previews see.                                                          |

### Compiler

Any `@vizejs/vite-plugin` option can be passed through `compiler`. Nuxt keeps its own defaults for the project root, on-demand compilation and `.vue` files in dependencies:

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compatibility: {
      // Usually inferred automatically.
      // Nuxt 2 defaults to Vue 2 compatibility mode; Nuxt 3/4 defaults to Vue 3.
      vueVersion: 3,
    },
    compiler: {
      // Any @vizejs/vite-plugin option can be passed here.
      configMode: "auto",
      customRenderer: false,
      debug: false,
      handleNodeModulesVue: true,
      ignorePatterns: ["node_modules/**", ".nuxt/**", ".output/**"],
      precompileBatchSize: 64,
      scanPatterns: [], // Nuxt defaults to on-demand compilation
      sourceMap: true,
      vapor: false,
    },
    bridge: {
      autoImports: true,
      components: true,
      i18n: true,
      stableInjectedKeys: true,
    },
    unocss: {
      originalSource: {
        maxBytes: 2 * 1024 * 1024,
      },
    },
    dev: {
      stylesheetLinks: true,
    },
    musea: false,
  },
});
```

The [Vite plugin guide](../guide/vite-plugin.md#compiler-options) describes each compiler option. Set `compiler: false` to keep Nuxt's default compiler and use only lint or Musea.

### Lint

With `lint` enabled, the module writes `.oxlint.vize.json` to the project root. It is the Oxlint counterpart of the configuration that `@nuxt/eslint` generates: rule blocks that match your Nuxt version and directories. Add the file to `.gitignore`.

If the project has no Oxlint or Vite configuration yet, the module also creates an `oxlint.config.mts` that loads the generated file. An existing configuration is never rewritten; load `.oxlint.vize.json` from it yourself.

| Option       | Default               | What it does                                                                                                |
| ------------ | --------------------- | ----------------------------------------------------------------------------------------------------------- |
| `configFile` | `".oxlint.vize.json"` | Where the generated configuration is written. It must be in the lint root or one of its parent directories. |
| `autoInit`   | `true`                | Creates `oxlint.config.mts` when no Oxlint or Vite configuration exists.                                    |
| `rootDir`    | the Nuxt root         | The directory that generated globs are relative to.                                                         |
| `devtools`   | `{ enabled: "lazy" }` | Shows the resolved lint plan in a Nuxt DevTools tab.                                                        |

`lint` also accepts the feature switches of `@nuxt/eslint-config`: `standalone` (default `true`), `import` (default `true`), `typescript`, `stylistic`, `tooling`, `formatters` and `nuxt`.

Set `lint.autoInit: false` when your lint configuration lives outside the Nuxt root. The [Oxlint plugin guide](../guide/oxlint.md) explains how to run the rules.

### Lint while developing

`checker` runs Oxlint and Vize's lint rules next to the dev server, the way `@nuxt/eslint`'s checker does:

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    checker: true,
  },
});
```

| Option        | Default                                | What it does                                           |
| ------------- | -------------------------------------- | ------------------------------------------------------ |
| `lintOnStart` | `true`                                 | Lints the whole project when the dev server starts.    |
| `cache`       | `true`                                 | After startup, lints only the files that changed.      |
| `include`     | `<srcDir>/**/*.{js,jsx,ts,tsx,vue}`    | Files the checker looks at.                            |
| `exclude`     | `**/node_modules/**` and the build dir | Files and directories the checker skips.               |
| `emitWarning` | `true`                                 | Prints warnings and shows them in the browser overlay. |
| `emitError`   | `true`                                 | Prints errors and shows them in the browser overlay.   |
| `fix`         | `false`                                | Applies safe fixes before reporting.                   |
| `formatter`   | `"stylish"`                            | The Oxlint output format.                              |

## Musea

Enable the component gallery with `musea`:

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
    musea: {
      include: ["**/*.art.vue"],
      tokensPath: "assets/tokens.json",
      previewCss: ["assets/styles/main.css", "assets/styles/musea-preview.css"],
      previewSetup: "musea.preview.ts",
    },
    nuxtMusea: {
      route: { path: "/" }, // Musea UI route within __musea__
    },
  },
});
```

During development the gallery is served at `/__musea__/`. The [Musea guide](../guide/musea.md) covers art files, design tokens and visual regression tests.

### Where to put art files

Nuxt scans every `.vue` file in its component directories, and Musea art files also end in `.vue`. Keep `*.art.vue` files outside those directories and point Musea at them:

```txt
app/components/Tag.vue
stories/shared/Tag.art.vue
```

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    musea: {
      include: ["stories/**/*.art.vue"],
    },
  },
});
```

When Musea is enabled, the module also excludes `**/*.art.vue` from Nuxt's component scanner, so art files that still sit next to components are not registered as components.

### Make Nuxt features available in previews

Components often rely on `NuxtLink`, `useRoute`, `useNuxtApp`, `useRuntimeConfig`, data composables or built-in Nuxt components. In a standalone Musea Vite config, add `@vizejs/musea-nuxt` and install its mocks from `previewSetup`:

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { musea } from "@vizejs/vite-plugin-musea";
import { nuxtMusea } from "@vizejs/musea-nuxt";

export default defineConfig({
  plugins: [
    nuxtMusea({
      route: { path: "/preview" },
      runtimeConfig: { public: { apiBase: "/api" } },
      fetchMocks: {
        "/api/user": { id: 1, name: "Ada" },
      },
    }),
    musea({
      previewSetup: "musea.preview.ts",
    }),
  ],
});
```

```ts
// musea.preview.ts
import { installNuxtMuseaMocks } from "@vizejs/musea-nuxt";
import { createI18n } from "vue-i18n";
import type { MuseaPreviewSetup } from "@vizejs/vite-plugin-musea";

export default ((app) => {
  installNuxtMuseaMocks(app, {
    route: { path: "/preview" },
    runtimeConfig: { public: { apiBase: "/api" } },
  });

  const i18n = createI18n({
    locale: "ja",
    messages: {
      ja: {/* ... */},
      en: {/* ... */},
    },
  });
  app.use(i18n);
}) satisfies MuseaPreviewSetup;
```

Inside a Nuxt app the module does not install these mocks globally: they would replace Nuxt's own `#imports`. Real Nuxt composables are used there instead.

## Nuxt 2 and Vue 2

The module supports Nuxt 2, 3 and 4. Vize's compiler produces Vue 3 output, so on Nuxt 2 the module leaves `.vue` compilation to the existing compiler (`@vitejs/plugin-vue2`, `vue-loader` or Nuxt's own) and still provides lint and Musea. The same applies to older Vue projects through `vueVersion: 0.11`, `vueVersion: 1` or `vueVersion: "legacy"`.

The versions are detected automatically. If a wrapper hides them from Nuxt Kit, set them yourself:

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compatibility: {
      nuxtVersion: 2,
      vueVersion: 2,
    },
  },
});
```

### Migrating from Nuxt 2

When moving a Nuxt 2 / Vue 2.7 application to Nuxt 3 or 4, use
`vize.compiler.whitespace: "vue2-preserve"` to retain Vue 2's template text rules:

```ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: { whitespace: "vue2-preserve" },
  },
});
```

This keeps whitespace around contiguous text and interpolations as written, while
whitespace-only runs between elements become one space and edge runs are dropped.
`"vue2-line-breaks"` is a narrower opt-in for individual line breaks, not full Vue 2
preservation. The [compiler reference](../guide/compiler-configuration-reference.md#whitespace)
describes all four modes and the comment, named-slot and preformatted-text rules.
A project still running Nuxt 2 keeps its existing compiler as described above.

## Use the Vite plugin directly

Nuxt builds with Vite, so the Vite plugin also works without the module. You lose the Nuxt-specific bridges and lint generation described above:

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [vize()],
  },
});
```

## How it works

1. **Vite plugin.** The module registers `@vizejs/vite-plugin`, which compiles `.vue` files.
2. **Vue plugin compatibility.** The plugin exposes the same API as `@vitejs/plugin-vue`, so Nuxt's internal checks for the Vue plugin still pass.
3. **SSR.** Vize compiles the server build as well, and keeps client and server environment variables separate.
4. **Nuxt transforms.** Auto-imports, composables and middleware go through Nuxt's own transform layer, which runs after Vize has compiled a component.

Published `@vizejs/nuxt` releases pin `vize`, `@vizejs/vite-plugin`, `@vizejs/native` and the other Vize packages to the same version.

## Example: Vue Fes Japan 2026

The [Vue Fes Japan 2026](https://vuefes.jp/2026) website uses the module with Nuxt 4:

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: false, // compiler disabled (using Nuxt's default)
    musea: {
      include: ["**/*.art.vue"],
      inlineArt: false,
      tokensPath: "assets/tokens.json",
      previewCss: ["assets/styles/main.css", "assets/styles/musea-preview.css"],
      previewSetup: "musea.preview.ts",
    },
  },
});
```

It uses Musea to develop and document components and keeps Nuxt's default compiler for the production build.

## Troubleshooting

- **Pkl configuration with pnpm.** `@vizejs/nuxt` depends on `vize`, which ships the default `vize.pkl`. With pnpm that file may not be at `node_modules/vize`. Install `vize` directly if you use a Pkl configuration:

  ```bash
  vp install vize
  ```

- **A custom `lint.configFile` is ignored.** The file must be inside `lint.rootDir` or one of its parent directories. A configuration nested in `.nuxt/` cannot apply Oxlint's project ignores.
- **Asset links in dev SSR HTML.** `dev.stylesheetLinks` keeps valid URL-encoded Nuxt asset links, such as `%40fs/` and encoded `assets/` paths, and drops links that contain decoded null bytes or path traversal.

For anything else, see [troubleshooting](../guide/troubleshooting.md) or [report a problem](../guide/testing.md).
