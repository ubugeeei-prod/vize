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

Checks the static name of a rendered slot outlet. The parent-side v-slot and #name provider syntax is outside this rule.

## Bad

The rendered slot outlet uses the hyphenated static name `my-slot`, while the default convention requires camelCase.

```vue annotate="remove:2"
<template>
<MyCard><slot name="my-slot" /></MyCard>
</template>
```

## Good

The outlet's static name uses camelCase. Parent-side `#name` provider syntax is outside this rule.

```vue annotate="add:2"
<template>
<MyCard><slot name="mySlot" /></MyCard>
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
