---
title: "vue/no-empty-pattern"
description: "Disallow empty destructuring patterns in template expressions."
---

# `vue/no-empty-pattern`

Disallow empty destructuring patterns in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The callback destructures an empty object pattern.

```vue annotate="remove:2"
<template>
  <p>{{ items.map(({}) => 1) }}</p>
</template>
```

## Good

The callback names the property it consumes.

```vue annotate="add:2"
<template>
  <p>{{ items.map(({ id }) => id) }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-empty-pattern": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_empty_pattern.rs#L10)
