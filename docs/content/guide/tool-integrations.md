---
title: Existing Tool Integrations
description: Use Vize through Parcel, ESLint, Prettier, Babel, tsserver, or any LSP editor without replacing the rest of your toolchain.
---

# Existing Tool Integrations

Vize can run inside a tool you already use. Most projects should choose one integration first; you
do not need every package on this page. Keep one owner for each job so the same lint diagnostic,
format edit, or editor feature is not produced twice.

## Choose one starting point

| Keep this workflow                  | Add this Vize surface                                                  | Availability                 |
| ----------------------------------- | ---------------------------------------------------------------------- | ---------------------------- |
| Parcel 2 builds                     | [`@vizejs/parcel-transformer`](#parcel)                                | Experimental; workspace only |
| `eslint`                            | [`eslint-plugin-vize`](#eslint-flat-config)                            | Experimental; workspace only |
| `prettier`                          | [`prettier-plugin-vize`](#prettier)                                    | Experimental; workspace only |
| Babel for JSX or TSX                | [`@vizejs/babel-plugin`](#babel)                                       | Compatibility preview on npm |
| TypeScript 5.7–6.x language service | [`@vizejs/typescript-vue-plugin`](#typescript-language-service-plugin) | Experimental; workspace only |
| Any editor with an LSP client       | [`vize lsp`](#editors-other-than-vs-code)                              | Available in the `vize` CLI  |

Workspace-only packages are implemented and tested in this repository but cannot yet be installed
from the npm registry. Their examples document the intended configuration for evaluation from a
checkout and for their first public release. See [package support tiers](../stability.md#package-support-tiers)
before adopting a preview integration.

## Parcel

Use `@vizejs/parcel-transformer` when Parcel 2 owns the application build. This integration
compiles complete Vue SFCs; it is separate from the Babel JSX/TSX plugin and does not start the Vize
type checker. It currently supports Parcel `^2.16.4`. The package does not have a supported npm
release yet, but its intended installation after the first public release is:

```bash
npm install --save-dev parcel @vizejs/parcel-transformer
npm install vue
```

Register the SFC transformer, its named component pipeline, and the CSS continuation in
`.parcelrc`:

```json
{
  "extends": "@parcel/config-default",
  "reporters": ["./vize-parcel-dev-cache.cjs", "..."],
  "transformers": {
    "*.vue": ["@vizejs/parcel-transformer"],
    "vize-component:*.vue": ["@vizejs/parcel-transformer"],
    "*.css": ["@vizejs/parcel-transformer/style", "..."]
  }
}
```

Create `vize-parcel-dev-cache.cjs` beside `.parcelrc`:

```js
module.exports = require("@vizejs/parcel-transformer/dev-cache");
```

Parcel validates reporter package names before resolving package exports, so the relative
CommonJS adapter loads the public reporter subpath. Keep it before `"..."`: during development it
prevents a rapid compile-error recovery from reusing a stale bundle timestamp.

Configure compiler controls in `.vizerc.parcel.json`, or use the
`"@vizejs/parcel-transformer"` key in `package.json`:

```json
{
  "compiler": {
    "customElements": ["my-element"],
    "templateSyntax": "standard"
  },
  "transformAssetUrls": true
}
```

The transformer supports inline JavaScript and TypeScript, template asset URLs, scoped CSS,
external style blocks, host preprocessors, source maps, browser builds, and native SSR builds.
Install the corresponding Parcel transformer and preprocessor when using Sass, Less, or Stylus;
Vize does not download those providers. Parcel supplies filenames, production mode, targets,
source-map settings, and scope identity, so those values are not Vize options.

Vapor output is opt-in with `"compiler": { "vapor": true }` and requires a compatible Vue runtime.
SFC CSS modules, external script or template blocks, custom blocks, and emitted macro artifacts are
currently rejected explicitly. Use Parcel's normal support for separate `.module.css` imports.
Development HMR conservatively reloads a changed component, so edits to an inline SFC can reset
local component state.

## ESLint flat config

Use the flat `eslint.config.mjs` format. The recommended preset includes the Vue and TypeScript
parsers and applies Vize's native per-file rules to `*.vue` files.

```js
// eslint.config.mjs
import vize from "eslint-plugin-vize";

export default [vize.configs.recommended];
```

Select a smaller preset or add individual rules in the same flat config:

```js
// eslint.config.mjs
import { createVizeConfig } from "eslint-plugin-vize";

export default [
  createVizeConfig({ presets: ["essential"] }),
  {
    files: ["**/*.vue"],
    rules: { "vize/a11y/alt-text": "error" },
  },
];
```

This adapter runs per-file rules through ESLint. Use `vize lint` or the Vite+ lint task when you
need Vize's project graph and cross-file rules. Do not run the same Vize rule through both hosts in
one CI job.

## Prettier

Keep Prettier's file discovery, ignore file, editor integration, and configuration. Add the Vize
plugin to format Vue SFCs and Musea `.art` files with Glyph.

```json
{
  "plugins": ["prettier-plugin-vize"],
  "printWidth": 100,
  "singleQuote": true
}
```

```bash
prettier --write "src/**/*.{vue,art}"
prettier --check "src/**/*.{vue,art}"
```

Use either this plugin or `vize fmt` as the formatter for a file. The plugin follows Prettier's
configuration and `.prettierignore`; it does not read Vize project configuration.

## Babel

Use the Babel plugin when Babel already owns JSX or TSX compilation. It is independent of the Vite
SFC plugin.

```bash
npm install --save-dev @babel/core @vizejs/babel-plugin
```

```js
// babel.config.mjs
export default {
  plugins: [["@vizejs/babel-plugin", { jsxMode: "vdom" }]],
  sourceMaps: true,
};
```

Use `@babel/preset-typescript` when Babel must strip types. Set `jsxMode: "vapor"` only when the app
uses the matching Vue Vapor runtime. See [Babel JSX compatibility](./jsx-babel-compat.md) for the
semantic differences and the package README for SSR and extracted-style options.

## TypeScript language service plugin

`@vizejs/typescript-vue-plugin` adds Vue projections to the JavaScript TypeScript language server
for TypeScript 5.7–6.x. It does not start a second language server and is not tied to VS Code: any
tsserver client that loads project plugins can use the same `tsconfig.json` entry.

```json
{
  "compilerOptions": {
    "strict": true,
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "plugins": [{ "name": "@vizejs/typescript-vue-plugin" }]
  },
  "include": ["src"]
}
```

Restart tsserver after changing the plugin entry. VS Code must use the workspace TypeScript
version; other clients must likewise start the project's TypeScript server. Ordinary `tsc` does not
load language-service plugins. For TypeScript 7 `tsc`, tsgo, or tsrs, use the
[content mapper](./content-mapper.md) instead.

## Editors other than VS Code

The VS Code extension is a convenience, not a requirement. `vize lsp` speaks standard Language
Server Protocol over stdio, so Zed, Neovim, Vim, Helix, Emacs, and other LSP clients can run it.

```bash
npm install --global vize
vize lsp
```

The [language server guide](./language-server.md) has ready-to-use configuration for those editors
and explains the `recommended`, `lint`, and `off` profiles. Start with `lint` when another Vue
language server already owns type checking, completion, and navigation.

## Avoid duplicate owners

- Use the Parcel transformer when Parcel owns the SFC build. Do not send the same Vue entry through
  both Parcel and Vite.
- Choose `eslint-plugin-vize` or the native Vize lint command for the same Vize rule set.
- Choose `prettier-plugin-vize` or `vize fmt` for the same files.
- Choose one primary Vue language server. Use the lint-only profile when keeping Vue - Official.
- Add the Babel plugin only when Babel owns JSX/TSX compilation; it does not replace SFC builds,
  linting, formatting, or editor support.
