---
title: Oxlint Plugin
---

# Oxlint Plugin

`oxlint-plugin-vize` lets Oxlint execute Vize Patina diagnostics through Oxlint's JS plugin system.
Use it when you want Oxlint's Rust-native JS and TS rules together with Vize's Vue-aware
diagnostics in one run.

> [!IMPORTANT]
> The package is available on npm, but the integration is still early. For human-readable terminal
> output, prefer `oxlint-vize -f stylish` while original SFC range fidelity continues to improve.

## Installation

```bash
pnpm add -D oxlint oxlint-plugin-vize
```

`oxlint-plugin-vize` resolves the matching Vize native binding through optional dependencies, so
most users do not need to install `@vizejs/native` separately.

## Basic Usage

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
    },
  },
  rules: configs.opinionated,
};
```

Available preset exports include:

- `configs.recommended`
- `configs.essential`
- `configs.opinionated`
- `configs.nuxt`
- `configs.all`
- `configs.recommendedWithTypeAware`
- `configs.opinionatedWithTypeAware`

## Recommended Command

```bash
pnpm exec oxlint-vize -c .oxlintrc.json -f stylish src
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
      "helpLevel": "short"
    }
  }
}
```

- `locale` controls the diagnostic language.
- `preset` accepts `"general-recommended"`, `"essential"`, `"incremental"`, `"opinionated"`, or `"nuxt"`.
- `preset` defaults to `"general-recommended"`.
- `incremental` runs only the rules you explicitly configure.
- `helpLevel` accepts `"full"`, `"short"`, or `"none"`.
- `showHelp` and `settings.patina` are still accepted for backward compatibility.

## Current Limitations

- Raw `oxlint` can still miss some `.vue` files without `<script>` or `<script setup>`. Use
  `oxlint-vize` if your project includes template-only SFCs.
- Oxlint JS plugins still anchor ranges to the extracted script program, so template and style
  diagnostics do not yet preserve original SFC ranges in every formatter.
- `stylish` is currently the best human-readable formatter for mixed Oxlint + Vize output. JSON and
  other machine-readable formats should be treated as best-effort for original template/style
  positions.

## Local Development

```bash
nix develop
pnpm install --frozen-lockfile
vp run --filter './npm/vize-native' build
vp run --filter './npm/oxlint-plugin-vize' build
```
