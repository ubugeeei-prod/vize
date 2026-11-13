---
title: "vue/prefer-single-event-payload"
description: "Enforce passing a single argument to custom event emissions."
---

# `vue/prefer-single-event-payload`

Enforce passing a single argument to custom event emissions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The move event declares and emits two payload values, so an inline listener only receives x.

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits<{ move: [x: number, y: number] }>()
emit("move", 1, 2)
</script>
<template />
```

## Good

The move event carries one object, so every listener receives both coordinates.

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits<{ move: [point: { x: number; y: number }] }>()
emit("move", { x: 1, y: 2 })
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
      "vue/prefer-single-event-payload": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/event_payload.rs#L34)
