---
title: "vue/sfc-element-order"
description: "Enforce consistent order of SFC top-level elements."
---

# `vue/sfc-element-order`

Enforce consistent order of SFC top-level elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The style block precedes the script block, contrary to the configured SFC block order.

```vue annotate="remove:2,6,7,8"
<style scoped>
.panel {
  color: red;
}
</style>
<script setup lang="ts">
const label = "Save";
</script>
```

## Good

The blocks follow script → template → style. Projects can choose a different order through this rule's typed option.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts">
const label = "Save";
</script>

<template>
  <p>{{ label }}</p>
</template>

<style scoped>
p {
  color: red;
}
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/sfc-element-order": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50)
