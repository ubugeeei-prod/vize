---
title: "vue/eqeqeq"
description: "Require strict equality in template expressions."
---

# `vue/eqeqeq`

Require strict equality in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Loose equality can coerce either operand.

```vue annotate="remove:2"
<template>
  <p>{{ value == null }}</p>
</template>
```

## Good

Strict equality compares without coercion.

```vue annotate="add:2"
<template>
  <p>{{ value === null }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/eqeqeq": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L90)
