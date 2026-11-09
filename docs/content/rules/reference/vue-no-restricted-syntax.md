---
title: "vue/no-restricted-syntax"
description: "Disallow configured syntax in template expressions."
---

# `vue/no-restricted-syntax`

Disallow configured syntax in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The configured `CallExpression` appears in the template.

```vue annotate="remove:2"
<template>
  <p>{{ format(value) }}</p>
</template>
```

## Good

The template reads a prepared binding instead of calling a function.

```vue annotate="add:2"
<template>
  <p>{{ formattedValue }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-syntax": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-syntax": {
        "vue/no-restricted-syntax": {
          "selectors": [
            "CallExpression"
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/restricted.rs#L16)
