# @vizejs/rspack-plugin

High-performance Rspack plugin for Vue SFC compilation powered by [Vize](https://github.com/ubugeeei/vize).

## Features

- ⚡ **Blazing Fast** - Powered by Rust-based `@vizejs/native` compiler
- 🔄 **HMR Support** - Script/template hot reload via `module.hot` + `__VUE_HMR_RUNTIME__`, CSS Modules HMR with targeted rerender
- 🎨 **CSS Processing** - Support for both native CSS (`experiments.css`) and CssExtractRspackPlugin
- 📦 **CSS Modules** - First-class CSS Modules support with per-module HMR
- 🔗 **`<style src>` Support** - Resolves external style files with watch dependency tracking
- 🔧 **TypeScript** - Full TypeScript support with auto-detection and optional built-in TS stripping
- 🗄️ **Compilation Cache** - Content-hash based caching to skip re-compilation of unchanged files
- 🛠️ **Vue DevTools** - Exposes `__file` for component file path in development mode
- 🧩 **Custom Elements** - Auto-detect `.ce.vue` or configure via `customElement` option

## Installation

```bash
pnpm add -D @vizejs/rspack-plugin @rspack/core
```

## Usage

### Auto Preset

Use `createVizeVueRules()` to generate `.vue` rules automatically.
If you don't call it, write rules manually.

```javascript
// rspack.config.mjs
import { rspack } from "@rspack/core";
import { VizePlugin, createVizeVueRules } from "@vizejs/rspack-plugin";

const isProduction = process.env.NODE_ENV === "production";

export default {
  mode: isProduction ? "production" : "development",

  experiments: {
    css: true,
  },

  module: {
    rules: [
      ...createVizeVueRules({
        isProduction,
        nativeCss: true,
        typescript: true, // auto-add SWC post-processing to strip TS types
        styleLanguages: ["scss", "sass", "less", "stylus", "styl"],
        loaderOptions: {
          include: [/src\/.*\.vue$/],
          exclude: [/node_modules/],
          sourceMap: !isProduction,
        },
      }),
    ],
  },

  plugins: [
    new VizePlugin({
      isProduction,
      include: [/src\/.*\.vue$/],
      exclude: [/node_modules/],
      css: { native: true },
    }),
  ],
};
```

### Option A: Native CSS (Recommended)

Uses Rspack's built-in `experiments.css` for optimal performance. CSS is processed by Rust-side LightningCSS.

```javascript
// rspack.config.mjs
import { rspack } from "@rspack/core";
import { VizePlugin } from "@vizejs/rspack-plugin";
import path from "node:path";

const isProduction = process.env.NODE_ENV === "production";

export default {
  mode: isProduction ? "production" : "development",

  // Enable Rspack native CSS support
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
              {
                loader: "@vizejs/rspack-plugin/style-loader",
              },
            ],
          },

          // SCSS (<style lang="scss">)
          {
            resourceQuery: /vue&type=style.*lang=scss/,
            type: "css/auto",
            use: [
              "sass-loader",
              {
                loader: "@vizejs/rspack-plugin/style-loader",
              },
            ],
          },

          // Regular CSS (<style>)
          {
            resourceQuery: /vue&type=style/,
            type: "css/auto",
            use: [
              {
                loader: "@vizejs/rspack-plugin/style-loader",
              },
            ],
          },

          // Main .vue file (compile SFC → JS)
          {
            use: [
              {
                loader: "@vizejs/rspack-plugin/loader",
                options: {
                  include: [/src\/.*\.vue$/],
                  exclude: [/node_modules/],
                  sourceMap: !isProduction,
                },
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
      include: [/src\/.*\.vue$/],
      exclude: [/node_modules/],
      css: {
        native: true,
      },
    }),
  ],

  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "src"),
    },
  },
};
```

### Option B: CssExtractRspackPlugin

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
        test: /\.vue$/,
        oneOf: [
          // SCSS style blocks
          {
            resourceQuery: /vue&type=style.*lang=scss/,
            type: "javascript/auto",
            use: [
              isProduction
                ? rspack.CssExtractRspackPlugin.loader
                : "style-loader",
              "css-loader",
              "sass-loader",
              {
                loader: "@vizejs/rspack-plugin/style-loader",
              },
            ],
          },

          // Regular CSS style blocks
          {
            resourceQuery: /vue&type=style/,
            type: "javascript/auto",
            use: [
              isProduction
                ? rspack.CssExtractRspackPlugin.loader
                : "style-loader",
              {
                loader: "css-loader",
                options: {
                  modules: {
                    auto: (_resourcePath, resourceQuery) =>
                      typeof resourceQuery === "string" &&
                      resourceQuery.includes("module="),
                  },
                },
              },
              {
                loader: "@vizejs/rspack-plugin/style-loader",
              },
            ],
          },

          // Main .vue file
          {
            use: [
              {
                loader: "@vizejs/rspack-plugin/loader",
                options: {
                  include: [/src\/.*\.vue$/],
                  exclude: [/node_modules/],
                  sourceMap: !isProduction,
                },
              },
            ],
          },
        ],
      },

      // Regular CSS files (non-Vue)
      {
        test: /\.css$/,
        type: "javascript/auto",
        use: [
          isProduction ? rspack.CssExtractRspackPlugin.loader : "style-loader",
          "css-loader",
        ],
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
      include: [/src\/.*\.vue$/],
      exclude: [/node_modules/],
    }),

    // CSS extraction (production only)
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
  vapor: boolean;           // Enable Vapor mode (default: false, SFC-level not yet supported)
  root: string;             // Root directory (default: Rspack's root)
  css: {
    native: boolean;        // Use experiments.css (default: false), warns if config mismatch
  };
  compilerOptions: {};      // Extra @vizejs/native compileSfc options
  debug: boolean;           // Enable debug logging (default: false)
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
    customElement: boolean | RegExp; // Custom element mode (default: /\.ce\.vue$/)
    hotReload: boolean;     // Enable HMR (default: true in dev, false in prod/SSR)
    compilerOptions: {      // Extra @vizejs/native compileSfc options
      filename?: string;
      sourceMap?: boolean;
      ssr?: boolean;
      isTs?: boolean;       // Preserve TypeScript (auto-detected from <script lang="ts">)
      scopeId?: string;
    };
  };
}
```

If `include/exclude` filters out a `.vue` file matched by this loader rule, the loader emits a warning and passes through the source unchanged.
This avoids hard failures while still alerting you to mismatched rule/filter configuration.

Compilation errors cause the loader to fail immediately (`callback(error)`) instead of returning broken code.

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

### createVizeVueRules

```typescript
import { createVizeVueRules } from "@vizejs/rspack-plugin";

const rules = createVizeVueRules({
  isProduction: false,
  nativeCss: true, // true: css/auto + css/module, false: css-loader chain
  styleLanguages: ["scss", "sass", "less", "stylus", "styl"],
  styleInjectLoader: "style-loader",
  styleExtractLoader: undefined, // e.g. rspack.CssExtractRspackPlugin.loader
  cssLoader: "css-loader",
  loaderOptions: {
    include: [/src\/.*\.vue$/],
    exclude: [/node_modules/],
    sourceMap: true,
  },
  styleLoaderOptions: {
    native: true, // match nativeCss setting
  },
  typescript: true, // or a custom LoaderEntry
});
```

## Comparison

| Feature            | Native CSS (`experiments.css`) | CssExtractRspackPlugin |
| ------------------ | ------------------------------ | ---------------------- |
| **CSS Engine**     | Rust (LightningCSS)            | JS (css-loader)        |
| **CSS Extraction** | Rspack automatic               | CssExtractRspackPlugin |
| **CSS Modules**    | `type: 'css/module'`           | css-loader config      |
| **HMR**            | Rspack native                  | style-loader           |
| **Vendor Prefix**  | LightningCSS built-in          | Requires autoprefixer  |
| **Performance**    | **Excellent** (Rust)           | Good (JS)              |
| **Use Case**       | **New projects**               | webpack compatibility  |

## Known Limitations

### HMR

Script and template changes trigger a component-level hot reload via `module.hot.accept()` + `__VUE_HMR_RUNTIME__.reload()`. CSS Module changes trigger a targeted rerender without full reload. Plain `<style>` HMR is handled natively by Rspack's CSS pipeline.

### Path Resolution

Style imports injected by the main loader are normalized to resolver-friendly request paths.
In modern Rspack setups this avoids most absolute-path query edge cases, especially across platforms.

### Diagnostics

Compiler diagnostics are emitted via loader APIs:

- Compile errors: `callback(error)` — fails the build immediately
- Compile warnings: `this.emitWarning(...)`
- Missing `<style src>` files: build error (fail fast, no silent style loss)
- Scoped CSS fallback: warning emitted once per file (deduplicated in watch mode)

Scoped preprocessor blocks such as `<style scoped lang="scss">` are currently rejected.
The fallback transformer only understands plain CSS selectors, so allowing SCSS/Less/Stylus here would silently produce incorrect scoped output.

### Scoped CSS

Scoped CSS scope IDs are derived from the file's **relative path** (relative to Rspack's `rootContext`). In production builds, the file content is also mixed into the hash to prevent collisions across packages with identically-named files. This ensures consistent scope IDs across different machines for SSR hydration.

The current scoped CSS implementation uses a fallback regex transformer with the following limitations:

- ❌ No support for `:deep()`, `:global()`, `:slotted()` pseudo-classes
- ❌ No support for nested `@media` / `@supports` selectors
- ❌ May not handle CSS comments containing `{` or `,` correctly

**Recommendation**: This is an MVP implementation. For production-grade scoped CSS, consider waiting for native-side precise API support.

### Source Maps

The current `@vizejs/native` NAPI does not yet return a source map field. The type definition reserves a `map?: string` field for forward-compatibility. Once the Rust side implements source map output, the loader will pass it to Rspack automatically.

### TypeScript Output

`@vizejs/native compileSfc` preserves TypeScript syntax in its output (same behavior as `@vue/compiler-sfc`). A downstream transpiler is needed to strip type annotations:

- **Recommended**: Use `createVizeVueRules({ typescript: true })` to auto-add a `builtin:swc-loader` post-processing rule
- **Custom loader**: Pass a `LoaderEntry` — e.g. `typescript: "esbuild-loader"` or `typescript: { loader: "esbuild-loader", options: { ... } }`
- **Manual**: Add your own `enforce: "post"` rule for `.vue` files (exclude `type=style` requests)

The `isTs` option is auto-detected from `<script lang="ts">` and passed to the native compiler for correct parsing.

## License

MIT
