---
title: "vue/no-use-v-else-with-v-for"
description: "Disallow using `v-else-if` or `v-else` on the same element as `v-for`."
---

# `vue/no-use-v-else-with-v-for`

Disallow using `v-else-if` or `v-else` on the same element as `v-for`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The else branch and v-for iteration are attached to the same paragraph. Vue accepts this syntax; the warning recommends separating the two control-flow responsibilities. HappyPath and Ecosystem now include it alongside Opinionated and Nuxt.

```vue annotate="remove:3"
<template>
<p v-if="ready">Ready</p>
<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

## Good

A separate template owns v-else, and its child paragraph owns v-for.

```vue annotate="add:3"
<template>
<p v-if="ready">Ready</p>
<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-use-v-else-with-v-for": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19)
