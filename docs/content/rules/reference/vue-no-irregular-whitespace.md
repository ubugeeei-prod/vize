---
title: "vue/no-irregular-whitespace"
description: "Disallow irregular whitespace in template expressions."
---

# `vue/no-irregular-whitespace`

Disallow irregular whitespace in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The expression contains a non-breaking space.

```vue annotate="remove:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## Good

The expression uses ordinary source whitespace.

```vue annotate="add:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-irregular-whitespace": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_irregular_whitespace.rs#L12)
