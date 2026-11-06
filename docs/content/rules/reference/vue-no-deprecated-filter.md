---
title: "vue/no-deprecated-filter"
description: "Disallow deprecated Vue 2 filter syntax using the pipe operator."
---

# `vue/no-deprecated-filter`

Disallow deprecated Vue 2 filter syntax using the pipe operator.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The pipe uses the removed Vue filter syntax to apply capitalize.

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

## Good

Calling capitalize(message) applies the transformation as an ordinary expression.

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-filter": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L56)
