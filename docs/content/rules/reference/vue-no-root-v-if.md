---
title: "vue/no-root-v-if"
description: "Disallow v-if on the single root element of a template."
---

# `vue/no-root-v-if`

Disallow v-if on the single root element of a template.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The component root itself appears and disappears under v-if.

```vue annotate="remove:2"
<template>
  <div v-if="show">content</div>
</template>
```

## Good

A stable outer div remains the root while the nested paragraph carries the visibility condition.

```vue annotate="add:2,3,4"
<template>
  <div>
    <p v-if="show">content</p>
  </div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-root-v-if": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L41)
