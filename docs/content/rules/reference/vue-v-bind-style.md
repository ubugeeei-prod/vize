---
title: "vue/v-bind-style"
---

# `vue/v-bind-style`

Enforce `v-bind` directive style

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
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
        "vue/v-bind-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`v-bind:class` uses the long form where the configured binding style requires the colon shorthand.

```vue
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

## Good

`:class` retains the same expression with the required shorthand; this rule concerns spelling rather than the value's type.

```vue
<template>
  <div :class="panelClass"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [All rules](../all.md)
