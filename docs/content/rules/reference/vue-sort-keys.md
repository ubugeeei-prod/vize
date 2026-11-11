---
title: "vue/sort-keys"
description: "Require sorted object keys, compatible with component option order."
---

# `vue/sort-keys`

Require sorted object keys, compatible with component option order.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The nested method keys are not in ascending order.

```vue annotate="remove:2"
<script>
export default { methods: { zebra() {}, alpha() {} } }
</script>
<template />
```

## Good

The nested method keys follow the configured order.

```vue annotate="add:2"
<script>
export default { methods: { alpha() {}, zebra() {} } }
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
      "vue/sort-keys": "warn"
    },
    "ruleOptions": {
      "vue/sort-keys": {
        "order": "asc",
        "caseSensitive": true,
        "natural": false
      }
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/sort_keys.rs#L22)
