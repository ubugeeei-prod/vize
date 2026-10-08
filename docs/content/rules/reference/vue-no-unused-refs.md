---
title: "vue/no-unused-refs"
---

# `vue/no-unused-refs`

Report template refs (ref="x") never referenced in &lt;script&gt;

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
        "vue/no-unused-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The template declares the unused ref name with no corresponding script reference binding.

```vue
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

## Good

The inputEl template ref has a same-named ref binding in script setup.

```vue
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) · [All rules](../all.md)
