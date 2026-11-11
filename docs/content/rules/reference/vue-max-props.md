---
title: "vue/max-props"
description: "enforce maximum number of props in Vue component."
---

# `vue/max-props`

enforce maximum number of props in Vue component.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The component declares three props, above the configured maximum.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ a: number; b: number; c: number }>();
</script>
<template>
  <p>props</p>
</template>
```

## Good

The component declares one prop.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ a: number }>();
</script>
<template>
  <p>props</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-props": "warn"
    },
    "ruleOptions": {
      "vue/max-props": {
        "maxProps": 2
      }
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/max_props.rs#L16)
