# Vize through JSR

`@vizejs/vize` exposes Vize's JavaScript APIs through JSR. Every entry point
re-exports the npm package of the same version, so JSR and npm users run the
same code and the same native binaries.

| Import                | API                                                  | npm package           |
| --------------------- | ---------------------------------------------------- | --------------------- |
| `@vizejs/vize`        | `defineConfig` and the configuration types           | `vize`                |
| `@vizejs/vize/config` | Configuration loading and normalization              | `vize/config`         |
| `@vizejs/vize/native` | Native compiler, linter, formatter and analysis APIs | `@vizejs/native`      |
| `@vizejs/vize/vite`   | The Vite plugin (default export) and its options     | `@vizejs/vite-plugin` |

## Install

```sh
npx jsr add @vizejs/vize    # Node.js 22 or newer
bunx jsr add @vizejs/vize   # Bun
deno add jsr:@vizejs/vize   # Deno 2
```

```ts
import { defineConfig } from "@vizejs/vize";
import { compileSfc } from "@vizejs/vize/native";
import vize from "@vizejs/vize/vite";

export default defineConfig({ compiler: { vapor: false } });

const result = compileSfc("<template><h1>Hello</h1></template>", {
  filename: "Hello.vue",
});
const plugin = vize();
```

## Requirements

- The native binaries are installed from npm as optional dependencies, so the
  install must reach `registry.npmjs.org` and keep optional dependencies on.
  Deno loads them through Node-API and needs `--allow-ffi`.
- Supported native targets are macOS x64/arm64, Windows x64/arm64, and Linux
  x64/arm64 with glibc or musl. Other targets have no JavaScript fallback.
- `@vizejs/vize/vite` needs `vite` and `vue` in the project.
- The package has no CLI. Install [`vize` from npm](https://www.npmjs.com/package/vize)
  for the `vize` command. For browsers, use `@vizejs/wasm` from npm.
