---
title: "vue/multi-word-component-names"
description: "Require component names to be multi-word."
---

# `vue/multi-word-component-names`

Require component names to be multi-word.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `nuxt`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

The filename is the finding. The configured exact Item exemption does not allow another single-word name.

## Bad

Thing.vue is a single-word name and is not in the configured ignores list.

`Thing.vue`

```vue
<template><p>Item</p></template>
```

## Good

Item.vue is permitted by the exact configured Item exemption.

`Item.vue`

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
    },
    "ruleOptions": {
      "vue/multi-word-component-names": {
        "ignores": [
          "Item"
        ]
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L41)
