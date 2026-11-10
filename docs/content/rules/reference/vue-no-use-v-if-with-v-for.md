---
title: "vue/no-use-v-if-with-v-for"
description: "Disallow using `v-if` on the same element as `v-for`."
---

# `vue/no-use-v-if-with-v-for`

Disallow using `v-if` on the same element as `v-for`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The same list element combines v-if and v-for, but the condition does not reference the loop binding.

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="showAll" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## Good

With allowUsingIterationVar enabled, the condition references item from the same v-for.

```vue annotate="add:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-use-v-if-with-v-for": "warn"
    },
    "ruleOptions": {
      "vue/no-use-v-if-with-v-for": {
        "allowUsingIterationVar": true
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35)
