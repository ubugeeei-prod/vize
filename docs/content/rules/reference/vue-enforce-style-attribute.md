---
title: "vue/enforce-style-attribute"
description: "Enforce allowed style-block attributes."
---

# `vue/enforce-style-attribute`

Enforce allowed style-block attributes.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The module style mode is outside the configured allow list.

```vue annotate="remove:2"
<template><div /></template>
<style module>.card { color: red }</style>
```

## Good

The scoped style mode is allowed by the project policy.

```vue annotate="add:2"
<template><div /></template>
<style scoped>.card { color: red }</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/enforce-style-attribute": "warn"
    },
    "ruleOptions": {
      "vue/enforce-style-attribute": {
        "vue/enforce-style-attribute": {
          "allow": [
            "scoped"
          ]
        }
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L32)
