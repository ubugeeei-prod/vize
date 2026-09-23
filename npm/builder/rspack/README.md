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
- 🔄 **HMR Support** - Script/template hot reload via `module.hot` + `__VUE_HMR_RUNTIME__`, CSS Modules HMR with targeted rerender
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
    new VizePlugin({}),

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

### Advanced: Manual `oneOf` Rules

`VizePlugin` will automatically:

1. Find the `.vue` rule containing the vize loader
2. Clone your CSS/SCSS/Less/Stylus rules for `?vue&type=style` sub-requests
3. Inject the vize scope-loader + style-loader at the end of each cloned chain
   (execution order: style-loader extracts block → preprocessor compiles → scope-loader applies native scoped CSS)
4. Build `oneOf` branches inside the `.vue` rule
5. Add `resourceQuery: { not: [/vue/] }` to original CSS rules so they don't conflict

To opt out and write manual rules, set `autoRules: false`:

```javascript
new VizePlugin({ autoRules: false });
```

If you need full control over the loader chain, set `autoRules: false` and write `oneOf` branches manually.

#### Request Routing: Main vs Style Sub-Requests

When writing manual rules, you must ensure the main `.vue` loader and the style loader see different requests.

The main loader produces `import './App.vue?vue&type=style&index=0&...'` statements. These style sub-requests must be routed through `@vizejs/rspack-plugin/scope-loader` and `@vizejs/rspack-plugin/style-loader` — **not** back into the main loader. If the main loader receives a `?type=style` query it will emit an explicit error.

Use `oneOf` to guarantee mutual exclusion:

```javascript
{
  test: /\.vue$/,
  oneOf: [
    { resourceQuery: /type=style/, use: [/* style pipeline */] },
    { use: [/* main vize loader */] },
  ],
}
```

<details>
<summary>Native CSS with manual oneOf</summary>

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
        test: /\.vue$/,
        oneOf: [
          // CSS Modules (<style module>)
          {
            resourceQuery: /vue&type=style.*module/,
            type: "css/module",
            use: [
              { loader: "@vizejs/rspack-plugin/scope-loader" },
              { loader: "@vizejs/rspack-plugin/style-loader" },
            ],
          },

          // SCSS (<style lang="scss">)
          {
            resourceQuery: /vue&type=style.*lang=scss/,
            type: "css/auto",
            use: [
              { loader: "@vizejs/rspack-plugin/scope-loader" },
              "sass-loader",
              { loader: "@vizejs/rspack-plugin/style-loader" },
            ],
          },

          // Regular CSS (<style>)
          {
            resourceQuery: /vue&type=style/,
            type: "css/auto",
            use: [
              { loader: "@vizejs/rspack-plugin/scope-loader" },
              { loader: "@vizejs/rspack-plugin/style-loader" },
            ],
          },

          // Main .vue file (compile SFC → JS)
          {
            use: [
              {
                loader: "@vizejs/rspack-plugin/loader",
              },
            ],
          },
        ],
      },
    ],
  },

  plugins: [
    new VizePlugin({
      autoRules: false,
      css: { native: true },
    }),
  ],

  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "src"),
    },
  },
};
```

</details>

<details>
<summary>CssExtractRspackPlugin with manual oneOf</summary>

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
        test: /\.vue$/,
        oneOf: [
          // SCSS style blocks
          {
            resourceQuery: /vue&type=style.*lang=scss/,
            type: "javascript/auto",
            use: [
              isProduction ? rspack.CssExtractRspackPlugin.loader : "style-loader",
              "css-loader",
              { loader: "@vizejs/rspack-plugin/scope-loader" },
              "sass-loader",
              { loader: "@vizejs/rspack-plugin/style-loader" },
            ],
          },

          // Regular CSS style blocks
          {
            resourceQuery: /vue&type=style/,
            type: "javascript/auto",
            use: [
              isProduction ? rspack.CssExtractRspackPlugin.loader : "style-loader",
              {
                loader: "css-loader",
                options: {
                  modules: {
                    auto: (_resourcePath, resourceQuery) =>
                      typeof resourceQuery === "string" && resourceQuery.includes("module="),
                  },
                },
              },
              { loader: "@vizejs/rspack-plugin/scope-loader" },
              { loader: "@vizejs/rspack-plugin/style-loader" },
            ],
          },

          // Main .vue file
          {
            use: [
              {
                loader: "@vizejs/rspack-plugin/loader",
              },
            ],
          },
        ],
      },

      // Regular CSS files (non-Vue)
      {
        test: /\.css$/,
        type: "javascript/auto",
        use: [isProduction ? rspack.CssExtractRspackPlugin.loader : "style-loader", "css-loader"],
      },

      // Regular SCSS files (non-Vue)
      {
        test: /\.scss$/,
        type: "javascript/auto",
        use: [
          isProduction ? rspack.CssExtractRspackPlugin.loader : "style-loader",
          "css-loader",
          "sass-loader",
        ],
      },
    ],
  },

  plugins: [
    new VizePlugin({
      autoRules: false,
    }),

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

</details>

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

## API

### VizePlugin

`new VizePlugin()` enables automatic style-rule cloning and TypeScript stripping.
The supported integration options are:

| Option       | Default              | Effect                                         |
| ------------ | -------------------- | ---------------------------------------------- |
| `css.native` | Detected from Rspack | Select Native CSS or a JS loader pipeline      |
| `autoRules`  | `true`               | Clone style rules for SFC sub-requests         |
| `typescript` | `true`               | Inject SWC to strip types from main SFC output |
| `debug`      | `false`              | Emit Vize infrastructure debug messages        |

Enable both Vize debug output and Rspack's logger to display debug messages:

```javascript
{
  infrastructureLogging: { level: "verbose", debug: /VizePlugin/ },
  plugins: [new VizePlugin({ debug: true })],
}
```

Warnings and errors do not depend on `debug`. Deprecated plugin fields remain
accepted with migration warnings; their previous behavior is retained.

### SFC loader

`@vizejs/rspack-plugin/loader` accepts `VizeSfcLoaderOptions`:

```typescript
import type { VizeSfcLoaderOptions } from "@vizejs/rspack-plugin";

const options = {
  ssr: false,
  vapor: false,
  customElement: /\.ce\.vue$/,
  hotReload: true,
  transformAssetUrls: true,
  compilerOptions: { templateSyntax: "standard" },
} satisfies VizeSfcLoaderOptions;
```

| Option               | Behavior                                                                                                                        |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `ssr`                | Server compilation; defaults to false; disables client HMR                                                                      |
| `vapor`              | Default SFC compilation mode; defaults to false                                                                                 |
| `sourceMap`          | Forwards available native maps through output assembly; falls back to `compilerOptions.sourceMap`, then Rspack's loader context |
| `isProduction`       | Override SFC production output; otherwise use loader mode / NODE_ENV                                                            |
| `root`               | Base for scope IDs and development file paths; relative values resolve against Rspack context                                   |
| `compilerOptions`    | Additional native compiler options                                                                                              |
| `css.native`         | Explicit mode for a manually routed SFC chain (`autoRules: false`); otherwise must match the plugin mode                        |
| `customElement`      | Custom element selection; defaults to matching .ce.vue                                                                          |
| `hotReload`          | Enables client HMR injection in development; false disables it                                                                  |
| `transformAssetUrls` | Rewrites supported static template asset URLs; defaults to true                                                                 |

Set `ssr`, `vapor`, and `sourceMap` at loader top level. Their nested
`compilerOptions` counterparts remain as deprecated fallbacks. Explicit top-level
values, including `false`, take precedence. Without an explicit source-map option
or loader context value, maps default to enabled in development and disabled in
production. `filename` and `scopeId` are derived internally.

The SFC loader passes native source maps to Rspack after accounting for generated
imports, export rewrites, component metadata, and template asset replacements.
External script content maps back to its source file. Rspack's `devtool` controls
final bundle map emission. Mapping coverage follows the native compiler: the
current SFC map covers script code; template-only SFCs can return no map. The
loader does not synthesize template mappings.

CSS Modules support Rspack native CSS and `css-loader` with either default or
named exports. Class maps are attached to the component's `$style` or the name
specified by `<style module="name">`.

Legacy loader `include` / `exclude` filters remain available. A filtered file
passes through unchanged with a warning; this is not a Vue 2 fallback. Prefer
Rspack rule conditions so excluded files never reach this loader.
Compilation errors fail the loader immediately.

### JSX loader

`@vizejs/rspack-plugin/jsx-loader` accepts `VizeJsxLoaderOptions`:
`jsxMode`, `jsxCompat`, `vapor`, `sourceMap`, and the legacy file filters.
Configure these on the JSX rule. `jsxCompat: "babel"` selects native compiler
compatibility semantics; it does not install a Babel loader.

The existing `VizeLoaderOptions` type remains exported for compatibility.

#### TypeScript

`@vizejs/native compileSfc` preserves TypeScript syntax in its output (same behavior as `@vue/compiler-sfc`). By default, `VizePlugin` injects a `.vue` `enforce: "post"` rule using Rspack's built-in `builtin:swc-loader` to strip those type annotations for main SFC requests.

- **Default behavior**: No extra config is required. `new VizePlugin()` automatically injects the SWC post-processing rule.
- **Opt out**: Set `new VizePlugin({ typescript: false })` if you want to manage `.vue` TypeScript stripping yourself.
- **Custom loader**: When opting out, use `esbuild-loader`, `builtin:swc-loader`, or any other TS transpiler as your own `enforce: "post"` rule for `.vue` files (excluding `type=style` requests).

<details>
<summary>Advanced: builtin:swc-loader</summary>

```javascript
// rspack.config.mjs
import { defineConfig } from "@rspack/cli";
import { VizePlugin } from "@vizejs/rspack-plugin";

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
      {
        test: /\.vue$/,
        resourceQuery: { not: [/type=/] },
        enforce: "post",
        loader: "builtin:swc-loader",
        options: {
          jsc: {
            parser: {
              syntax: "typescript",
            },
          },
        },
        type: "javascript/auto",
      },
    ],
  },
  plugins: [
    new VizePlugin({
      typescript: false,
    }),
  ],
});
```

</details>

<details>
<summary>Advanced: esbuild-loader</summary>

```javascript
// rspack.config.mjs
import { defineConfig } from "@rspack/cli";
import { VizePlugin } from "@vizejs/rspack-plugin";

export default defineConfig({
  module: {
    rules: [
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
      {
        test: /\.vue$/,
        resourceQuery: { not: [/type=/] },
        enforce: "post",
        loader: "esbuild-loader",
        options: {
          loader: "ts",
          target: "es2020",
        },
        type: "javascript/auto",
      },
    ],
  },
  plugins: [
    new VizePlugin({
      typescript: false,
    }),
  ],
});
```

> Requires `esbuild-loader` and `esbuild` to be installed in your project.

</details>

The `isTs` option is auto-detected from `<script lang="ts">` and passed to the native compiler for correct parsing.

#### `transformAssetUrls`

Static asset URLs in template element attributes (e.g. `<img src="./logo.png">`) are automatically rewritten into JavaScript `import` bindings so that Rspack can process them through its asset pipeline.

| Value                      | Behaviour                                                                                                                   |
| -------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `true` (default)           | Apply built-in transforms: `img[src]`, `video[src,poster]`, `source[src]`, `image[xlink:href,href]`, `use[xlink:href,href]` |
| `false`                    | Disable the feature entirely — URL strings are left as-is                                                                   |
| `Record<string, string[]>` | Custom element/attribute mapping that **replaces** the built-in defaults                                                    |

Only relative (`./`, `../`), alias (`@/`), and tilde (`~/`, `~pkg`) URLs are transformed. External (`https://…`), protocol-relative (`//…`), and data URIs are left unchanged.

```js
// Custom mapping example
{
  loader: "@vizejs/rspack-plugin/loader",
  options: {
    transformAssetUrls: {
      "my-image": ["data-src"],
      img: ["src"],
    },
  },
}
```

> **Known limitations**
>
> - URL rewriting operates via string replacement on the compiled JS output, not on AST nodes. If a `<script>` block contains an identical string literal it will also be replaced (extremely unlikely in practice).
> - URLs with hash fragments (e.g. `./icons.svg#home`) are split: the base path becomes the `import` specifier and the fragment is concatenated at runtime.

### VizeStyleLoader

```typescript
// In rspack.config.js
{
  loader: "@vizejs/rspack-plugin/style-loader",
  options: {
    native: boolean;        // Rspack native CSS mode. Default: true on Rspack 2.x, false on 1.x (1.x also needs experiments: { css: true })
  };
}
```

### VizeScopeLoader

Applies native scoped CSS transformation using `@vizejs/native compileCss`. Runs **after** preprocessors (SCSS/Less/Stylus → CSS) and **before** css-loader or Rspack native CSS handling. For native CSS, Rspack 1.x needs `experiments.css`, while Rspack 2.x enables it by default.

```typescript
// In rspack.config.js
{
  loader: "@vizejs/rspack-plugin/scope-loader",
  // No options — scope metadata is extracted from the query string (?scoped=xxxxx)
}
```

The scope-loader is automatically injected by `VizePlugin` when `autoRules: true` (default). Only needed in manual `oneOf` configurations.

## License

MIT
