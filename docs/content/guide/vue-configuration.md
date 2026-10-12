---
title: Shared Vue configuration
---

# Shared Vue configuration

Keep native Vue settings in the `vue` section of `vite.config.ts`. The compiler,
CLI and editor use the same native option names. Start with the
[Vite+ integration](./vite-plus.md) to run build, check, lint and format tasks.

This API is prepared for the next minor release. Existing published versions
retain the configuration described in [Configuration](./configuration.md).

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
It also accepts `version` and `entries`. Compiler and formatter sections contain
options; they do not have an `enabled` member. Typechecker, linter and language
server sections retain their native `enabled` flag. Unknown canonical members
produce an error.

## Keep an existing Vite+ configuration

```ts
import { defineConfig } from "vite-plus";
import { withVue } from "@vizejs/vite-plugin/vite-plus";
import vpConfig from "./vite.base.ts";

export default withVue(
  { compiler: { sourceMap: true }, formatter: { singleQuote: true } },
  defineConfig(vpConfig),
);
```

`withVue({}, defineConfig(vpConfig))` is the minimal form. The first argument is
Vue settings; the second is the existing Vite+ export. Objects, promises,
configuration functions and async functions are supported. The existing export
receives Vite's environment when it is evaluated. Existing plugins, hooks and
Vite+ fields stay with that export, and Vite+ plugin injection runs once.

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

## Migrate existing settings

Move native settings from the top-level `vize` section to `vue`; use
`typeChecker`, `linter` and `formatter` instead of Vite+ task spellings such as
`typecheck`, `lint.vize` and `fmt.vize`. Existing legacy ingress remains accepted.
The former config-first `withVue(config)` alias becomes options-first; use
`defineConfig(config)` or the retained `withVize(config)` for the old call shape.
Explicit canonical leaves take precedence over legacy settings in the same
selected Vite source. Unspecified leaves are inherited, arrays are replaced,
and `false` remains `false`.

An existing dedicated `vize.config.*` still wins during native CLI/editor file
discovery. The Vite+ helper uses its selected configuration without mixing in an
unselected dedicated file. To share canonical settings with native commands,
keep them in the selected Vite configuration. See the
[dedicated configuration reference](./configuration-reference.md) for existing
formats and explicit file selection.
