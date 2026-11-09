---
title: "vue/no-shadow-native-events"
description: "Disallow component events that shadow native events."
---

# `vue/no-shadow-native-events`

Disallow component events that shadow native events.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The component declares `click`, which collides with a native event.

```vue annotate="remove:2"
<script setup>
const emit = defineEmits(['click'])
</script>
<template />
```

## Good

A component-specific `activate` event avoids the collision.

```vue annotate="add:2"
<script setup>
const emit = defineEmits(['activate'])
</script>
<template />
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-shadow-native-events": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L52)
