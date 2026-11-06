---
title: "vue/valid-v-else"
description: "Enforce valid `v-else` directives."
---

# `vue/valid-v-else`

Enforce valid `v-else` directives.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The examples give `v-else` an expression, combine it with `v-if`, or omit its adjacent preceding conditional branch.

```vue annotate="remove:2,3"
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

## Good

Place bare `v-else` immediately after the corresponding `v-if` branch.

```vue annotate="add:2"
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-else": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L31)
