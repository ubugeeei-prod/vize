# Vize

The `vize` npm package provides:

- shared config utilities (`defineConfig`, `loadConfig`)
- the native `vize lint` command
- the native `vize check` command for package scripts

For Vite integration, pair it with `@vizejs/vite-plugin`.
For the full Rust-native CLI (`build`, `fmt`, project-backed `check`, `lsp`, `ide`), install the
Rust `vize` binary with `cargo install vize`.

Need `vp` first? Install Vite+ once from the [Vite+ install guide](https://viteplus.dev/guide/install).

## Installation

```bash
vp install -D vize
```

## CLI

The npm CLI exposes `lint` and `check`:

```bash
vp exec vize lint src
vp exec vize lint --preset opinionated --help-level short src
vp exec vize check
vp exec vize check src
```

Shared config discovery is supported for the npm CLI:

- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.pkl`
- `vize.config.json`

```ts
import { defineConfig } from "vize";

export default defineConfig({
  linter: {
    preset: "opinionated",
  },
  typeChecker: {
    strict: true,
  },
});
```

Override config discovery with `--config`, or disable it with `--no-config`.

`vize check` in the npm package uses the packaged NAPI checker so it can run from `package.json`
scripts after installing `vize`. Use the Rust CLI when you need Corsa project diagnostics across
Vue, TS, TSX, and `.d.ts` inputs.

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
