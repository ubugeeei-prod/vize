# @vizejs/rspack-plugin

High-performance Rspack plugin for Vue SFC compilation powered by [Vize](https://github.com/ubugeeei-prod/vize).

> [!NOTE]
> Rspack intentionally uses the dedicated `@vizejs/rspack-plugin` path instead of an `@vizejs/unplugin/rspack` export.
> Its loader chain, native CSS handling, and HMR behavior need Rspack-specific handling.
>
> Non-Vite bundler integrations are still unstable.
> If you need Rollup, Rolldown, Webpack, esbuild, or Babel, use `@vizejs/unplugin` and test carefully before relying on it in production.

## Features

- ⚡ **Blazing Fast** - Powered by Rust-based `@vizejs/native` compiler
- 🔄 **SFC HMR** - State-preserving style/template updates for compatible Vue 3 VDOM components; script updates reload the component
- 🎨 **CSS Processing** - Support for both Rspack native CSS (`experiments.css` in Rspack 1.x, default capability in Rspack 2.x) and CssExtractRspackPlugin
- 📦 **CSS Modules** - First-class CSS Modules support with per-module HMR
- 🔗 **`<style src>` Support** - Resolves external style files with watch dependency tracking
- 🔧 **TypeScript** - Full TypeScript support with auto-detection and built-in SWC stripping by default
- 🗄️ **Compilation Cache** - Content-hash based caching to skip re-compilation of unchanged files
- 🛠️ **Vue DevTools** - Exposes `__file` for component file path in development mode
- 🧩 **Custom Elements** - Auto-detect `.ce.vue` or configure via `customElement` option

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the packages:

```bash
vp install -D @vizejs/rspack-plugin @rspack/core
```

## Usage

> [!IMPORTANT]
> **Rspack 2.x**: native CSS is the default — you don't need to set `experiments.css` or `css: { native: true }`. (`experiments.css` is deprecated in 2.x but still works; the recommended path is to declare CSS rules with `type: "css/auto"`, which VizePlugin does for you.) Set `css: { native: false }` only to opt out and use a JS-based style pipeline (e.g. `CssExtractRspackPlugin` / `style-loader`).
> **Rspack 1.x**: native CSS is off by default. Set `experiments: { css: true }` to enable it. The plugin option alone does not enable the Rspack 1.x experiment.

### Simple Mode (Recommended)

Write a single `.vue` rule and your normal CSS rules. `VizePlugin` automatically clones your CSS rules for Vue style sub-requests and injects Rspack's built-in SWC post-processing for `.vue` TypeScript output.

```javascript
// rspack.config.mjs
import { VizePlugin } from "@vizejs/rspack-plugin";

const isProduction = process.env.NODE_ENV === "production";

export default {
  mode: isProduction ? "production" : "development",
  // Rspack 2.x: native CSS is on by default — nothing to configure here.
  // Rspack 1.x only, to enable native CSS: experiments: { css: true },

  module: {
    rules: [
      {
        test: /\.vue$/,
        use: [{ loader: "@vizejs/rspack-plugin/loader" }],
      },
    ],
  },

  plugins: [
    new VizePlugin({
      // Native CSS is the default on Rspack 2.x.
      // On Rspack 1.x, pass css: { native: true } and set experiments: { css: true }.
      // Pass css: { native: false } to opt out of native CSS.
    }),
  ],
};
```

### Native CSS with SCSS (Simple Mode)

Uses Rspack native CSS for optimal performance. On Rspack 2.x it's the default, so there's nothing extra to configure; on Rspack 1.x, add `experiments: { css: true }`. Just add your SCSS rule — VizePlugin handles the rest.

```javascript
// rspack.config.mjs
import { VizePlugin } from "@vizejs/rspack-plugin";
import path from "node:path";

const isProduction = process.env.NODE_ENV === "production";

export default {
  mode: isProduction ? "production" : "development",

  // Rspack 2.x: native CSS is on by default.
  // Rspack 1.x only, to enable native CSS: experiments: { css: true },

  module: {
    rules: [
      {
        test: /\.scss$/,
        type: "css/auto",
        use: ["sass-loader"],
      },
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
    ],
  },

  plugins: [new VizePlugin()],

  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "src"),
    },
  },
};
```

### CssExtractRspackPlugin (Simple Mode)

Compatible with webpack ecosystem, suitable for projects requiring PostCSS plugin chains.

```javascript
// rspack.config.mjs
import { rspack } from "@rspack/core";
import { VizePlugin } from "@vizejs/rspack-plugin";
import path from "node:path";

const isProduction = process.env.NODE_ENV === "production";

export default {
  mode: isProduction ? "production" : "development",

  module: {
    rules: [
      {
        test: /\.css$/,
        type: "javascript/auto",
        use: [isProduction ? rspack.CssExtractRspackPlugin.loader : "style-loader", "css-loader"],
      },
      {
        test: /\.scss$/,
        type: "javascript/auto",
        use: [
          isProduction ? rspack.CssExtractRspackPlugin.loader : "style-loader",
          "css-loader",
          "sass-loader",
        ],
      },

      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
    ],
  },

  plugins: [
    new VizePlugin({ css: { native: false } }),

    ...(isProduction
      ? [
          new rspack.CssExtractRspackPlugin({
            filename: "styles/[name].[contenthash:8].css",
            chunkFilename: "styles/[name].[contenthash:8].chunk.css",
          }),
        ]
      : []),
  ],

  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "src"),
    },
  },
};
```

## Detailed configuration

- [Manual `oneOf` rules and style request routing](./docs/manual-rules.md)
- [Plugin and loader API reference](./docs/api.md)
- [Migration guide](./MIGRATION.md)
- [SFC hot updates and browser regression tests](./docs/hmr.md)

## Configuration responsibilities

Rspack rules select files through `test`, `include`, and `exclude`. SFC and JSX
loader options configure compilation. `VizePlugin` configures CSS integration,
automatic rules, TypeScript post-processing, and diagnostic logging.

Compilation options on `VizePlugin` are deprecated and are not forwarded to loaders.
Existing configurations can be updated using the [migration guide](./MIGRATION.md).
With automatic rules, SFC loader `css.native` must match the plugin's resolved
CSS mode. With `autoRules: false`, each loader's explicit mode is preserved and
the plugin supplies a default only where the mode is unspecified. This applies
to static entries in nested `rules` / `oneOf` as well. Automatic
style-rule cloning targets the first matching root-level Vue rule without `oneOf`; nested
configurations must supply their own style routing. Function-valued `use` entries
are not rewritten; use static entries for plugin-managed CSS configuration.

Manual style routing retains control over loader order, preprocessor options,
CSS Modules, and rule-level parser/generator settings. Match each SFC loader's
CSS mode to its style sub-request pipeline. Automatic cloning copies the CSS
loader chain and type; it does not reproduce arbitrary CSS rule conditions or
parser/generator settings. Use manual routing for those configurations.

## License

MIT
