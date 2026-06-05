# @vizejs/unplugin

Experimental unplugin-based Vue SFC integration powered by [Vize](https://github.com/ubugeeei-prod/vize).

> [!WARNING]
> `@vizejs/unplugin` is still unstable.
> `@vizejs/vite-plugin` remains the recommended and best-tested bundler integration today.

`@vizejs/unplugin` provides experimental support for:

- `rollup`
- `rolldown`
- `webpack`
- `esbuild`
- `babel`

Rspack intentionally uses the dedicated `@vizejs/rspack-plugin` path instead of an `unplugin` export because its loader chain, `experiments.css`, and HMR behavior need Rspack-specific handling.

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the package:

```bash
vp install @vizejs/unplugin
```

## Usage

### rollup

```javascript
import vize from "@vizejs/unplugin/rollup";

export default {
  plugins: [vize()],
};
```

### webpack

```javascript
import Vize from "@vizejs/unplugin/webpack";

export default {
  plugins: [Vize()],
};
```

Webpack 5 is detected through `compiler.webpack`. Webpack 4 is supported by resolving
`DefinePlugin` from the host project's `webpack` package:

```javascript
import Vize from "@vizejs/unplugin/webpack";

export default {
  plugins: [
    Vize({
      compatibility: {
        webpackVersion: 4,
      },
    }),
  ],
};
```

For Vue 0.11, Vue 1, Vue 2, or Nuxt 2 projects that already use the legacy
Vue/Webpack compiler chain, keep Vize non-invasive and let the host compiler
handle `.vue` files:

```javascript
Vize({
  compatibility: {
    vueVersion: 2,
    webpackVersion: 4,
  },
});
```

### rolldown

```javascript
import vize from "@vizejs/unplugin/rolldown";

export default {
  plugins: [vize()],
};
```

### esbuild

```javascript
import { build } from "esbuild";
import vize from "@vizejs/unplugin/esbuild";

await build({
  entryPoints: ["src/main.ts"],
  bundle: true,
  plugins: [vize()],
});
```

### babel

```javascript
import vize from "@vizejs/unplugin/babel";

export default {
  plugins: [vize()],
};
```

The Babel adapter compiles `.vue` files before Babel parses them. Keep your usual Babel
TypeScript/JSX transforms in the pipeline if your SFC scripts use those syntaxes.

## Caveats

- Vite is still the recommended integration if you need the most complete behavior today.
- CSS Modules and preprocessors depend on the host bundler CSS pipeline and are more likely to change than the Vite path.
- If your bundler inlines the Vue runtime, Vize injects the usual Vue 3 compile-time feature flags for Rollup-like and Webpack hosts when its compiler is active.
- Test carefully before depending on this package in production.
