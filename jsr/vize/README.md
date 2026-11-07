# Vize through JSR

`@vizejs/vize` exposes Vize's JavaScript APIs through JSR. Every entry point
re-exports the npm package of the same version, so JSR and npm users run the
same code and the same native binaries.

| Import                | API                                                  | npm package            |
| --------------------- | ---------------------------------------------------- | ---------------------- |
| `@vizejs/vize`        | `defineConfig` and the configuration types           | `vize`                 |
| `@vizejs/vize/babel`  | Babel plugin for Vue SFC and JSX transforms          | `@vizejs/babel-plugin` |
| `@vizejs/vize/config` | Configuration loading and normalization              | `vize/config`          |
| `@vizejs/vize/native` | Native compiler, linter, formatter and analysis APIs | `@vizejs/native`       |
| `@vizejs/vize/oxlint` | Oxlint plugin, presets and Vite+ config helpers      | `oxlint-plugin-vize`   |
| `@vizejs/vize/vite`   | The Vite plugin (default export) and its options     | `@vizejs/vite-plugin`  |

## Install

```sh
npx jsr add @vizejs/vize    # Node.js 22 or 24+
bunx jsr add @vizejs/vize   # Bun
```

```ts
import { defineConfig } from "@vizejs/vize";
import vizeBabel from "@vizejs/vize/babel";
import { compileSfc } from "@vizejs/vize/native";
import vizeOxlint, { configs as vizeLintConfigs } from "@vizejs/vize/oxlint";
import vize from "@vizejs/vize/vite";

export default defineConfig({ compiler: { vapor: false } });

const result = compileSfc("<template><h1>Hello</h1></template>", {
  filename: "Hello.vue",
});
const plugin = vize();
const babelPlugin = vizeBabel;
const oxlintPlugin = vizeOxlint;
const recommendedRules = vizeLintConfigs.recommended;
void babelPlugin;
void oxlintPlugin;
void recommendedRules;
```

## Deno 2.9.7

From a new empty directory, run the same published API smoke checked on Ubuntu 24.04, Linux x64 with glibc:

```sh
VIZE_JSR_EXAMPLES=https://raw.githubusercontent.com/ubugeeei-prod/vize/1b6f856f4f276b853f649df42943a1fd5ac2798d/tools/support/release/jsr
curl -fsSL "$VIZE_JSR_EXAMPLES/consumer-deno.json" -o deno.json
curl -fsSL "$VIZE_JSR_EXAMPLES/consumer.ts" -o consumer.ts
deno run --config deno.json --allow-all consumer.ts jsr:@vizejs/vize@0.457.0 --core-only
```

It prints `Verified jsr:@vizejs/vize@0.457.0 (--core-only)` after importing configuration
and native APIs and compiling an SFC. The canonical configuration keeps external
dependencies at `P1D` (24 hours), exempts exactly 15 listed Vize-owned packages for
fresh releases, and uses Deno's cache with `nodeModulesDir: "none"`. Merge those
settings into an existing Deno project. `--allow-all` grants full permissions,
including native FFI; use it with trusted build dependencies.

See the [JSR guide](https://vizejs.dev/guide/jsr) or
[日本語ガイド](https://vizejs.dev/ja/guide/jsr) for installation, imports, Vite/Vue
peers and the tested runtime boundaries. The Deno smoke does not exercise `/vite`.

## Requirements

- The native binaries are installed from npm as optional dependencies, so the
  install must reach `registry.npmjs.org` and keep optional dependencies on.
  Deno loads them through Node-API and needs `--allow-ffi`.
- Supported native targets are macOS x64/arm64, Windows x64/arm64, and Linux
  x64/arm64 with glibc or musl. Other targets have no JavaScript fallback.
- `@vizejs/vize/vite` needs `vite` and `vue` in the project.
- `@vizejs/vize/babel` is the Babel 7/8 integration; install a supported `@babel/core` host before configuring it.
- `@vizejs/vize/oxlint` exposes the JavaScript plugin and config helpers. The actual lint host still comes from Oxlint or Vite+.
- The package has no CLI. Install [`vize` from npm](https://www.npmjs.com/package/vize)
  for the `vize` command. For browsers, use `@vizejs/wasm` from npm.
