# Vize

The `vize` npm package is the default application-facing entry point for package scripts and shared
configuration. It provides:

- shared config utilities (`defineConfig`, `loadConfig`)
- package-script commands backed by the native binding: `build`, `fmt`, `lint`, and `check`
- `ready` for `fmt --write -> lint -> check -> build`
- `upgrade` for updating the npm package

For Vite integration, pair it with `@vizejs/vite-plugin`.
For the full Rust-native CLI (`lsp`, `ide`, project-backed `check`, and `check-server`), use the
GitHub release binaries or the Nix entry point. The Rust CLI is not published through crates.io for
v1 alpha.

Need `vp` first? Install Vite+ once from the [Vite+ install guide](https://viteplus.dev/guide/install).

## Installation

For an existing Vite or Vite+ project, run the setup command from the project root:

```bash
vp dlx vize setup
```

The command:

- installs `vize`, the Vite and Musea plugins, Oxlint, and `oxlint-plugin-vize`;
- creates `vize.config.ts` and, for plain Vite projects, `oxlint.config.ts` when no supported config
  already exists;
- enables the Vize Oxlint preset in the `lint` block of a standard Vite+ config, so `vp lint`
  checks Vue files;
- adds non-conflicting `vize:*` package scripts for build, format, lint, check, Musea, and `ready`;
- replaces a canonical zero-option `@vitejs/plugin-vue` import with `@vizejs/vite-plugin`, then
  removes the old dependency.

Setup is idempotent. It preserves existing config files and package scripts, and it leaves custom
Vite plugin calls such as `vue({ ... })` unchanged because those options need a deliberate
migration. Use `vize setup --no-install` to generate the files and scripts without changing
dependencies.

After installation, `vz` is a short alias for the same CLI:

```bash
vz ready src
```

The remaining work for the full
[Vize Plus proposal](https://github.com/ubugeeei-prod/vize/issues/3278) is safe merging into existing
Vize and Vite+ lint/Oxlint configs, option-aware migration of custom or multiple Vite configs, and
project-specific Musea plugin injection. Those cases are intentionally reported and preserved by
this first setup slice instead of being rewritten heuristically.

For manual installation:

```bash
vp install -D vize
```

The package declares TypeScript 7's native Corsa runtime as an optional dependency, so standard
installs include the runtime needed by `vize check`. The `--corsa-path` CLI option remains available
for custom native TypeScript builds.

## Package Scripts

Add scripts to your project and run Vize through your package manager:

```json
{
  "scripts": {
    "vize:build": "vize build src",
    "vize:fmt": "vize fmt --write src",
    "vize:lint": "vize lint --preset happy-path src",
    "vize:check": "vize check",
    "vize:ready": "vize ready src",
    "vize:upgrade": "vize upgrade"
  }
}
```

Then run:

```bash
vp run vize:build
vp run vize:fmt
vp run vize:lint
vp run vize:check
vp run vize:ready
vp run vize:upgrade
```

For one-off local debugging, the installed binary is also available through npm exec:

```bash
vp exec vize lint --preset essential src
```

Shared config discovery is supported for the npm package commands:

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
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
    templateSyntax: "standard",
  },
  experimentals: {
    vapor: false,
    jsxVapor: false,
    intagComment: false,
    pattenedTemplate: false,
    serverScript: false,
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

`vize lint` runs Vue-aware Patina diagnostics through the native binding. Prefer named package
scripts for the presets your project uses:

```json
{
  "scripts": {
    "vize:lint:ci": "vize lint --preset essential --max-warnings 0 src",
    "vize:lint:ecosystem": "vize lint --preset ecosystem src",
    "vize:lint:opinionated": "vize lint --preset opinionated --help-level short src",
    "vize:lint:json": "vize lint --format json src",
    "vize:lint:plain": "vize lint --format plain src",
    "vize:lint:agent": "vize lint --format agent src"
  }
}
```

```bash
vp run vize:lint:ci
vp run vize:lint:ecosystem
vp run vize:lint:opinionated
vp run vize:lint:json
vp run vize:lint:plain
vp run vize:lint:agent
```

Lint output supports `text`, `ansi`, `plain`, `json`, `stylish`, `markdown`, `html`, and `agent`.
The human and agent-friendly formats include local rule documentation paths such as
`docs/content/rules/vue.md`.

`vize check` in the npm package uses the packaged NAPI checker and TypeScript 7's native Corsa
runtime, so it can run from `package.json` scripts after installing `vize`:

```json
{
  "scripts": {
    "vize:check:strict": "vize check src --strict",
    "vize:check:virtual-ts": "vize check src --show-virtual-ts",
    "vize:check:declarations": "vize check src --declaration --declaration-dir dist/types"
  }
}
```

```bash
vp run vize:check:strict
vp run vize:check:virtual-ts
vp run vize:check:declarations
```

Use the Rust CLI when you need Corsa project diagnostics across Vue, TS, TSX, and `.d.ts` inputs.

`vize ready` runs `fmt --write`, `lint`, `check`, and `build` in that order.

## TypeScript Content Mapper

The TypeScript 7.1
[API roadmap](https://github.com/microsoft/typescript-go/issues/4830) identifies Content Mappers as
the TS Server plugin replacement needed by Vue. Vize publishes the package metadata and protocol
server merged upstream in
[microsoft/typescript-go#4712](https://github.com/microsoft/typescript-go/pull/4712),
letting a `tsgo` build with content-mapper support transform `.vue` files directly instead of
materializing a parallel `.vue.ts` project.

```json
{
  "compilerOptions": {
    "module": "preserve",
    "strict": true
  },
  "contentMappers": [
    {
      "package": "vize",
      "extensions": [".vue"]
    }
  ],
  "include": ["src"]
}
```

```bash
tsgo --runExternalCode --noEmit -p tsconfig.json
```

Content Mappers are merged on the `typescript-go` main branch but are not in a released TypeScript
native preview yet. Use a `tsgo` built from main while evaluating them, keep `--runExternalCode`
explicit, and keep `vize check` as the supported typecheck path until a native preview release
ships the protocol. Vize negotiates protocol v1 with UTF-8 mappings, resolves its mapper options
and its declared `noUnusedLocals` compiler-option dependency per project through the
`openProject`/`closeProject` lifecycle (invalid options surface as `optionDiagnostics` in the
tsconfig), maps `<!-- @vue-expect-error -->` and `<!-- @vue-ignore -->` template comments onto
the protocol's diagnostic directives, and tags every transform response with the `.tsx` virtual
extension so both TypeScript and embedded JSX parse correctly. See the
[Content Mapper guide](https://vizejs.dev/guide/content-mapper) for setup and directive semantics.

## Compiler and Tool Options

Important shared fields:

| Field                            | Used by                        | Purpose                                                            |
| -------------------------------- | ------------------------------ | ------------------------------------------------------------------ |
| `compiler.sourceMap`             | Vite plugin                    | Enable source maps                                                 |
| `compiler.ssr`                   | npm build, Vite plugin         | Force SSR compilation                                              |
| `compiler.vapor`                 | npm build, Vite plugin         | Enable Vapor compilation                                           |
| `compiler.customRenderer`        | npm build, Vite plugin         | Treat lowercase non-HTML tags as custom renderer elements          |
| `compiler.customElements`        | npm build, Vite plugin         | Tag patterns compiled as custom elements instead of Vue components |
| `compiler.templateSyntax`        | npm build, Vite plugin         | Choose standard, strict, or quirks template syntax mode            |
| `experimentals.vapor`            | npm build, Vite plugin         | Opt into experimental SFC Vapor before compiler support            |
| `experimentals.jsxVapor`         | Vite plugin                    | Opt into experimental JSX Vapor by default                         |
| `experimentals.intagComment`     | npm build, Vite plugin, syntax | Opt into in-tag `//` comments                                      |
| `experimentals.pattenedTemplate` | npm build, Vite plugin         | Opt into `v-match` / `v-case` templates                            |
| `experimentals.serverScript`     | npm build, Vite plugin         | Preserve server-script RFC opt-in surface                          |
| `compiler.compatibility`         | integrations                   | Opt into legacy Vue, Nuxt, CDN, Vapor, or Webpack bridges          |
| `compiler.scriptExt`             | npm build                      | Preserve TypeScript output or downcompile to JavaScript            |
| `vite.scanPatterns`              | Vite plugin                    | Pre-compile matching Vue files                                     |
| `linter.preset`                  | npm lint                       | Select the Patina lint preset                                      |
| `typeChecker.strict`             | npm check                      | Enable strict checks                                               |
| `formatter.printWidth`           | npm fmt                        | Set formatting width                                               |

### Template syntax

`compiler.templateSyntax` defaults to `"standard"`.

- `"standard"` warns and rewrites invalid non-void HTML self-closing tags such as `<div />`.
- `"strict"` reports invalid syntax as compilation errors.
- `"quirks"` preserves compatibility quirks without extra warnings.

```text
<template>
  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="(item in items">{{ item }}</div>

  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="item) in items">{{ item }}</div>

  <!-- Standard warns and rewrites this as `<div></div>`. Strict errors. Quirk keeps it as a leaf. -->
  <div />
</template>
```

Vue upstream reference:

- [`forAliasRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/utils.ts#L571)
- [`stripParensRE` in `parseForExpression`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts#L493-L530)

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
