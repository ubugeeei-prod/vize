---
title: "vue/multi-word-component-names"
---

# `vue/multi-word-component-names`

Require component names to be multi-word

Default severity: `error`  
Presets: `essential`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

The filename is the finding. Rename the same component; changing a child tag does not fix it.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/multi-word-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`Item.vue`

```vue
<template><p>Item</p></template>
```

## Good

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [All rules](../all.md)
