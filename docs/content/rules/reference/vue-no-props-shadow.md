---
title: "vue/no-props-shadow"
description: "Disallow setup declarations that shadow props."
---

# `vue/no-props-shadow`

Disallow setup declarations that shadow props.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The local `value` declaration shadows the prop.

```vue annotate="remove:3"
<script setup>
defineProps({ value: String })
const value = "local"
</script>
<template />
```

## Good

The local binding has a distinct name.

```vue annotate="add:3"
<script setup>
defineProps({ value: String })
const localValue = "local"
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
      "vue/no-props-shadow": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/options.rs#L17)
