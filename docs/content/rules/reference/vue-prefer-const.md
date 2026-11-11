---
title: "vue/prefer-const"
description: "Require const for bindings the script and template never reassign."
---

# `vue/prefer-const`

Require const for bindings the script and template never reassign.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The template reads label, and neither the script nor the template reassigns it, so let is wider than the binding needs.

```vue annotate="remove:2"
<script setup>
let label = "Hello";
</script>
<template><p>{{ label }}</p></template>
```

## Good

const matches a binding that the script and the template only read.

```vue annotate="add:2"
<script setup>
const label = "Hello";
</script>
<template><p>{{ label }}</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-const": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/script_unused/prefer_const.rs#L16)
