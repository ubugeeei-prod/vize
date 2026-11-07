---
title: "vue/slot-name-casing"
description: "Enforce camelCase for rendered slot names."
---

# `vue/slot-name-casing`

Enforce camelCase for rendered slot names.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The named slot `mySlot` uses camelCase where the rule requires a hyphenated name.

```vue annotate="remove:2"
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

## Good

`#my-slot` uses kebab-case. Rename the corresponding slot outlet to the same name.

```vue annotate="add:2"
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/slot-name-casing": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L28)
