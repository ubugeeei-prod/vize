---
title: "vue/no-static-inline-styles"
description: "Disallow static inline style attributes."
---

# `vue/no-static-inline-styles`

Disallow static inline style attributes.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The paragraph carries the constant color declaration in its style attribute.

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

## Good

A notice class and scoped stylesheet hold the constant color outside the template attribute.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-static-inline-styles": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L19)
