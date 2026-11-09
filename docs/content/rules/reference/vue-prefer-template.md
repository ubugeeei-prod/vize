---
title: "vue/prefer-template"
description: "Prefer template literals in template expressions."
---

# `vue/prefer-template`

Prefer template literals in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

String concatenation mixes a literal with `name`.

```vue annotate="remove:2"
<template>
  <p>{{ "Hello " + name }}</p>
</template>
```

## Good

A template literal keeps interpolation in one string.

```vue annotate="add:2"
<template>
  <p>{{ `Hello ${name}` }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-template": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L167)
