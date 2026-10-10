---
title: JSR distribution
description: Vize's Node.js API entry points, native dependency boundary, and publication verification.
---

# JSR distribution

The JSR offering is prepared as `@vizejs/vize`. **First publication and published
consumer verification remain pending** in [#8367](https://github.com/ubugeeei-prod/vize/issues/8367).
The commands below become usable after that package is published. Preparation
and a publish dry-run do not establish JSR support.

## Package boundary

JSR distributes a small ESM facade over the existing, exact-version npm releases.
It adds no compiler pipeline, native binary duplication or JavaScript fallback.

| Import path after installation | Public API                                                  |
| ------------------------------ | ----------------------------------------------------------- |
| `@vizejs/vize`                 | Configuration utilities and types from npm `vize`           |
| `@vizejs/vize/config`          | Configuration loading and native normalization              |
| `@vizejs/vize/native`          | Public native compiler, linter, formatter and analysis APIs |
| `@vizejs/vize/vite`            | Default Vite plugin and public plugin exports               |

Node.js 22 or newer is required; Vite 8 additionally requires Node.js 22.12 or
newer (or Node.js 24). Native dependencies are installed from npm, so optional
dependencies must be enabled and `registry.npmjs.org` must be reachable.
Supported native platforms are macOS x64/arm64, Windows x64/arm64, and Linux
x64/arm64 with glibc or musl. Published JSR consumers run on Linux x64 with Node
22 and 24, macOS arm64 with Node 24, and Windows x64 with Node 24. The remaining
native targets use the existing npm release platform qualification.

The JSR facade supports Node projects. Deno, Bun, browser and Cloudflare Worker
execution is outside this offering's supported boundary. JSR publishes ESM; the
CommonJS native loader stays inside its existing npm dependency. Browser WASM
remains available as `@vizejs/wasm` on npm.

## Installation and imports after publication

Install into an existing Node project:

```sh
npx jsr add @vizejs/vize
```

For a reproducible installation, append the desired published version:
`npx jsr add @vizejs/vize@<version>`. The installer configures npm's JSR
compatibility registry and preserves the `@vizejs/vize` import name.

```ts
import { defineConfig } from "@vizejs/vize";
import { compileSfc } from "@vizejs/vize/native";

const config = defineConfig({ compiler: { vapor: false } });
const result = compileSfc("<template><h1>Hello</h1></template>", {
  filename: "Hello.vue",
});
```

For Vite, install Vite and Vue as project peer dependencies and import the
plugin from `@vizejs/vize/vite`:

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vize/vite";

export default defineConfig({ plugins: [vize()] });
```

The JSR facade defines no CLI bin entry point. Install the npm `vize` package
directly for the supported CLI workflow. There is no `jsr dlx vize` command or JSR CLI package in this
offering. Other Vize packages continue to use their documented npm distribution.

## Maintainer bootstrap and release verification

The initial registry setup requires a JSR scope administrator:

1. Create the `@vizejs` scope and `@vizejs/vize` package on JSR under the project's
   authorized ownership. Link the package to `ubugeeei-prod/vize` in its settings.
2. Set package compatibility to Node supported and the other runtimes unsupported.
   Retain the scope's default publishing restriction to authorized scope members.
3. Set GitHub repository variable `VIZE_JSR_ENABLED=true` and dispatch **Release
   JSR** from `main` with the exact currently published npm version. No registry
   secret is required: publication uses repository-linked GitHub Actions OIDC.
4. Require all four published consumer jobs to pass. Preserve their registry
   metadata, package locks, exact dependency identities, public TypeScript
   checks, native compilation, configuration loading and real Vite build results.
5. Only then record the initial published version, source SHA and successful run
   on #8367 and replace this page's pending status with those concrete receipts.
6. Review a change enabling `jsr/vize/channel.json`. Subsequent release source
   cuts freeze that policy and require exact-version JSR delivery.

The committed channel policy starts disabled, so existing product releases
continue while registry ownership remains pending. The normal Release workflow
reads the policy from the exact frozen source H after release authorization.
An enabled source requires JSR publication and all four consumer checks after
the matching CLI, native and Vite npm packages publish. Clearing the mutable
repository variable then fails authorization inside the required publisher;
it cannot skip that source's JSR requirement. GitHub Release creation keeps its
existing prerequisites and can complete before the additional JSR lane.
Disable the variable if initial bootstrap verification fails, repair the failure
and verify the same published version before enabling the committed policy.

Release source catalogs retain the JSR workflow, generator, public consumer,
package template, package README, license, channel policy and policy reader
byte for byte. Public verification derives the requirement and exact package
from raw H objects, checks published source digests, and requires the publisher
and every original consumer in that official Release run. A disabled source
has no published JSR credit. Sources predating this lane retain their original
public plan and receipt fields. Record the enabled channel requirement and
actual published consumer receipts for each release.
If publication succeeded and a consumer failed, rerun the failed consumer jobs
on that exact run; do not attempt to overwrite the immutable published version.

The package version and every Vize npm dependency are generated from the same
checked-out release manifests. A mismatch fails before publication. JSR's
publisher uses Deno solely for package validation and publication; fresh exact
release dependencies bypass Deno's default minimum dependency age so release
verification can run immediately.

See the official [JSR publishing guide](https://jsr.io/docs/publishing-packages)
and [Node installation guide](https://jsr.io/docs/using-packages).
