---
title: "vue/attribute-order"
---

# `vue/attribute-order`

Enforce a consistent order of attributes

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/attribute-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The event handler appears before the structural v-if directive and ordinary id attribute.

```vue
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

## Good

v-if comes first, followed by id and then the event handler, following the rule ordering.

```vue
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [All rules](../all.md)
