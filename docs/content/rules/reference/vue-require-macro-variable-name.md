---
title: "vue/require-macro-variable-name"
description: "Require conventional macro variable names."
---

# `vue/require-macro-variable-name`

Require conventional macro variable names.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The `defineProps` result is bound to `p`.

```vue annotate="remove:2"
<script setup>
const p = defineProps({ value: String })
</script>
<template />
```

## Good

The conventional `props` name identifies the macro result.

```vue annotate="add:2"
<script setup>
const props = defineProps({ value: String })
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
      "vue/require-macro-variable-name": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L56)
