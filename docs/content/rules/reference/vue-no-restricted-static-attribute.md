---
title: "vue/no-restricted-static-attribute"
description: "disallow specific attribute."
---

# `vue/no-restricted-static-attribute`

disallow specific attribute.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The button has the forbidden data-legacy attribute.

```vue annotate="remove:2"
<template>
  <button type="button" data-legacy="x">x</button>
</template>
```

## Good

The button keeps only type.

```vue annotate="add:2"
<template>
  <button type="button">x</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-static-attribute": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-static-attribute": [
        "data-legacy"
      ]
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_static_attribute.rs#L13)
