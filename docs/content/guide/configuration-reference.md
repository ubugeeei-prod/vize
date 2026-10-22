---
title: Shared Configuration Reference
---

<span id="standalone-configuration-reference"></span>

# Shared Configuration Reference

Use `vue:` in the existing Vite configuration and `vueCompilerOptions` in the
TypeScript project. See [Configuration](./configuration.md) for a minimal setup.

## Config Files

The CLI and editor discover `vite.config.ts`, `.mts`, `.cts`, `.js`, `.mjs` and
`.cjs` in the nearest project. `package.json`, `tsconfig.json` and `jsconfig.json`
form project boundaries. Commands can select a Vite config with `--config`.
Dedicated `vize.config.*` files are no longer supported. A leftover dedicated
file beside a Vite configuration is an error, the same migration error as a
dedicated-only project.

Vite-owned relative paths use the selected Vite `root`, which resolves from the
configuration directory. Explicit CLI inputs and `--tsconfig` use the command's
working directory. Direct plugin options and explicit editor feature switches
override the shared settings.

## TypeScript Config

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  server: { port: 3000 },
}).withVue({
  compiler: {
    sourceMap: true,
    vapor: false,
    templateSyntax: "standard",
  },
  linter: { preset: "happy-path" },
  formatter: { printWidth: 100 },
  languageServer: { hover: true },
});
```

The `vue` section accepts `version`, `compiler`, `typeChecker`, `linter`,
`formatter`, `languageServer`, `globalTypes`, `files`, `ignores`, `basePath`,
`lib`, `vite`, `musea`, `experimentals`, `dialect`, `name` and `entries`. Use native option names. Unknown canonical
members are errors. See the [compiler reference](./compiler-configuration-reference.md)
for compilation modes and [rule options](../rules/options.md) for typed lint options.

## Vue Type Resolution

`vize check` and the language server resolve Vue types from the analyzed project.
Declare the intended Vue version there and keep `vue`, `@vue/compiler-sfc` and
framework integrations aligned. Vize's own build dependencies do not choose the
consumer's Vue version.

Keep ambient declarations, generated auto-import files, path aliases and Vue
`ComponentCustomProperties` declarations in the project's `tsconfig.json`.
Standard TypeScript `extends` (including package and array forms) and project
`references` retain their meaning.

```json
{
  "extends": "./tsconfig.base.json",
  "compilerOptions": { "strict": true, "noEmit": true },
  "vueCompilerOptions": {
    "target": 3.5,
    "strictTemplates": true,
    "checkUnknownComponents": true
  },
  "include": ["src/**/*"]
}
```

Supported Vue options include `target`, `strictTemplates`, `checkUnknownProps`,
`checkUnknownComponents`, `checkUnknownEvents`, `checkUnknownDirectives`,
`strictVModel`, `strictCssModules`, `jsxSlots`, `inferComponentDollarEl`,
`inferComponentDollarRefs`, `inferTemplateDollarEl`, `inferTemplateDollarRefs`,
`inferTemplateDollarSlots`, `inferTemplateDollarAttrs`, `fallthroughAttributes`,
`checkRequiredFallthroughAttributes`, `resolveStyleImports`, `resolveStyleClassNames`
and `vapor`. This does not imply support for every `vue-tsc` option.

`target` accepts Vue 2.x through 2.7, Vue 3.x, `99` and `"auto"`. Automatic target
selection follows the installed project Vue package; explicit `vue.version` in
Vite takes precedence. Unsupported target values produce a configuration error.
TypeScript strictness remains in `compilerOptions`; explicit Vize typechecker
options override the corresponding project defaults.

### TypeScript project ownership and support limits

These options come from the TypeScript project that owns the source file, including inherited JSONC configuration, standard package/array `extends` and project `references`. `target: "auto"` resolves the project's installed Vue package; a missing or unsupported version produces a configuration error. Numeric Vue 2 targets below 2.7 select Vue 2, exactly 2.7 selects Vue 2.7, and Vue 3.x or `99` selects the modern Vue 3 family. An explicitly supplied `vue.version` in Vite, including `"3"`, takes precedence over `target`.

| Vue compiler option | Supported behavior |
| --- | --- |
| `target` | Select the project's Vue target as described above. |
| `strictTemplates` | Supply defaults for the supported strict template checks; explicit options remain authoritative. |
| `checkUnknownProps`, `checkUnknownComponents`, `checkUnknownEvents`, `checkUnknownDirectives` | Configure the corresponding unknown template member checks. |
| `strictVModel`, `strictCssModules`, `strictComponentAttrs` | Configure the corresponding checks. `strictComponentAttrs` is explicit; `strictTemplates` does not enable it automatically. |
| `jsxSlots` | Configure JSX slot checking. |
| `inferComponentDollarEl`, `inferComponentDollarRefs`, `inferTemplateDollarEl`, `inferTemplateDollarRefs`, `inferTemplateDollarSlots`, `inferTemplateDollarAttrs` | Configure the corresponding component/template inference. |
| `fallthroughAttributes`, `checkRequiredFallthroughAttributes` | Configure fallthrough attribute handling and checks. |
| `resolveStyleImports`, `resolveStyleClassNames` | Configure the supported style import/class resolution. |
| `vapor` | Set the supported Vapor project option. |
| `plugins` | Recognize `vue-router/volar/sfc-typed-router`; arbitrary Vue language plugins are not loaded. |

`optionsWrapper` is unsupported. In Vue tooling it describes code prefix/suffix wrapping, not an Options API switch. Vize's existing Options API checking is controlled separately by `vue.typeChecker.optionsApi`. The mapping above is bounded support, not complete `vue-tsc` option parity.

### Options API and legacy Vue

`vue.typeChecker.optionsApi` resolves Vue 3 Options API template bindings from `data`, `computed`, `methods`, `inject`, `setup`, and `props` in ordinary `<script> export default { ... }` components. It is enabled by default in the standard build, matching `vue-tsc`. Script-setup components skip this path. Set `optionsApi: false` to disable it.

Vue 2.7 and Nuxt 2 compatibility require the separate `legacy` build. `vue.typeChecker.legacyVue2` supplies the Nuxt 2 template globals. Keep the target Vue dependencies and ambient declarations in the owning package; choosing a Corsa executable does not select a Vue type version.

## Experimental Flat Entries

Use the target package's TypeScript project for monorepo checking. Scoped compiler
and Vue lint settings can use `vue.entries`:

```ts
export default defineConfig({
  root: "apps/web",
  vue: {
    entries: [
      {
        files: ["src/trusted/**/*.vue"],
        compiler: { whitespace: "condense" },
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  },
});
```

Entries accept `basePath`, `files`, `ignores`, `compiler` and `linter`; other sections remain
project-wide. Do not move TypeScript project references into these entries.

## Shared Bases with `extends`

There is no Vize-specific `extends`. Reuse ordinary JavaScript objects:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import vpConfig from "./build-options.ts";
import vueConfig from "./vue-options.ts";

export default defineConfig(vpConfig).withVue(vueConfig);
```

This does not change standard TypeScript `extends` or Vite config functions.

## Static Analysis Options

Set native Vue lint options in `vue.linter` and language-server options in
`vue.languageServer`. Vue rules use `vue.linter.rules`; Oxlint rules keep their
existing `lint.rules` location. See [Lint execution](./lint-execution.md) for
cross-file and type-aware checks and [Rule Options](../rules/options.md) for
complete option objects. Unknown rule option fields are rejected.

`vue.typeChecker.tsconfig` selects another TypeScript project. Use
`vue.typeChecker.corsaPath` only to select an installed checker executable, not
Vue types. `vue.typeChecker.servers` limits parallel checker processes (`1`–`32`);
when omitted, Vize tunes concurrency to the project and available CPUs.

## Musea Options

See [Musea shared configuration](#musea-shared-configuration). Presentation options
such as `previewCss`, `previewSetup`, `tokensPath`, `theme` and `storybookOutDir`
belong in the `musea()` plugin options.

## Musea shared configuration

Keep shared discovery, route and VRT settings in `vue.musea`. Direct `musea()`
options override corresponding shared values.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
import { musea } from "@vizejs/vite-plugin-musea";

export default defineConfig({
  plugins: [musea({ storybookCompat: false })],
  vue: {
    musea: {
      include: ["src/**/*.art.vue"],
      exclude: ["node_modules/**", "dist/**"],
      basePath: "/__musea__",
      inlineArt: false,
    },
  },
});
```

`musea-vrt` reads shared `include`, `exclude` and `vrt` options from the same Vite
configuration. `vize musea new` creates an example story without creating a
dedicated configuration file.

## Migration recipes

Dedicated JSON/PKL configuration and custom `extends` are removed. Reuse ordinary imported JavaScript objects. These recipes preserve the supported settings when migrating existing projects.

### Dynamic Vite settings

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig(({ command, mode, isSsrBuild }) => ({
  vue: {
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
    languageServer: {
      lint: true,
      typecheck: false,
      editor: false,
      formatting: false,
    },
    musea: {
      include: ["src/**/*.art.vue"],
      basePath: "/__musea__",
    },
  },
}));
```

### Convert retired JSON and PKL settings

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  compiler: {
    sourceMap: true,
    vapor: false,
    customRenderer: false,
    templateSyntax: "standard",
  },
  vite: { scanPatterns: ["src/**/*.vue"] },
  linter: { preset: "happy-path" },
  typeChecker: { enabled: true, strict: true },
  languageServer: { lint: true, typecheck: false, editor: false, formatting: false },
  musea: { include: ["src/**/*.art.vue"], basePath: "/__musea__" },
});
```

### Separate package projects

`entries` scopes lint settings only. Keep package-specific checker and formatter options in each package’s Vite and TypeScript project. The two files below belong to separate package projects.

```ts
// apps/web/vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  name: "web app",
  files: ["src/**/*.vue"],
  formatter: { printWidth: 100 },
  typeChecker: { tsconfig: "tsconfig.app.json" },
});
```

```ts
// packages/ui/vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
export default defineConfig({}).withVue({
  name: "ui package",
  files: ["src/**/*.vue"],
  formatter: { printWidth: 100, singleQuote: true },
});
```

### Static analysis and checker processes

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    enabled: true,
    preset: "opinionated",
    rules: {
      "vue/require-v-for-key": "error",
      "vue/no-v-html": "warn",
    },
  },
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

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  typeChecker: { servers: 1 },
});
```

### Existing workflow settings

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
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
