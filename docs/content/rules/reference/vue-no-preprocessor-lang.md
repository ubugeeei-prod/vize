---
title: "vue/no-preprocessor-lang"
---

# `vue/no-preprocessor-lang`

Discourage CSS preprocessor usage in favor of modern CSS

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-preprocessor-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The style block selects SCSS through lang, so the no-preprocessor policy reports the authored block.

```vue
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

## Good

The same declarations use plain CSS without a preprocessor lang.

```vue
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L11) · [All rules](../all.md)
