---
title: "vue/no-multi-spaces"
---

# `vue/no-multi-spaces`

Disallow multiple consecutive spaces

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multi-spaces": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

```vue
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

## Good

```vue
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [All rules](../all.md)
