---
title: "vue/next-tick-style"
description: "Enforce Promise-style nextTick usage."
---

# `vue/next-tick-style`

Enforce Promise-style nextTick usage.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

A callback is passed to `nextTick`.

```vue annotate="remove:3"
<script setup>
import { nextTick } from "vue"
nextTick(() => focusInput())
</script>
<template />
```

## Good

The code awaits the Promise before continuing.

```vue annotate="add:3,4"
<script setup>
import { nextTick } from "vue"
await nextTick()
focusInput()
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
      "vue/next-tick-style": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/next_tick_style.rs#L12)
