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

<p align="center">
  <a href="https://crates.io/crates/vize"><img src="https://img.shields.io/crates/v/vize.svg" alt="crates.io" /></a>
  <a href="https://www.npmjs.com/package/vize"><img src="https://img.shields.io/npm/v/vize.svg?label=vize" alt="npm" /></a>
  <a href="https://www.npmjs.com/package/@vizejs/vite-plugin"><img src="https://img.shields.io/npm/v/@vizejs/vite-plugin.svg?label=@vizejs/vite-plugin" alt="npm" /></a>
  <a href="https://www.npmjs.com/package/@vizejs/wasm"><img src="https://img.shields.io/npm/v/@vizejs/wasm.svg?label=@vizejs/wasm" alt="npm" /></a>
  <a href="https://github.com/ubugeeei/vize/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License" /></a>
</p>

> [!WARNING]
> This project is under active development and is not yet ready for production use.
> APIs and features may change without notice.

> [!NOTE]
> `@vizejs/vite-plugin` is the recommended bundler integration today.
> `@vizejs/unplugin` (rollup / webpack / esbuild) and `@vizejs/rspack-plugin` are available, but non-Vite integrations are still unstable and should be tested carefully before adoption.
> Rspack intentionally keeps a dedicated package because its loader chain, `experiments.css`, and HMR behavior need Rspack-specific handling instead of the shared unplugin path.

---

## Features

- **Compile** — Vue SFC compiler (DOM / Vapor / SSR)
- **Lint** — Vue.js linter with i18n diagnostics
- **Format** — Vue.js formatter
- **Type Check** — TypeScript type checker for Vue
- **LSP** — Language Server Protocol for editor integration
- **Musea** — Component gallery (Storybook-like)
- **MCP** — AI integration via Model Context Protocol

## Quick Start

```bash
npm install -g vize
```

For Vite projects that want one shared config for the plugin and the npm CLI:

```bash
npm install -D vize @vizejs/vite-plugin
```

```bash
vize build src/**/*.vue    # Compile
vize fmt --check           # Format check
vize lint --fix            # Lint & auto-fix
vize build --profile       # Profile parse, transform, codegen, and I/O
vize lint --profile src    # Profile parse, rule hooks, Croquis, and type-aware linting
vize check --profile src   # Profile Virtual TS, Croquis, and Corsa diagnostics
```

`--profile` reports wall/cumulative timings, hot files, and internal operation rows so compiler,
linter, formatter, typecheck, and Croquis costs can be compared from one output.

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
  },
});
```

See the [documentation](https://vizejs.dev) for detailed usage, Vite plugin setup, experimental bundler integrations, WASM bindings, and more.

## Nix Flake

```bash
nix run github:ubugeeei/vize#vp -- --version
nix run github:ubugeeei/vize#vize -- --help
nix profile install github:ubugeeei/vize#vize
```

For local development:

```bash
nix develop
vp env install
vp install
```

## Development Environment

The primary local setup is `Nix + vp`. Nix provides the Rust / WASM toolchain and the `vp` CLI itself, while Node.js version management stays with `vp` and `.node-version`.

```bash
nix develop
vp env install
vp install
```

If you want `node`, `npm`, and related shims to follow the pinned version in your shell, run `vp env setup` once and enable managed mode with `vp env on`.

## Workspace Tasks

Workspace orchestration lives in the root `vite.config.ts` via Vite+'s `run.tasks`.

```bash
vp run --workspace-root check       # packages + examples + playground
vp run --workspace-root check:fix   # auto-fix JS/TS checks where supported
vp run --workspace-root fmt         # format workspace files

vp run --filter './playground' test:browser
vp run --filter './examples/vite-musea' build
```

Use `vp run` directly; `mise` task wrappers have been removed.
`npm/vscode-vize` and `npm/vscode-art` stay outside the root `vp run` graph, so build those from their package directories.

## Performance

Benchmarks with **15,000 Vue SFC files** (36.9 MB). "User-facing speedup" = traditional tool (single-thread) vs Vize (multi-thread).

| Tool             | Traditional (ST)          | Vize (MT)                 | User-facing Speedup |
| ---------------- | ------------------------- | ------------------------- | ------------------- |
| **Compiler**     | @vue/compiler-sfc 9.28s   | 434ms                     | **20.9x**           |
| **Linter**       | eslint-plugin-vue 65.30s  | patina 5.48s              | **11.9x**           |
| **Formatter**    | Prettier 104.08s          | glyph 1.32s               | **78.9x**           |
| **Type Checker** | vue-tsc 22.13s            | canon 3.33s               | **6.6x** \*         |
| **Vite Plugin**  | @vitejs/plugin-vue 16.98s | @vizejs/vite-plugin 6.90s | **2.5x** \*\*       |

<details>
<summary>Detailed compiler benchmark</summary>

|                   | @vue/compiler-sfc | Vize  | Speedup  |
| ----------------- | ----------------- | ----- | -------- |
| **Single Thread** | 9.28s             | 3.30s | **2.8x** |
| **Multi Thread**  | 3.35s             | 434ms | **7.7x** |

</details>

\* canon is still in early development and the Corsa-backed diagnostics path is still catching up with vue-tsc fidelity. These numbers reflect the current native project-session implementation and will keep changing as diagnostics coverage improves.

\*\* Vite Plugin benchmark uses Vite v8.0.0 (Rolldown). The plugin replaces only the SFC compilation step; all other Vite internals are unchanged.

Run `vp run --workspace-root bench:all` to reproduce all benchmarks.

`bench:check` also includes the diagnostics-heavy `npmx.dev` e2e fixture when that fixture is present, so the Corsa diagnostic mapping path stays covered.

## Contributing

See the [documentation](https://vizejs.dev) for architecture overview and development setup.

## Credits

This project is inspired by and builds upon the work of these amazing projects:
[Volar.js](https://github.com/volarjs/volar.js) ・ [vuejs/language-tools](https://github.com/vuejs/language-tools) ・ [eslint-plugin-vue](https://github.com/vuejs/eslint-plugin-vue) ・ [eslint-plugin-vuejs-accessibility](https://github.com/vue-a11y/eslint-plugin-vuejs-accessibility) ・ [Lightning CSS](https://github.com/parcel-bundler/lightningcss) ・ [Storybook](https://github.com/storybookjs/storybook) ・ [OXC](https://github.com/oxc-project/oxc)

## Sponsors

This project is maintained by [@ubugeeei](https://github.com/ubugeeei). If you find Vize useful, please consider [sponsoring](https://github.com/sponsors/ubugeeei).

## License

[MIT](./LICENSE)
