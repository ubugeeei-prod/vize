---
title: Oxlint Plugin
---

# Oxlint Plugin

`oxlint-plugin-vize` lets Oxlint execute Vize Patina diagnostics through Oxlint's JS plugin system.
Use it when you want Oxlint's Rust-native JS and TS rules together with Vize's Vue-aware
diagnostics in one run.

For the native lint and type-checking pipeline outside Oxlint, see
[Static Analysis](./static-analysis.md).

> [!IMPORTANT]
> The package is available on npm, but the integration is still early. For human-readable terminal
> output, prefer `oxlint-vize -f stylish` while original SFC range fidelity continues to improve.

## Installation

Install `vp` once from the [Vite+ install guide](https://viteplus.dev/guide/install), then add the packages:

```bash
vp install -D oxlint oxlint-plugin-vize
```

`oxlint-plugin-vize` resolves the matching Vize native binding through optional dependencies, so
most users do not need to install `@vizejs/native` separately.

## Which File To Configure

| Command                 | Reads                                |
| ----------------------- | ------------------------------------ |
| `vp lint`, `vp check`   | the `lint` block in `vite.config.ts` |
| `oxlint`, `oxlint-vize` | `.oxlintrc.json` (or `-c <path>`)    |

> [!WARNING]
> Vite+ never reads `.oxlintrc.json`. A `.oxlintrc.json` carrying `jsPlugins` and `vize/*` rules
> looks configured, but `vp lint` ignores the file, so Oxlint never sees a `vize/*` rule id and
> reports **zero** Vize diagnostics while exiting `0`. `vp lint --init` does not migrate an existing
> `.oxlintrc.json` either: it writes a fresh `lint` block and leaves the old file in place.

[`vize init`](./init.md) picks the right file for you: it detects whether your lint command is
`vp lint` or `oxlint` and writes the configuration that command reads, or writes both when both are
in use. It also refuses to write anything rather than fall back to a file your lint command ignores.

## Basic Usage With `vp lint`

`createVizeLintConfig()` returns the whole Vite+ `lint` block, so the `jsPlugins` entry that loads
the bridge cannot go missing:

```ts
// vite.config.ts
import { defineConfig } from "vite-plus";
import { createVizeLintConfig } from "oxlint-plugin-vize";

export default defineConfig({
  lint: createVizeLintConfig({
    preset: "essential",
    rules: {
      "no-console": "warn",
    },
    settings: {
      helpLevel: "short",
    },
  }),
});
```

`preset` drives both the emitted rule map and `settings.vize.preset`. Keeping them in lockstep
matters because the bridge silently drops any `vize/*` rule outside the active preset, so a rule
listed under a mismatched preset reports nothing at all. `createVizeLintConfig` throws for that
case, and for unknown `vize/*` ids, rather than leaving you with a config that looks enabled and
stays silent.

- `preset: "incremental"` runs only the rules you list.
- `preset: "all"` runs every bundle at once.
- `plugins` keeps the rest of your built-in Oxlint plugins. They are merged with `vue`, never
  replaced, because narrowing the list would silently drop everything those plugins report. A
  `create-vue` project passes `["eslint", "typescript", "unicorn", "oxc"]`.
- Spread the result (`{ ...createVizeLintConfig(), ignorePatterns: ["dist/**"] }`) to merge it into
  an existing `lint` block.

## Basic Usage With `oxlint` And `oxlint-vize`

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "helpLevel": "short"
    }
  },
  "rules": {
    "eqeqeq": "error",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "no-console": "warn"
  }
}
```

If you use a JS or TS Oxlint config, the package also exports preset rule maps:

```js
import { configs } from "oxlint-plugin-vize";

export default {
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      helpLevel: "short",
      preset: "opinionated",
      typeAware: true,
    },
  },
  rules: configs.opinionatedWithTypeAware,
};
```

Available preset exports include:

- `configs.recommended`
- `configs.essential`
- `configs.opinionated`
- `configs.nuxt`
- `configs.all`
- `configs.recommendedWithTypeAware`
- `configs.ecosystemWithTypeAware`
- `configs.opinionatedWithTypeAware`

## Recommended Command

```bash
vp exec oxlint-vize -c .oxlintrc.json -f stylish src
```

`oxlint-vize` is a thin wrapper around `oxlint` that smooths over scriptless `.vue` edge cases
while upstream JS plugin coverage continues improving.

## Settings

Settings are passed through `settings.vize`:

```json
{
  "settings": {
    "vize": {
      "locale": "ja",
      "preset": "general-recommended",
      "helpLevel": "short",
      "typeAware": true
    }
  }
}
```

- `locale` controls the diagnostic language.
- `preset` accepts `"general-recommended"`, `"essential"`, `"ecosystem"`, `"incremental"`, `"opinionated"`, or `"nuxt"`.
- `preset` defaults to `"general-recommended"`.
- `incremental` runs only the rules you explicitly configure.
- `helpLevel` accepts `"full"`, `"short"`, or `"none"`.
- `typeAware: true` enables Corsa-backed `vize/type/*` rules during shared Patina passes.
- `corsaPath` selects the Corsa or `tsgo` executable for type-aware linting.
- `showHelp` and `settings.patina` are still accepted for backward compatibility.

## Current Limitations

- Raw `oxlint` can still miss some `.vue` files without `<script>` or `<script setup>`. Use
  `oxlint-vize` if your project includes template-only SFCs.
- Oxlint JS plugins still anchor ranges to the extracted script program, so template and style
  diagnostics do not yet preserve original SFC ranges in every formatter.
- `stylish` is currently the best human-readable formatter for mixed Oxlint + Vize output. JSON and
  other machine-readable formats should be treated as best-effort for original template/style
  positions.
- Type-aware rule exports are experimental. Use a `*WithTypeAware` config and set
  `settings.vize.typeAware: true` when you want the shared full-file pass to run those rules eagerly.

## Local Development

```bash
nix develop
vp install --frozen-lockfile
vp run --filter './npm/native' build
vp run --filter './npm/oxlint' build
```
