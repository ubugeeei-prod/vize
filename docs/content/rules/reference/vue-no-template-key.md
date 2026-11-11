---
title: "vue/no-template-key"
description: "Disallow `key` attribute on `<template>`."
---

# `vue/no-template-key`

Disallow `key` attribute on `<template>`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

A non-loop template wrapper has a key even though it is not the keyed iteration boundary.

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

## Good

The key belongs to a template v-for iteration, where it identifies each repeated fragment.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-key": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L29)
