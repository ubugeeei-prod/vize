---
title: Shared Vue configuration
---

# Shared Vue configuration

Keep native Vue settings in the `vue` section of `vite.config.ts`. The compiler,
CLI and editor use the same native option names. Start with the
[Vite+ integration](./vite-plus.md) to run build, check, lint and format tasks.

## One configuration

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    version: "3",
    compiler: { sourceMap: true },
    typeChecker: { strict: true },
    linter: { preset: "essential" },
    formatter: { singleQuote: true },
    languageServer: { hover: true },
  },
  server: { port: 3000 },
});
```

`vue` uses the native `compiler`, `typeChecker`, `linter`, `formatter`,
`languageServer`, `globalTypes`, `files`, `ignores`, `basePath` and `lib` members.
It also accepts `version`, `vite`, `musea`, `experimentals`, `dialect`, `name` and `entries`. Compiler and formatter sections contain
options; they do not have an `enabled` member. Typechecker, linter and language
server sections retain their native `enabled` flag. Unknown canonical members
produce an error.

## Keep an existing Vite+ configuration

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = { server: { port: 3000 } };
const vueConfig = {
  compiler: { sourceMap: true },
  formatter: { singleQuote: true },
};

export default defineConfig(vpConfig).withVue(vueConfig);
```

The helper accepts the existing Vite+ object, promise, function or async function.
It forwards Vite's environment to configuration functions and preserves plugins,
hooks and Vite+ fields. Vue plugin injection runs once. Reuse ordinary imported
objects; a Vize-specific config file or `extends` is unnecessary.

## Paths and scopes

Relative native paths in Vite configuration use the selected Vite `root`, which
itself resolves from the configuration directory. Explicit CLI file arguments
retain their existing working-directory semantics.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  root: "packages/app",
  vue: {
    files: ["src/**/*.vue"],
    entries: [
      {
        files: ["src/trusted/**/*.vue"],
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  },
});
```

Canonical entries currently accept only `basePath`, `files`, `ignores` and
`linter`. Other sections and `version` remain project-wide; placing them in an
entry is an error. The CLI's scoped linter policy retains its existing behavior.
Per-file editor scope support and additional scoped sections remain unfinished.

## TypeScript project settings

Keep TypeScript `compilerOptions`, standard `extends` and `references` in the
existing TypeScript project. Put Vue template options such as `strictTemplates`
in its `vueCompilerOptions`; the CLI and editor use this project configuration.
See [Configuration](./configuration.md#typescript-and-vue-template-checking).

## Migrate existing settings

Move top-level `vize:` and dedicated `vize.config.*` native settings into `vue:`.
Use `typeChecker`, `linter` and `formatter` for Vue tools. Vite+ task extensions
use `typecheck`, `lint.vue`, `fmt.vue` and `pack.vue`. The removed dedicated config
formats and Vize-specific `extends` are no longer loaded. Preserve reusable
settings as ordinary imported JavaScript objects. Keep TypeScript options in
`tsconfig.json`, including `vueCompilerOptions`.
