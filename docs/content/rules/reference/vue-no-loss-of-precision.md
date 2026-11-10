---
title: "vue/no-loss-of-precision"
description: "Disallow numeric literals that lose precision."
---

# `vue/no-loss-of-precision`

Disallow numeric literals that lose precision.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The integer cannot be represented precisely as a JavaScript number.

```vue annotate="remove:2"
<template>
  <p>{{ 9007199254740993 }}</p>
</template>
```

## Good

A precomputed precise binding avoids an imprecise template literal.

```vue annotate="add:2"
<template>
  <p>{{ preciseValue }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-loss-of-precision": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_loss_of_precision.rs#L12)
