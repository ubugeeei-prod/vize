---
title: "css/no-v-bind-performance"
---

# `css/no-v-bind-performance`

Warn about performance cost of CSS v-bind()

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
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
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

## Good

```vue
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L21) · [All rules](../all.md)
