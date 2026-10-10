# Vize for Node.js through JSR

This package exposes the existing Vize npm APIs through JSR. It requires Node.js
22 or newer. Native dependencies remain npm packages; installation must allow
optional dependencies and registry.npmjs.org access.

| Import                | API                                                  |
| --------------------- | ---------------------------------------------------- |
| `@vizejs/vize`        | Vize configuration utilities and types               |
| `@vizejs/vize/config` | Configuration loading and normalization              |
| `@vizejs/vize/native` | Native compiler, linter, formatter and analysis APIs |
| `@vizejs/vize/vite`   | Default Vite plugin and its public exports           |

Install in a Node project with `npx jsr add @vizejs/vize`. Then use:

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

The JSR package and its npm dependencies use the same exact release version.
Supported native targets are macOS x64/arm64, Windows x64/arm64, and Linux
x64/arm64 with glibc or musl. Unsupported targets have no JavaScript fallback.
Node imports on Linux x64, macOS arm64 and Windows x64 are exercised against
the published package in Actions; other native targets retain the existing npm
release platform qualification.

JSR does not provide the Vize CLI executable. Install `vize` from npm to run
`vize`/`vz`. Browser, Cloudflare Workers, Deno and Bun execution are outside
this package's supported boundary. The browser WASM package remains
`@vizejs/wasm` on npm. Vite and Vue are project peer dependencies.

See [the distribution guide](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/jsr.md)
for publication status and release verification.
