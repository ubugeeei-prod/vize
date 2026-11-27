# @vizejs/swc-plugin

Compile Vue JSX/TSX with Vize inside SWC, and Vue single-file components with the
SFC API. Both routes use the shared Vize compiler. SWC owns target lowering,
module conversion and final source-map generation.

```sh
pnpm add -D @swc/core @vizejs/swc-plugin
pnpm add vue
```

This compatibility preview is tested with `@swc/core` 1.16.22. The package root
resolves to a Wasm binary, as required by SWC's `jsc.experimental.plugins` API.
The SFC API is exported separately from `@vizejs/swc-plugin/sfc`.

## JSX and TSX

```json
{
  "jsc": {
    "parser": { "syntax": "typescript", "tsx": true },
    "target": "es2022",
    "experimental": {
      "runPluginFirst": true,
      "plugins": [["@vizejs/swc-plugin", { "jsxMode": "vdom" }]]
    }
  },
  "sourceMaps": true
}
```

For JavaScript JSX, select `{ "syntax": "ecmascript", "jsx": true }` instead.
Keep `runPluginFirst: true`: Vize needs component prop types before SWC erases
TypeScript. The normal SWC CLI, `@swc/core` API and loaders that support Wasm
plugins can consume this configuration. Raw `.vue` files require the SFC API
below because SWC's parser runs before Wasm plugins.

| Option      | Default    | Meaning                                                       |
| ----------- | ---------- | ------------------------------------------------------------- |
| `jsxMode`   | `"vdom"`   | `"vdom"` or `"vapor"`; component mode directives override it. |
| `jsxCompat` | `"native"` | `"native"` or Vue `"babel"` JSX semantics.                    |
| `ssr`       | `false`    | Generate SSR output for the selected renderer.                |

Vize's JSX authoring rules and renderer limitations apply. For SSR or Vapor,
use a block setup component, for example
`const App = (props: { label: string }) => { return <p>{props.label}</p>; }`.
Vapor requires a Vue build with the matching Vapor APIs (currently the Vize
preview targets Vue 3.6.0-rc.11). Babel compatibility cannot be combined with
Vapor. The application supplies the Vue runtime and, for SSR, `@vue/server-renderer`.

Scoped JSX style blocks fail explicitly: SWC Wasm plugins have no stylesheet
emission channel. Use a Vize bundler integration for extracted JSX CSS.
Transforms that introduce JSX must run before this plugin. Files without JSX
pass through unchanged. The adapter prints the current AST, compiles it once,
rebuilds SWC binding contexts and maps native output origins back to the host
source. Authored comments travel through the same pipeline.

## Vue single-file components

```js
import { readFile } from "node:fs/promises";
import { transformVue } from "@vizejs/swc-plugin/sfc";

const result = await transformVue(await readFile("Card.vue", "utf8"), {
  filename: "Card.vue",
  sourceMaps: true,
  jsc: { target: "es2022" },
  module: { type: "es6" },
  vize: { mode: "vdom", styles: "inject" },
});
// result.code / result.map: SWC output
// result.vize: { css, scopeId, warnings, dependencies }
```

`transformVueSync` accepts the same options. Both ESM and CommonJS entrypoints
are provided. Pass SWC options directly: implicit `.swcrc`, `configFile` and
upstream `inputSourceMap` are rejected. Vize supplies the generated TypeScript
parser input and its map to the original `.vue` source.
Static or macro-only modules without authored runtime origins have an empty
mapping; generated helpers and style bootstrap code remain unmapped.

| `vize` option  | Default                     | Meaning                                                                                                        |
| -------------- | --------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `mode`         | `"vdom"`                    | `"vdom"` or `"vapor"`.                                                                                         |
| `ssr`          | `false`                     | Generate server output; never insert CSS into the DOM.                                                         |
| `styles`       | `"inject"`                  | Inject plain/scoped CSS once in the browser, return `"metadata"` for a CSS consumer, or reject with `"error"`. |
| `root`         | `process.cwd()`             | Root used to derive stable scope IDs.                                                                          |
| `isProduction` | `NODE_ENV === "production"` | Production compiler settings.                                                                                  |

Use identical root, filename and production settings for client and server
compilation. CSS remains in `result.vize.css` in every style mode. A metadata
consumer must deliver it to the page. External script/template/style blocks,
CSS modules, CSS `@import`, style preprocessors, custom blocks and emitted macro
artifacts require a bundler adapter and fail explicitly here. This standalone
API does not provide bundler module resolution or hot module replacement.

## Building from source

Use the repository's pinned Rust and Node versions, install the
`wasm32-wasip1` Rust target, then run:

```sh
vp install --frozen-lockfile
vp run --filter './npm/builder/swc' build
vp run --filter './npm/native' build:debug
vp run --filter './npm/builder/swc' test
```

The normal release workflow builds both entrypoints and publishes through npm
trusted publishing (`ubugeeei-prod/vize`, `release.yml`, environment `npm`).
