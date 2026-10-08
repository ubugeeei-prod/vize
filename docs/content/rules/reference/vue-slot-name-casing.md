---
title: "vue/slot-name-casing"
---

# `vue/slot-name-casing`

Enforce kebab-case for named slots used via v-slot

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

The named slot `mySlot` uses camelCase where the rule requires a hyphenated name.

```vue
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

## Good

`#my-slot` uses kebab-case. Rename the corresponding slot outlet to the same name.

```vue
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [All rules](../all.md)
