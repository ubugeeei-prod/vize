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
For the full Rust-native CLI (`lsp`, `ide`, project-backed `check`, and `check-server`), install
the Rust `vize` binary with `cargo install vize`.

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
vp exec vize lint src
vp exec vize check
vp exec vize ready src
vp exec vize upgrade
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
scripts after installing `vize`. Add `--declaration --declaration-dir dist/types` to emit Vue
component `.d.ts` files. Use the Rust CLI when you need Corsa project diagnostics across Vue, TS,
TSX, and `.d.ts` inputs.

`vize ready` runs `fmt --write`, `lint`, `check`, and `build` in that order.

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
