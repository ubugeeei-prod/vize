# API reference

[Package overview](../README.md) · [Manual rules](./manual-rules.md)


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
