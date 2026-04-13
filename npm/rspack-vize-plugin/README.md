# @vizejs/rspack-plugin

High-performance Rspack plugin for Vue SFC compilation powered by [Vize](https://github.com/ubugeeei/vize).

> [!NOTE]
> Rspack intentionally uses the dedicated `@vizejs/rspack-plugin` path instead of an `@vizejs/unplugin/rspack` export.
> Its loader chain, `experiments.css`, and HMR behavior need Rspack-specific handling.
>
> Non-Vite bundler integrations are still unstable.
> If you need rollup, webpack, or esbuild, use `@vizejs/unplugin` and test carefully before relying on it in production.

## Features

- ⚡ **Blazing Fast** - Powered by Rust-based `@vizejs/native` compiler
- 🔄 **HMR Support** - Script/template hot reload via `module.hot` + `__VUE_HMR_RUNTIME__`, CSS Modules HMR with targeted rerender
- 🎨 **CSS Processing** - Support for both native CSS (`experiments.css`) and CssExtractRspackPlugin
- 📦 **CSS Modules** - First-class CSS Modules support with per-module HMR
- 🔗 **`<style src>` Support** - Resolves external style files with watch dependency tracking
- 🔧 **TypeScript** - Full TypeScript support with auto-detection and built-in SWC stripping by default
- 🗄️ **Compilation Cache** - Content-hash based caching to skip re-compilation of unchanged files
- 🛠️ **Vue DevTools** - Exposes `__file` for component file path in development mode
- 🧩 **Custom Elements** - Auto-detect `.ce.vue` or configure via `customElement` option

## Installation

```bash
pnpm add -D @vizejs/rspack-plugin @rspack/core
```

## Usage

### Simple Mode (Recommended)

Write a single `.vue` rule and your normal CSS rules. `VizePlugin` automatically clones your CSS rules for Vue style sub-requests and injects Rspack's built-in SWC post-processing for `.vue` TypeScript output.

```javascript
// rspack.config.mjs
import { VizePlugin } from "@vizejs/rspack-plugin";

const isProduction = process.env.NODE_ENV === "production";

export default {
  mode: isProduction ? "production" : "development",

  experiments: {
    css: true, // Enable Rspack native CSS
  },

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
      isProduction,
      css: { native: true },
    }),
  ],
};
```

### Native CSS with SCSS (Simple Mode)

Uses Rspack's built-in `experiments.css` for optimal performance. Just add your SCSS rule — VizePlugin handles the rest.

```javascript
// rspack.config.mjs
import { VizePlugin } from "@vizejs/rspack-plugin";
import path from "node:path";

const isProduction = process.env.NODE_ENV === "production";

export default {
  mode: isProduction ? "production" : "development",

  experiments: {
    css: true,
  },

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

  plugins: [
    new VizePlugin({
      isProduction,
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
    new VizePlugin({
      isProduction,
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

  experiments: {
    css: true,
  },

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
      isProduction,
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
      isProduction,
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

## API

### VizePlugin

```typescript
import { VizePlugin } from "@vizejs/rspack-plugin";

new VizePlugin({
  isProduction: boolean;    // Auto-detected from Rspack mode
  include: string | RegExp | (string | RegExp)[]; // Filter watched .vue files
  exclude: string | RegExp | (string | RegExp)[]; // Exclude watched .vue files
  ssr: boolean;             // Enable SSR mode (default: false)
  sourceMap: boolean;       // Enable source maps (default: true in dev)
  vapor: boolean;           // Enable Vapor mode (default: false)
  root: string;             // Root directory (default: Rspack's root)
  css: {
    native: boolean;        // Use experiments.css (default: false), warns if config mismatch
  };
  compilerOptions: {};      // Extra @vizejs/native compileSfc options
  debug: boolean;           // Enable debug logging (default: false)
  autoRules: boolean;       // Auto-clone CSS rules for Vue style sub-requests (default: true)
  typescript: boolean;      // Auto-inject builtin:swc-loader for .vue post-processing (default: true)
});
// Debug logging uses Rspack's infrastructure logger.
// Control verbosity via `infrastructureLogging.level` in your rspack config.
```

### VizeLoader

```typescript
// In rspack.config.js
{
  loader: "@vizejs/rspack-plugin/loader",
  options: {
    include: string | RegExp | (string | RegExp)[]; // Safe compile allowlist
    exclude: string | RegExp | (string | RegExp)[]; // Safe compile denylist
    sourceMap: boolean;     // Enable source maps (default: true)
    ssr: boolean;           // Enable SSR mode (default: false)
    vapor: boolean;         // Enable Vapor mode (default: false)
    customElement: boolean | RegExp; // Custom element mode (default: /\.ce\.vue$/)
    hotReload: boolean;     // Enable HMR (default: true in dev, false in prod/SSR)
    transformAssetUrls: boolean | Record<string, string[]>; // See below (default: true)
    compilerOptions: {      // Extra @vizejs/native compileSfc options
      filename?: string;
      sourceMap?: boolean;
      ssr?: boolean;
      isTs?: boolean;       // Preserve TypeScript (auto-detected from <script lang="ts">)
      vapor?: boolean;      // Enable Vapor mode compilation
      scopeId?: string;
    };
  };
}
```

If `include/exclude` filters out a `.vue` file matched by this loader rule, the loader emits a warning and passes through the source unchanged.
This avoids hard failures while still alerting you to mismatched rule/filter configuration.

Compilation errors cause the loader to fail immediately (`callback(error)`) instead of returning broken code.

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
    native: boolean;        // Using experiments.css (default: false)
  };
}
```

### VizeScopeLoader

Applies native scoped CSS transformation using `@vizejs/native compileCss`. Runs **after** preprocessors (SCSS/Less/Stylus → CSS) and **before** css-loader or `experiments.css`.

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
