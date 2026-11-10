---
title: "vue/dot-notation"
description: "Prefer dot notation in template expressions."
---

# `vue/dot-notation`

Prefer dot notation in template expressions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The static `name` property is accessed with brackets.

```vue annotate="remove:2"
<template>
  <p>{{ user["name"] }}</p>
</template>
```

## Good

Dot notation expresses the same static property directly.

```vue annotate="add:2"
<template>
  <p>{{ user.name }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/dot-notation": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/dot_notation.rs#L10)
