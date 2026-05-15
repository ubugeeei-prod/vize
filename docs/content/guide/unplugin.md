---
title: Experimental Bundler Integrations
---

# Experimental Bundler Integrations

> **⚠️ Experimental:** `@vizejs/unplugin` and `@vizejs/rspack-plugin` are still unstable.
> `@vizejs/vite-plugin` remains the recommended and best-tested bundler integration today.

Vize provides an experimental [unplugin](https://unplugin.unjs.io/) package for `rollup`, `webpack`, and `esbuild`, plus a dedicated `Rspack` package:

- `@vizejs/unplugin` — `rollup` / `webpack` / `esbuild`
- `@vizejs/rspack-plugin` — `Rspack` only

Rspack intentionally does **not** go through the shared unplugin path.
Its loader chain, `experiments.css`, and HMR behavior need Rspack-specific handling.

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the packages:

```bash
vp install @vizejs/unplugin
```

For Rspack:

```bash
vp install -D @vizejs/rspack-plugin @rspack/core
```

## rollup

```javascript
// rollup.config.mjs
import vize from "@vizejs/unplugin/rollup";

export default {
  plugins: [vize()],
};
```

## webpack

```javascript
// webpack.config.mjs
import Vize from "@vizejs/unplugin/webpack";

export default {
  plugins: [Vize()],
};
```

## esbuild

```javascript
// build.mjs
import { build } from "esbuild";
import vize from "@vizejs/unplugin/esbuild";

await build({
  entryPoints: ["src/main.ts"],
  bundle: true,
  plugins: [vize()],
});
```

## Rspack

Use the dedicated `@vizejs/rspack-plugin` package instead of `@vizejs/unplugin`:

```javascript
// rspack.config.mjs
import { VizePlugin } from "@vizejs/rspack-plugin";

export default {
  experiments: {
    css: true,
  },
  module: {
    rules: [
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
    ],
  },
  plugins: [new VizePlugin()],
};
```

See the package README for the full Rspack configuration surface.

## Caveats

- Vite is still the recommended integration if you need the most complete and best-tested behavior.
- CSS Modules and style preprocessors outside Vite depend on the host bundler's CSS pipeline and are more likely to change.
- If your bundler inlines the Vue runtime instead of externalizing it, make sure the usual Vue compile-time feature flags are configured for that bundler.
- Treat these integrations as experimental and validate them against your own application before rollout.
