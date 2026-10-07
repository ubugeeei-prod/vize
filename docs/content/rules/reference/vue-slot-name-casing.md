---
title: "vue/slot-name-casing"
---

# `vue/slot-name-casing`

Enforce kebab-case for named slots used via v-slot

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
        "vue/slot-name-casing": "warn"
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
<MyComponent>
<template #mySlot>...</template>
<template #my_slot>...</template>
</MyComponent>
```

## Good

```vue
<MyComponent>
<template #my-slot>...</template>
<template #default>...</template>
</MyComponent>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [All rules](../all.md)
