---
title: "vue/camelcase"
description: "Require camelCase identifiers in template expressions."
---

# `vue/camelcase`

Require camelCase identifiers in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

`user_name` contains an internal underscore.

```vue annotate="remove:2"
<template>
  <p>{{ user_name }}</p>
</template>
```

## Good

`userName` follows camelCase.

```vue annotate="add:2"
<template>
  <p>{{ userName }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/camelcase": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/camelcase.rs#L10)
