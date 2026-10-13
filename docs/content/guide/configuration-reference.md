---
title: Shared Configuration Reference
---

<span id="standalone-configuration-reference"></span>

# Shared Configuration Reference

Prefer existing `vite.config.*` and `tsconfig.json`; see [Configuration](./configuration.md). This page documents the native settings and optional dedicated formats.

## Config Files

Vite config is discovered when no dedicated config exists in the same nearest project directory. The CLI also accepts an explicit `--config`; direct plugin options and explicit editor feature switches take precedence. Dedicated-config paths use the config directory. Vite-owned `typeChecker` paths and scoped `basePath` use the selected Vite `root`; explicit CLI inputs retain their invocation-directory base. For package scopes, use `vize.entries` and the target package's TypeScript project.

The npm package commands and `@vizejs/vite-plugin` load these files from the project root in this priority order:

- `vize.config.pkl`
- `vize.config.ts`
- `vize.config.js`
- `vize.config.mjs`
- `vize.config.json`

The Rust CLI reads the same config file names in the order above for command-native settings such as
`check`, `lint`, `lsp`, and `fmt`.

## TypeScript Config

```ts
import { defineConfig } from "vize";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  compiler: {
    sourceMap: mode !== "production",
    ssr: isSsrBuild,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: {
    include: [/\.vue$/],
    exclude: [/node_modules/],
    scanPatterns: ["src/**/*.vue"],
    ignorePatterns: ["node_modules/**", "dist/**", ".git/**"],
  },
  linter: {
    enabled: command !== "build",
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
  },
  formatter: {
    printWidth: 100,
    singleQuote: false,
  },
  lsp: {
    lint: true,
    typecheck: false,
    editor: false,
    formatting: false,
  },
  musea: {
    include: ["src/**/*.art.vue"],
    basePath: "/__musea__",
  },
}));
```

## Vue Type Resolution

Vize does not pin Vue's type surface from the published `vize` package: `vize check`, the language
server, and package commands resolve `vue`, `@vue/compiler-sfc`, and related ambient types from the
analyzed project, so Vue 3 patch, minor, and prerelease choices stay under that project's control
rather than the version used to build Vize. For predictable results, declare the supported Vue
version in the user project (not via Vize internals), keep `vue`, `@vue/compiler-sfc`, and
integrations such as Nuxt aligned there, and run `vize check` from the project root or point
`typeChecker.tsconfig` at the target package; use `typeChecker.corsaPath` only to pick the checker
binary, never to override Vue type versions. When a project supports multiple Vue ranges, test each
in its own package matrix so Vize follows the active dependency graph, not a hard-coded type path.

## Experimental Flat Entries

Monorepos can describe root defaults and package-scoped overrides with `entries`. Plain object
configs are normalized to one entry internally, and array exports are accepted by `defineConfig` for
ESLint-flat-config-style authoring.

```ts
export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  entries: [
    {
      name: "web app",
      basePath: "apps/web",
      files: ["src/**/*.vue"],
      typeChecker: {
        tsconfig: "tsconfig.app.json",
      },
    },
    {
      name: "ui package",
      basePath: "packages/ui",
      files: ["src/**/*.vue"],
      formatter: {
        singleQuote: true,
      },
    },
  ],
});
```

## PKL Config

```pkl
amends "node_modules/vize/pkl/vize.pkl"

compiler {
  sourceMap = true
  vapor = false
  customRenderer = false
  templateSyntax = "standard"
}

vite {
  scanPatterns = new Listing {
    "src/**/*.vue"
  }
}

linter {
  preset = "happy-path"
}

typeChecker {
  enabled = true
  strict = true
}

entries = new Listing {
  new ConfigEntry {
    name = "web app"
    basePath = "apps/web"
    files = new Listing { "src/**/*.vue" }
    typeChecker {
      tsconfig = "tsconfig.app.json"
    }
  }
}

lsp {
  lint = true
  typecheck = false
  editor = false
  formatting = false
}
```

## JSON Config

```json
{
  "$schema": "./node_modules/vize/schemas/vize.config.schema.json",
  "compiler": {
    "sourceMap": true,
    "vapor": false,
    "customRenderer": false,
    "templateSyntax": "standard"
  },
  "vite": {
    "scanPatterns": ["src/**/*.vue"]
  },
  "linter": {
    "preset": "happy-path"
  },
  "typeChecker": {
    "enabled": true,
    "strict": true
  },
  "musea": {
    "include": ["src/**/*.art.vue"],
    "basePath": "/__musea__"
  }
}
```

## Compiler reference

[Compiler options and syntax](./compiler-configuration-reference.md) covers the option table,
template syntax modes, JSX/TSX output modes, and standalone HTML dialect detection.

## Static Analysis Options

Use `linter` for the npm lint path:

```ts
export default defineConfig({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
});
```

### Lint Rule Options

Some rules accept typed settings under `linter.ruleOptions`; see
[Rule Options](../rules/options.md) for the complete table. Severity still belongs in
`linter.rules`.

Use `typeChecker` for the npm check path:

```ts
export default defineConfig({
  typeChecker: {
    enabled: true,
    strict: true,
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    // Vue 3 Options API template bindings; default-on (matches vue-tsc).
    optionsApi: true,
  },
});
```

`typeChecker.optionsApi` resolves Vue 3 Options API template bindings
(`data`/`computed`/`methods`/`inject`/`setup`/`props` on a plain `<script> export default { ... }`).
It ships in the standard build (not the `legacy` feature), is **on by default** (matching `vue-tsc`),
and runs only for non-`<script setup>` components, so that common path stays zero-cost; set
`optionsApi: false` to opt out. Legacy Vue 2.7 / Nuxt 2 support (`typeChecker.legacyVue2`, which adds
the Nuxt 2 template globals) is a separate `legacy`-build opt-in.

`typeChecker.tsconfig` and `typeChecker.corsaPath` are part of the shared schema, but the
project-backed Corsa path is the Rust CLI surface today. `corsaPath` is shared by `vize check`,
type-aware `vize lint`, and `vize lsp` (`typeChecker.tsgoPath` is a deprecated alias); the runtime
stack is the TypeScript 7 native platform package (`typescript` / `@typescript/typescript-*`) plus
the Corsa/corsa-bind API layer. Leave `corsaPath` unset unless you need to point Vize at a specific
installed `lib/tsc` executable. Keep ambient declarations, generated auto-import files, path aliases, and Vue
`ComponentCustomProperties` declarations in your project `tsconfig.json`, and use a package script
such as `vize:check:app` for `--tsconfig` or `--corsa-path` overrides.

```json
{ "typeChecker": { "servers": 1 } }
```

`typeChecker.servers` is reserved for future Corsa worker pools. The direct project-session runner
currently supports only `1`; larger values fail fast instead of pretending to tune concurrency.

## Musea Options

Shared config currently covers the gallery file set and route:

```ts
export default defineConfig({
  musea: { include: ["src/**/*.art.vue"], exclude: ["node_modules/**", "dist/**"], basePath: "/__musea__", storybookCompat: false, inlineArt: false },
});
```

Pass presentation-focused options such as `previewCss`, `previewSetup`, `tokensPath`, `theme`, and
`storybookOutDir` directly to `musea()` in `vite.config.ts`.

## Existing dedicated workflow settings

Projects already using a dedicated `vize.config.*` can retain this complete workflow preset. For new projects, prefer the existing Vite configuration in [User Workflows](./workflows.md); published-release availability is explained in [Configuration](./configuration.md).

```ts
import { defineConfig } from "vize";

export default defineConfig({
  formatter: {
    printWidth: 100,
  },
  linter: {
    preset: "happy-path",
  },
  typeChecker: {
    enabled: true,
    strict: true,
    tsconfig: "tsconfig.json",
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
```

## Musea shared configuration

Put shared defaults in the top-level `vize.musea` option of `vite.config.*`.
`musea()` options override individual shared options. Existing `vize.config.*`
files remain supported and take precedence over shared settings in the same Vite project.

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [musea()],
  vize: {
    musea: {
      include: ["src/**/*.art.vue"],
      exclude: ["node_modules/**", "dist/**"],
      basePath: "/__musea__",
      storybookCompat: false,
      inlineArt: false,
    },
  },
});
```

`musea-vrt` also reads shared `include`, `exclude`, and `vrt` settings from this
Vite configuration. Shared `vrt.outDir` selects the snapshot directory; plugin
`vrt` options override the corresponding shared options. `vize musea new` creates
an example art file and uses Musea's default discovery without generating a
dedicated configuration file.
