---
title: "vue/prefer-v-model"
description: "Prefer v-model over matching bind and update listeners."
---

# `vue/prefer-v-model`

Prefer v-model over matching bind and update listeners.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The bind and update listener manually mirror one value.

```vue annotate="remove:2"
<template>
  <TextField :model-value="value" @update:model-value="value = $event" />
</template>
```

## Good

`v-model` expresses the same two-way component contract once.

```vue annotate="add:2"
<template>
  <TextField v-model="value" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-v-model": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules/prefer_v_model.rs#L19)
