---
title: Nuxt
---

# Nuxt Integration

> **⚠️ Work in Progress:** Vize is under active development and is not yet ready for production use. Test thoroughly before adopting in Nuxt projects.

Vize provides first-class Nuxt integration through the `@vizejs/nuxt` module. This replaces Nuxt's default Vue compiler with Vize's Rust-native compiler, providing the same speed improvements in Nuxt projects.

## Getting Started

### 1. Install the Module

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the module:

```bash
vp install @vizejs/nuxt
```

If you want to use `pkl` config with pnpm, you might need to install the `vize` package itself.
`@vizejs/nuxt` installs `vize` which serves `vize.pkl` with default config, but the location of `vize.pkl` may differ when using pnpm.

```bash
vp install vize
```

### 2. Register the Nuxt Module

```ts
// nuxt.config.ts
export default defineNuxtConfig({
  modules: ["@vizejs/nuxt"],
  vize: {
    compiler: true,
  },
});
```

### 3. Start Nuxt

Start the dev server as usual:

```bash
vp run dev
```

The module injects `@vizejs/vite-plugin` into Nuxt's Vite config and keeps Nuxt-specific transforms
in the pipeline, so auto-imports, components, middleware, and SSR behavior continue to work through
Nuxt.
During development, the server response cleanup preserves valid URL-encoded Nuxt asset links such
as `%40fs/` and encoded `assets/` paths while dropping decoded null-byte or traversal paths.

## Module Options

`@vizejs/nuxt` keeps the simple `compiler: true | false` switch, but the module options also expose
the Vize compiler and Nuxt compatibility bridges for projects that need tighter control:

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
      handleNodeModulesVue: false,
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

| Option                | Type                                 | Default                    | Description                                                                                                                                                                                 |
| --------------------- | ------------------------------------ | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `compatibility`       | `VizeNuxtCompatibilityOptions`       | auto-detected              | Overrides detected Nuxt/Vue major versions for unusual wrappers. Nuxt 2 defaults to Vue 2 host-compiler compatibility; Nuxt 3/4 defaults to Vue 3. Vue 0.11/1/2 all use host-compiler mode. |
| `compiler`            | `boolean \| VizeNuxtCompilerOptions` | `true`                     | Enables Vize as the Vue SFC compiler. Passing an object forwards options to `@vizejs/vite-plugin` while keeping Nuxt defaults for `root`, `devUrlBase`, and on-demand `scanPatterns`.       |
| `bridge`              | `boolean \| VizeNuxtBridgeOptions`   | `true`                     | Controls the Nuxt transform bridge for auto-imports, component imports, i18n helpers, and stable async-data keys on Vize virtual modules.                                                   |
| `unocss`              | `boolean \| VizeNuxtUnoCssOptions`   | `true`                     | Controls the UnoCSS bridge for Vize virtual modules. `originalSource: false` disables reading source SFCs; `maxBytes` limits memory use.                                                    |
| `dev.stylesheetLinks` | `boolean`                            | `true`                     | Enables dev-only SSR HTML stylesheet-link cleanup for Vize-generated Nuxt asset URLs.                                                                                                       |
| `musea`               | `boolean \| MuseaOptions`            | `false`                    | Opts into Musea gallery integration. Use `true` for Musea defaults or pass an object to configure include patterns, tokens, preview CSS, and routing.                                       |
| `nuxtMusea`           | `NuxtMuseaOptions`                   | `{ route: { path: "/" } }` | Documents the Nuxt mock shape used by Musea preview helpers. The Nuxt module does not install the mock layer globally because doing so would shadow Nuxt's own `#imports`.                  |

## Advanced Setup

### Nuxt 2 and Legacy Vue

Nuxt 2 projects use Vue 2 compiler output. Vize's native SFC compiler targets Vue 3, so the Nuxt
module automatically avoids replacing the host compiler when it detects Nuxt 2. For Nuxt 2 Bridge
or other Vite-based Vue 2 setups, the Vite plugin receives `vueVersion: 2`, which keeps
`@vitejs/plugin-vue2`, `vue-loader`, or Nuxt's own compiler in charge of `.vue` files.

The same host-compiler mode is available for older Vue projects via `vueVersion: 0.11`,
`vueVersion: 1`, or `vueVersion: "legacy"`.

If your project wraps Nuxt in a way that hides the version from Nuxt Kit, set the compatibility
override explicitly:

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

### Using the Vite Plugin Directly

Alternatively, you can use the Vite plugin directly. Since Nuxt uses Vite under the hood, this works but lacks some Nuxt-specific optimizations:

```ts
// nuxt.config.ts
import vize from "@vizejs/vite-plugin";

export default defineNuxtConfig({
  vite: {
    plugins: [vize()],
  },
});
```

## Musea Integration

The Nuxt module also supports Musea (component gallery) integration:

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

When configured, the Musea gallery is available at `/__musea__/` during development.

### Art File Placement

Nuxt component auto-discovery scans `.vue` files inside configured component directories. Because
Musea art files also end in `.vue`, keep `*.art.vue` files outside those directories in Nuxt
projects and point Musea at that location:

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

When Musea is enabled through `@vizejs/nuxt`, the module also excludes `**/*.art.vue` from Nuxt's
component scanner so colocated legacy files do not reach Nuxt's webpack or Vite component pipeline.

### Preview Setup for Nuxt

Nuxt projects often use features that need to be mocked in the Musea preview environment (vue-i18n, NuxtLink, useNuxtApp, etc.):

```ts
// musea.preview.ts
import { createI18n } from "vue-i18n";
import { createRouter, createMemoryHistory } from "vue-router";
import type { MuseaPreviewSetup } from "@vizejs/vite-plugin-musea";

export default ((app) => {
  // Mock vue-i18n
  const i18n = createI18n({
    locale: "ja",
    messages: {
      ja: {
        /* ... */
      },
      en: {
        /* ... */
      },
    },
  });
  app.use(i18n);

  // Mock vue-router (for NuxtLink compatibility)
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: "/", component: { template: "<div />" } },
      { path: "/about", component: { template: "<div />" } },
    ],
  });
  app.use(router);

  // Register NuxtLink as RouterLink
  app.component("NuxtLink", app.component("RouterLink"));

  // Mock useNuxtApp if needed
  app.provide("nuxt-app", {
    $config: {
      public: {
        /* ... */
      },
    },
  });
}) satisfies MuseaPreviewSetup;
```

## How It Works

When the Nuxt module is installed:

1. **Vite plugin injection** — The module registers `@vizejs/vite-plugin` as a Vite plugin, intercepting `.vue` file compilation.
2. **Compatibility shim** — The plugin exposes a `@vitejs/plugin-vue` compatibility API, so Nuxt's internal checks (which probe for the Vue plugin) work correctly.
3. **SSR support** — Vize's `vize_atelier_ssr` handles server-side compilation. The plugin isolates client and server environment variables to prevent cross-contamination.
4. **Nuxt features preserved** — Auto-imports, composables, middleware, and other Nuxt features work through Nuxt's own transform layer, which runs after Vize's compilation.

## Real-World Example

The [Vue Fes Japan 2026](https://vuefes.jp/2026) conference website uses Vize with Nuxt 4:

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

This configuration uses Musea for component development and documentation while keeping Nuxt's default compiler for production builds.

## Notes

- Vize is under active development — test thoroughly before using in production Nuxt projects
- SSR compilation is supported via `vize_atelier_ssr`
- Nuxt-specific features (auto-imports, composables, middleware) work through Nuxt's own transform layer
- The Nuxt module supports Nuxt 2, Nuxt 3, and Nuxt 4. Nuxt 2 uses host-compiler compatibility mode because Vize's native SFC compiler targets Vue 3 output.
