---
title: "vue/valid-v-cloak"
---

# `vue/valid-v-cloak`

Enforce valid `v-cloak` directives

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/valid-v-cloak": "error"
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
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

## Good

```vue
<template>
<div v-cloak></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) · [All rules](../all.md)
