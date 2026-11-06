---
title: "vue/no-array-index-key"
description: "Disallow using the v-for index variable directly as the :key."
---

# `vue/no-array-index-key`

Disallow using the v-for index variable directly as the :key.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The list key is its current index, so item identity changes when the list is reordered.

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

## Good

The key comes from item.id, preserving the identity of each item across position changes.

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-array-index-key": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32)
