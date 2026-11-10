---
title: "vue/no-define-model-default"
description: "Disallow default values in defineModel."
---

# `vue/no-define-model-default`

Disallow default values in defineModel.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The child model declares a default that can desynchronize the parent.

```vue annotate="remove:1,2"
<script setup>
const model = defineModel({ default: "draft" })
</script>
<template />
```

## Good

The parent supplies the initial model value instead.

```vue annotate="add:1,2"
<script setup lang="ts">
const model = defineModel<string>()
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
      "vue/no-define-model-default": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/no_define_model_default.rs#L11)
