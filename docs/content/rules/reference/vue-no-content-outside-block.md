---
title: "vue/no-content-outside-block"
description: "Disallow content outside SFC top-level blocks."
---

# `vue/no-content-outside-block`

Disallow content outside SFC top-level blocks.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The leading text is outside every SFC top-level block.

```vue annotate="remove:1,2"
unexpected
<template><div /></template>
```

## Good

All authored content is inside the template block.

```vue annotate="add:1"
<template><div>expected</div></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-content-outside-block": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/content.rs#L8)
