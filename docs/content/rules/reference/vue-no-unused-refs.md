---
title: "vue/no-unused-refs"
description: "Report template refs (ref=\"x\") never referenced in <script>."
---

# `vue/no-unused-refs`

Report template refs (ref="x") never referenced in &lt;script&gt;.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The template declares the unused ref name with no corresponding script reference binding.

```vue annotate="remove:1,3"
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

## Good

The inputEl template ref has a same-named ref binding in script setup.

```vue annotate="add:1,3,4"
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-refs": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L67)
