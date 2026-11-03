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
- Every `<style>` block, including a `<style scoped>` in a `.jsx` or `.tsx` component, is a stylesheet module for the bundler's CSS pipeline, which writes the CSS file, as with `unplugin-vue`. The module is named like Vue's style requests, `App.vue?vue&type=style&index=0&lang.css` (`lang.module.css`, `lang.scss`), so a pipeline that handles a `.css` or `.scss` file handles the block: tsdown with `@tsdown/css` writes `style.css`, esbuild writes `<entry>.css` (set `outdir` or `outfile`), Rollup needs a CSS plugin such as `rollup-plugin-postcss`, and webpack uses your `.css` rule. A build with no CSS pipeline fails on the first style block. Rolldown alone does not bundle CSS, so build through tsdown or Vite.
- `<style src>`, `<script src>` and `<template src>` follow Vue's SFC spec, as in `@vitejs/plugin-vue`: a `./` or `../` path is read next to the SFC, and any other `src` (a package such as `modern-normalize/modern-normalize.css`, an alias, an absolute path, or a bare `foo.css`) is a module request resolved by the bundler's own resolver. The file is inlined before compilation and watched. A `<style src>` block's stylesheet module sits beside its file, so its relative `@import`, `@use` and `url()` resolve from there, as when the file is imported directly. A module the bundler keeps external (tsdown keeps `dependencies` external) stays an import, as in `@vitejs/plugin-vue`: `<style src>` becomes `import "…"` and `<script src>` imports the component from it. `<template src>`, `<script setup src>` and a scoped or module `<style src>` are compiled into the component, so an external one is a build error; let the bundler bundle it. A request that resolves to nothing is a build error.
- Babel and esbuild without `bundle` bundle no stylesheet, so there the compiled CSS is injected into JavaScript at run time, CSS Modules included. A preprocessor block or an `@import` is a build error there, since nothing compiles or resolves it.
- CSS Modules and preprocessors depend on the host bundler CSS pipeline and are more likely to change than the Vite path.
- Scoped preprocessor styles are compiled before they are scoped, as in Vue: Webpack uses your `.scss`/`.less`/`.styl` rules, while Rollup, Rolldown and esbuild run the project's installed `sass` (or `sass-embedded`), `less` or `stylus`. Unscoped preprocessor blocks go to the host's preprocessor (`@tsdown/css`, `rollup-plugin-postcss`, your webpack rules), except under esbuild, whose CSS loaders compile none, so the plugin runs the installed preprocessor there too.
- If your bundler inlines the Vue runtime, Vize injects the usual Vue 3 compile-time feature flags for Rollup-like and Webpack hosts when its compiler is active.
- Test carefully before depending on this package in production.
