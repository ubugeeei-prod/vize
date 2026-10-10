---
title: "vue/no-root-v-if"
---

# `vue/no-root-v-if`

Disallow v-if on the single root element of a template

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
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-root-v-if": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The component root itself appears and disappears under v-if.

```vue
<template>
  <div v-if="show">content</div>
</template>
```

## Good

A stable outer div remains the root while the nested paragraph carries the visibility condition.

```vue
<template>
  <div>
    <p v-if="show">content</p>
  </div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [All rules](../all.md)
