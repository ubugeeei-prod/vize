# Vize

The `vize` npm package provides:

- shared config utilities (`defineConfig`, `loadConfig`)
- the native `vize lint` command

For Vite integration, pair it with `@vizejs/vite-plugin`.
For the full Rust-native CLI (`build`, `fmt`, `check`, `lsp`, `ide`), install the Rust `vize`
binary with `cargo install vize`.

## Installation

```bash
pnpm add -D vize
```

## CLI

The npm CLI currently exposes `lint`:

```bash
pnpm exec vize lint src
pnpm exec vize lint --preset opinionated --help-level short src
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
});
```

Override config discovery with `--config`, or disable it with `--no-config`.

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
