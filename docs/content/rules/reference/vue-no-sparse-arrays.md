---
title: "vue/no-sparse-arrays"
description: "Disallow sparse arrays in template expressions."
---

# `vue/no-sparse-arrays`

Disallow sparse arrays in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The missing second element creates a sparse array.

```vue annotate="remove:2"
<template>
  <p>{{ [1,, 3] }}</p>
</template>
```

## Good

Every array position has an explicit value.

```vue annotate="add:2"
<template>
  <p>{{ [1, 2, 3] }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-sparse-arrays": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L146)
