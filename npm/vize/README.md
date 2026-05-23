# Vize

The `vize` npm package provides:

- shared config utilities (`defineConfig`, `loadConfig`)
- the native `vize build` command
- the native `vize fmt` command
- the native `vize lint` command
- the native `vize check` command for package scripts
- `vize ready` for `fmt --write -> lint -> check -> build`
- `vize upgrade` for updating the npm package

For Vite integration, pair it with `@vizejs/vite-plugin`.
For the full Rust-native CLI (`lsp`, `ide`, project-backed `check`, and `check-server`), use the
GitHub release binaries or the Nix entry point. The Rust CLI is not published through crates.io for
v1 alpha.

Need `vp` first? Install Vite+ once from the [Vite+ install guide](https://viteplus.dev/guide/install).

## Installation

```bash
vp install -D vize
```

## CLI

The npm CLI exposes the common package-script commands:

```bash
vp exec vize build src
vp exec vize fmt --write src
vp exec vize lint --preset happy-path src
vp exec vize check src
vp exec vize ready src
vp exec vize upgrade
```

Recommended scripts:

```json
{
  "scripts": {
    "vue:build": "vize build src",
    "vue:fmt": "vize fmt --write src",
    "vue:lint": "vize lint --preset happy-path src",
    "vue:check": "vize check src",
    "vue:ready": "vize ready src"
  }
}
```

Shared config discovery is supported for the npm CLI:

- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.pkl`
- `vize.config.json`

Pkl config files require either `@pkl-community/pkl` installed in the project or a `pkl` binary on
`PATH`. The Pkl runtime is optional so packages that only consume Vize through framework plugins do
not install it by default.

```ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
  linter: {
    preset: "opinionated",
  },
  typeChecker: {
    enabled: true,
    strict: true,
  },
});
```

Override config discovery with `--config`, or disable it with `--no-config`.

## Static Analysis

`vize lint` runs Vue-aware Patina diagnostics through the native binding:

```bash
vp exec vize lint --preset essential --max-warnings 0 src
vp exec vize lint --preset ecosystem src
vp exec vize lint --preset opinionated --help-level short src
vp exec vize lint --format json src
vp exec vize lint --format plain src
vp exec vize lint --format agent src
```

Lint output supports `text`, `ansi`, `plain`, `json`, `stylish`, `markdown`, `html`, and `agent`.
The human and agent-friendly formats include local rule documentation paths such as
`docs/content/rules/vue.md`.

`vize check` in the npm package uses the packaged NAPI checker so it can run from `package.json`
scripts after installing `vize`:

```bash
vp exec vize check src --strict
vp exec vize check src --show-virtual-ts
vp exec vize check src --declaration --declaration-dir dist/types
```

Use the Rust CLI when you need Corsa project diagnostics across Vue, TS, TSX, and `.d.ts` inputs.

`vize ready` runs `fmt --write`, `lint`, `check`, and `build` in that order.

## Compiler and Tool Options

Important shared fields:

| Field                     | Used by                | Purpose                                                 |
| ------------------------- | ---------------------- | ------------------------------------------------------- |
| `compiler.sourceMap`      | Vite plugin            | Enable source maps                                      |
| `compiler.ssr`            | npm build, Vite plugin | Force SSR compilation                                   |
| `compiler.vapor`          | npm build, Vite plugin | Enable Vapor compilation                                |
| `compiler.customRenderer` | npm build, Vite plugin | Support custom renderer element semantics               |
| `compiler.scriptExt`      | npm build              | Preserve TypeScript output or downcompile to JavaScript |
| `vite.scanPatterns`       | Vite plugin            | Pre-compile matching Vue files                          |
| `linter.preset`           | npm lint               | Select the Patina lint preset                           |
| `typeChecker.strict`      | npm check              | Enable strict checks                                    |
| `formatter.printWidth`    | npm fmt                | Set formatting width                                    |

## Programmatic Config Helpers

```ts
import { defineConfig, loadConfig } from "vize";

export default defineConfig({
  linter: {
    preset: "happy-path",
  },
});

const config = await loadConfig(process.cwd());
```

## Related Packages

- `@vizejs/vite-plugin`
- `@vizejs/native`
- `@vizejs/wasm`
- `@vizejs/nuxt`
- `@vizejs/vite-plugin-musea`

## License

MIT
