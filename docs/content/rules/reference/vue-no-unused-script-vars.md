---
title: "vue/no-unused-script-vars"
description: "Disallow unused script variables, imports, and parameters."
---

# `vue/no-unused-script-vars`

Disallow unused script variables, imports, and parameters.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The script setup declares unused, and neither the script nor the template reads it.

```vue annotate="remove:4"
<script setup>
const unused = 1;
</script>
<template><p>Welcome</p></template>
```

## Good

The template interpolates unused, so the declared binding is read.

```vue annotate="add:4"
<script setup>
const unused = 1;
</script>
<template><p>{{ unused }}</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-script-vars": "warn"
    },
    "ruleOptions": {
      "vue/no-unused-script-vars": {
        "varsIgnorePattern": "^_",
        "argsIgnorePattern": "^_"
      }
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/script_unused/no_unused_script_vars.rs#L24)
