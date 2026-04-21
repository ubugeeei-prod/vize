<p align="center">
  <img src="./playground/public/og-image.png" alt="Vize" width="600" />
</p>

<p align="center">
  <strong>Unofficial High-Performance Vue.js Toolchain in Rust</strong>
</p>

<p align="center">
  <em>/viːz/ — Named after Vizier + Visor + Advisor: a wise tool that sees through your code.</em>
</p>

<p align="center">
  <a href="https://vizejs.dev"><strong>Documentation</strong></a> ・
  <a href="https://vizejs.dev/play/"><strong>Playground</strong></a> ・
  <a href="https://github.com/sponsors/ubugeeei"><strong>Sponsor</strong></a>
</p>

> [!WARNING]
> Vize is under active development. APIs, package boundaries, and editor features are still moving.

> [!IMPORTANT]
> For day-to-day editor support, keep using the official Vue language tools (`vuejs/language-tools`) for now.
> Vize's VS Code extension, Zed extension, and `vize lsp` default to opt-in capabilities so teams can adopt them gradually.

## What Ships Today

- Rust workspace crates for parsing, semantic analysis, compilation, linting, formatting, type checking, LSP, Musea art tooling, and bindings
- A full Rust CLI via the `vize` crate (`build`, `fmt`, `lint`, `check`, `musea`, `lsp`, `ide`)
- npm packages including `@vizejs/vite-plugin`, `@vizejs/native`, `@vizejs/wasm`, `@vizejs/unplugin`, `@vizejs/rspack-plugin`, `@vizejs/nuxt`, `@vizejs/vite-plugin-musea`, `@vizejs/musea-mcp-server`, and `oxlint-plugin-vize`
- The `vize` npm package for shared config utilities and the native `lint` command

## Quick Start

### Vite

```bash
pnpm add -D vize @vizejs/vite-plugin
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  linter: {
    preset: "opinionated",
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
});
```

### npm CLI

The npm `vize` package currently exposes the native lint command plus shared config helpers:

```bash
pnpm add -D vize
pnpm exec vize lint src
pnpm exec vize lint --preset opinionated --help-level short src
```

### Full Rust CLI

For the full native CLI, install the Rust binary:

```bash
cargo install vize
```

```bash
vize build src/**/*.vue
vize fmt --check src
vize lint --profile src
vize check --profile src
vize lsp
```

You can also run the current workspace build directly:

```bash
nix run github:ubugeeei/vize#vize -- --help
```

## Oxlint Integration

`oxlint-plugin-vize` lets Oxlint execute Vize Patina diagnostics through Oxlint's JS plugin system.

```bash
pnpm add -D oxlint oxlint-plugin-vize
pnpm exec oxlint-vize -c .oxlintrc.json -f stylish src
```

This keeps Oxlint's core JS and TS rules active while adding Vue-aware diagnostics under the `vize/*` namespace.

## Editor Integration

Vize editor support is designed for incremental adoption alongside `vuejs/language-tools`.

Start with lint-only mode in VS Code:

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

Zed can enable the same capabilities through LSP initialization options:

```json
{
  "languages": {
    "Vue": {
      "language_servers": ["vize", "..."]
    }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true
      }
    }
  }
}
```

The same feature names can be committed in `vize.config.json` or `vize.config.pkl` under `lsp`.

## Local Development

The primary local setup is `Nix + pnpm + vp`.

```bash
nix develop
pnpm install --frozen-lockfile
vp build
vp run --workspace-root check
```

Useful workspace tasks:

```bash
vp build
vp run --workspace-root check
vp run --workspace-root check:fix
vp run --workspace-root fmt
vp run --workspace-root bench:all
```

## Credits

This project draws inspiration from:
[Volar.js](https://github.com/volarjs/volar.js) ・
[vuejs/language-tools](https://github.com/vuejs/language-tools) ・
[eslint-plugin-vue](https://github.com/vuejs/eslint-plugin-vue) ・
[eslint-plugin-vuejs-accessibility](https://github.com/vue-a11y/eslint-plugin-vuejs-accessibility) ・
[Lightning CSS](https://github.com/parcel-bundler/lightningcss) ・
[Storybook](https://github.com/storybookjs/storybook) ・
[OXC](https://github.com/oxc-project/oxc)

## Sponsors

Vize is maintained by [@ubugeeei](https://github.com/ubugeeei). If you find it useful, please consider
[sponsoring](https://github.com/sponsors/ubugeeei).

## License

[MIT](./LICENSE)
