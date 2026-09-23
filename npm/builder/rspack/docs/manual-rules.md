# Manual SFC rule routing

[Package overview](../README.md) · [API reference](./api.md)

## Manual `oneOf` rules

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
      css: { native: false },
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
