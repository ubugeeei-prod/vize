---
title: "vue/no-v-for-template-key"
description: "Disallow `key` on `<template v-for>`."
---

# `vue/no-v-for-template-key`

Disallow `key` on `<template v-for>`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

This is a Vue 2 compatibility rule and stays opt-in. Vue 2 expects the key on a real child; Vue 3 expects it on `<template v-for>` instead. For the Vue 3 convention, use `vue/no-v-for-template-key-on-child`; do not enable both conventions together.

## Bad

For the Vue 2 contract, a key on `<template v-for>` is ignored because the template does not render a real element.

```vue annotate="remove:1"
<template><ul><template v-for="item in items" :key="item.id"><li>{{ item.label }}</li></template></ul></template>
```

## Good

Put the item's key on the repeated real `li` child, as required by the Vue 2 convention checked here.

```vue annotate="add:1"
<template><ul><template v-for="item in items"><li :key="item.id">{{ item.label }}</li></template></ul></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-v-for-template-key": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key.rs#L33)
