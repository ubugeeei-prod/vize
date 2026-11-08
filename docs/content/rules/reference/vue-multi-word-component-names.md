---
title: "vue/multi-word-component-names"
description: "Require component names to be multi-word."
---

# `vue/multi-word-component-names`

Require component names to be multi-word.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

The filename is the finding. Rename the same component; changing a child tag does not fix it.

## Bad

Item.vue gives the component a single-word name.

`Item.vue`

```vue
<template><p>Item</p></template>
```

## Good

TodoItem.vue gives the same template a multi-word component name.

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/multi-word-component-names": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L41)
