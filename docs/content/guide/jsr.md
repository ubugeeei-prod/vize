---
title: JavaScript APIs through JSR
description: Use Vize's published JSR packages with Node, Deno or Bun, including their native requirements.
---

# JavaScript APIs through JSR

Every JavaScript package in a Vize npm release is also published to JSR at the same exact version. The JSR packages are typed source facades over the npm artifacts, so both registries execute the same implementation. The compiler still loads its platform-specific native binary from npm.

[`@vizejs/vize`](https://jsr.io/@vizejs/vize) is the combined facade for the most common APIs:

| Import                | API                                           | npm package            |
| --------------------- | --------------------------------------------- | ---------------------- |
| `@vizejs/vize`        | `defineConfig` and configuration types        | `vize`                 |
| `@vizejs/vize/babel`  | Babel plugin for Vue SFC and JSX transforms   | `@vizejs/babel-plugin` |
| `@vizejs/vize/config` | Configuration loading and normalization       | `vize/config`          |
| `@vizejs/vize/native` | Compiler, linter, formatter and analysis APIs | `@vizejs/native`       |
| `@vizejs/vize/oxlint` | Oxlint plugin, presets and Vite+ config       | `oxlint-plugin-vize`   |
| `@vizejs/vize/vite`   | Vite plugin and options                       | `@vizejs/vite-plugin`  |

Packages can also be installed directly instead of through the combined facade:

| JSR package                    | npm package                     |
| ------------------------------ | ------------------------------- |
| `@vizejs/vize`                 | `vize`                          |
| `@vizejs/vite-plugin`          | `@vizejs/vite-plugin`           |
| `@vizejs/oxlint-plugin-vize`   | `oxlint-plugin-vize`            |
| `@vizejs/unplugin`             | `@vizejs/unplugin`              |
| `@vizejs/fresco`               | `@vizejs/fresco`                |
| `@vizejs/musea-mcp-server`     | `@vizejs/musea-mcp-server`      |
| `@vizejs/vite-plugin-musea`    | `@vizejs/vite-plugin-musea`     |
| `@vizejs/rspack-plugin`        | `@vizejs/rspack-plugin`         |
| `@vizejs/musea-nuxt`           | `@vizejs/musea-nuxt`            |
| `@vizejs/nuxt-lint-config`     | `@vizejs/nuxt-lint-config`      |
| `@vizejs/nuxt`                 | `@vizejs/nuxt`                  |
| `@vizejs/marquette`            | `@vizejs/marquette`             |
| `@vizejs/composable`           | `@vizejs/composable`            |
| `@vizejs/router`               | `@vizejs/router`                |
| `@vizejs/data`                 | `@vizejs/data`                  |
| `@vizejs/ui`                   | `@vizejs/ui`                    |
| `@vizejs/plugin-sdk`           | `@vizejs/plugin-sdk`            |
| `@vizejs/babel-plugin`         | `@vizejs/babel-plugin`          |

## Node.js and Bun

In a project using Node.js 22 or 24+, install the combined JSR facade and the Vite integration's peers:

```bash
npm install vite@8 vue@3
npx --yes jsr add @vizejs/vize
```

For Bun:

```bash
bun add vite@8 vue@3
bunx jsr add @vizejs/vize
```

Import the APIs from the installed facade:

```ts
import { defineConfig } from "@vizejs/vize";
import vizeBabel from "@vizejs/vize/babel";
import { compileSfc } from "@vizejs/vize/native";
import vizeOxlint, { configs as vizeLintConfigs } from "@vizejs/vize/oxlint";
import vize from "@vizejs/vize/vite";

const config = defineConfig({ compiler: { vapor: false } });
const compiled = compileSfc("<template><h1>Hello</h1></template>", {
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

Every installed JSR package pins its Vize npm dependency to the same exact release. Commit the generated package manifest, lockfile and `.npmrc`; Bun and npm use JSR's npm compatibility registry through that configuration. For a Vue application, the [Vite integration](./vite-plugin.md) and [getting started guide](../getting-started.md) explain the project configuration.

## Deno 2.9.7

The following smoke check is verified with Deno 2.9.7 on Ubuntu 24.04, Linux x64 with glibc. Start in a new directory. Download the shared consumer policy and the [small public API example](https://github.com/ubugeeei-prod/vize/blob/main/tools/support/release/jsr/consumer.ts):

```bash
mkdir vize-jsr-example
cd vize-jsr-example
VIZE_JSR_EXAMPLES=https://raw.githubusercontent.com/ubugeeei-prod/vize/1b6f856f4f276b853f649df42943a1fd5ac2798d/tools/support/release/jsr
curl -fsSL "$VIZE_JSR_EXAMPLES/consumer-deno.json" -o deno.json
curl -fsSL "$VIZE_JSR_EXAMPLES/consumer.ts" -o consumer.ts
deno run --config deno.json --allow-all consumer.ts jsr:@vizejs/vize@0.457.0 --core-only
```

Expected output:

```console
Verified jsr:@vizejs/vize@0.457.0 (--core-only)
```

The example imports configuration and native APIs, compiles a Vue component, and checks the result. In your own Deno code, use versioned specifiers such as `jsr:@vizejs/vize@0.457.0/native`.

The [shared policy](https://github.com/ubugeeei-prod/vize/blob/main/tools/support/release/jsr/consumer-deno.json) keeps Deno's external dependency minimum age at `P1D` (24 hours). Exactly 15 explicitly listed Vize-owned packages may resolve immediately after a release. This is a package list, not an exemption for the whole `@vizejs` namespace. The configuration also sets `nodeModulesDir: "none"`, matching the tested consumer and avoiding `package.json`'s uninstalled manual `node_modules` mode. Merge these settings into an existing Deno project rather than replacing its other configuration.

The command uses `--allow-all`, as the tested native consumer does. It grants full permissions, including the [FFI permission](https://docs.deno.com/runtime/fundamentals/ffi/) needed to load Node-API native code. Use it with trusted build dependencies. This smoke check does not establish a smaller permission set or Deno support on every native target. The [read-only Actions proof](https://github.com/ubugeeei-prod/vize/actions/runs/38904123567) records the actual published 0.457.0 import and SFC compilation in a fresh cache; separate controls verify that young external packages remain blocked.

## Runtime and package boundaries

- Native packages require access to `registry.npmjs.org` and optional dependencies. Available prebuilt targets are macOS x64/arm64, Windows x64/arm64, and Linux x64/arm64 with glibc or musl. An available binary is distinct from a tested Deno/runtime combination; see the [runtime support matrix](../stability.md#runtime-support).
- `/vite` needs Vite and Vue in the consuming project. The native Deno smoke above does not exercise the Vite plugin; use the Node or Bun Vite integration for that workflow.
- `/babel` needs a supported Babel 7 or 8 host. `/oxlint` provides the JavaScript plugin and config helpers; run it through Oxlint or Vite+.
- JSR provides JavaScript APIs, not the `vize` executable. Install the npm [`vize` package](https://www.npmjs.com/package/vize) for the [standalone CLI](./cli.md).
- Browser usage uses the npm [`@vizejs/wasm` package](./wasm.md). The JSR native entry point has no JavaScript or browser fallback.
