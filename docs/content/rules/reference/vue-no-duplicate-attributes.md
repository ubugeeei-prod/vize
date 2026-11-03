---
title: "vue/no-duplicate-attributes"
description: "Disallow duplicate attributes on the same element."
---

# `vue/no-duplicate-attributes`

Disallow duplicate attributes on the same element.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The same button declares class twice instead of one combined class value.

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

## Good

Both class tokens appear in a single class attribute.

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-duplicate-attributes": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31)
