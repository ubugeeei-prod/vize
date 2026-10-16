---
title: JavaScript APIs through JSR
description: Use Vize's published JSR facade with Node, Deno or Bun, including its native requirements.
---

# JavaScript APIs through JSR

[`@vizejs/vize`](https://jsr.io/@vizejs/vize) exposes Vize's JavaScript APIs. Each entry point re-exports an npm package at the same exact Vize version. The compiler still loads the native binary from npm.

| Import | API | npm package |
| --- | --- | --- |
| `@vizejs/vize` | `defineConfig` and configuration types | `vize` |
| `@vizejs/vize/config` | Configuration loading and normalization | `vize/config` |
| `@vizejs/vize/native` | Compiler, linter, formatter and analysis APIs | `@vizejs/native` |
| `@vizejs/vize/vite` | Vite plugin and options | `@vizejs/vite-plugin` |

## Node.js and Bun

In a project using Node.js 22 or 24+, install the JSR facade and the Vite integration's peers:

```bash
npm install vite@8 vue@3
npx --yes jsr add @vizejs/vize@0.457.0
```

For Bun:

```bash
bun add vite@8 vue@3
bunx jsr add @vizejs/vize@0.457.0
```

Import the APIs from the installed facade:

```ts
import { defineConfig } from "@vizejs/vize";
import { compileSfc } from "@vizejs/vize/native";
import vize from "@vizejs/vize/vite";

const config = defineConfig({ compiler: { vapor: false } });
const compiled = compileSfc("<template><h1>Hello</h1></template>", {
  filename: "Hello.vue",
});
const plugin = vize();
```

These examples pin the published 0.457.0 facade, which already pins its Vize npm dependencies to the same release. For a Vue application, the [Vite integration](./vite-plugin.md) and [getting started guide](../getting-started.md) explain the project configuration.

## Deno 2.9.7

The following smoke check is verified with Deno 2.9.7 on Ubuntu 24.04, Linux x64 with glibc. Start in a new directory. Download the shared consumer policy and the [small public API example](https://github.com/ubugeeei-prod/vize/blob/main/tools/support/release/jsr/consumer.ts):

```bash
mkdir vize-jsr-example
cd vize-jsr-example
curl -fsSL https://raw.githubusercontent.com/ubugeeei-prod/vize/main/tools/support/release/jsr/consumer-deno.json -o deno.json
curl -fsSL https://raw.githubusercontent.com/ubugeeei-prod/vize/main/tools/support/release/jsr/consumer.ts -o consumer.ts
deno run --config deno.json --allow-all consumer.ts jsr:@vizejs/vize@0.457.0
```

Expected output:

```text
Verified jsr:@vizejs/vize@0.457.0 (no flags)
```

The example imports configuration and native APIs, compiles a Vue component, and checks the result. In your own Deno code, use versioned specifiers such as `jsr:@vizejs/vize@0.457.0/native`.

The [shared policy](https://github.com/ubugeeei-prod/vize/blob/main/tools/support/release/jsr/consumer-deno.json) keeps Deno's external dependency minimum age at `P1D` (24 hours). Exactly 13 explicitly listed Vize-owned packages may resolve immediately after a release. This is a package list, not an exemption for the whole `@vizejs` namespace. The configuration also sets `nodeModulesDir: "none"`, matching the tested consumer and avoiding `package.json`'s uninstalled manual `node_modules` mode. Merge these settings into an existing Deno project rather than replacing its other configuration.

The command uses `--allow-all`, as the tested native consumer does. It grants full permissions, including the [FFI permission](https://docs.deno.com/runtime/fundamentals/ffi/) needed to load Node-API native code. Use it with trusted build dependencies. This smoke check does not establish a smaller permission set or Deno support on every native target. The [read-only Actions proof](https://github.com/ubugeeei-prod/vize/actions/runs/38904123567) records the actual published 0.457.0 import and SFC compilation in a fresh cache; separate controls verify that young external packages remain blocked.

## Runtime and package boundaries

- Native packages require access to `registry.npmjs.org` and optional dependencies. Available prebuilt targets are macOS x64/arm64, Windows x64/arm64, and Linux x64/arm64 with glibc or musl. An available binary is distinct from a tested Deno/runtime combination; see the [runtime support matrix](../stability.md#runtime-support).
- `/vite` needs Vite and Vue in the consuming project. The native Deno smoke above does not exercise the Vite plugin; use the Node or Bun Vite integration for that workflow.
- JSR provides JavaScript APIs, not the `vize` executable. Install the npm [`vize` package](https://www.npmjs.com/package/vize) for the [standalone CLI](./cli.md).
- Browser usage uses the npm [`@vizejs/wasm` package](./wasm.md). The JSR native entry point has no JavaScript or browser fallback.
