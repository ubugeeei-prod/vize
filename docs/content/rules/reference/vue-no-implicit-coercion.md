---
title: "vue/no-implicit-coercion"
description: "Disallow implicit coercion in template expressions."
---

# `vue/no-implicit-coercion`

Disallow implicit coercion in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Double negation coerces the value implicitly.

```vue annotate="remove:2"
<template>
  <p>{{ !!value }}</p>
</template>
```

## Good

`Boolean` makes the conversion explicit.

```vue annotate="add:2"
<template>
  <p>{{ Boolean(value) }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-implicit-coercion": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_implicit_coercion.rs#L10)
