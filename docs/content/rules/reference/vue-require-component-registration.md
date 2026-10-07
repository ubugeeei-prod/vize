---
title: "vue/require-component-registration"
---

# `vue/require-component-registration`

Require explicit import or registration for components

Default severity: `warning`  
Presets: `opinionated`  
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
        "vue/require-component-registration": "warn"
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
<script setup lang="ts">
// MyButton is never imported.
</script>

<template>
  <MyButton>Save</MyButton>
</template>
```

## Good

```vue
<script setup lang="ts">
import MyButton from "./MyButton.vue";
</script>

<template>
  <MyButton>Save</MyButton>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [All rules](../all.md)
