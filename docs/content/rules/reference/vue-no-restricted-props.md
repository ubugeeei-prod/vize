---
title: "vue/no-restricted-props"
description: "disallow specific props."
---

# `vue/no-restricted-props`

disallow specific props.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The component declares the forbidden third prop.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ third?: number }>();
</script>
<template>
  <p>ok</p>
</template>
```

## Good

The component declares first.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ first?: number }>();
</script>
<template>
  <p>ok</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-props": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-props": [
        "third"
      ]
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_props.rs#L12)
