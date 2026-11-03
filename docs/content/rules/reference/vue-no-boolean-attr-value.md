---
title: "vue/no-boolean-attr-value"
description: "Disallow explicit values for boolean HTML attributes."
---

# `vue/no-boolean-attr-value`

Disallow explicit values for boolean HTML attributes.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Yes | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The boolean disabled and checked attributes redundantly contain string values.

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

## Good

The presence of each boolean attribute expresses the same enabled state without a value.

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
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
        "vue/no-boolean-attr-value": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36)
