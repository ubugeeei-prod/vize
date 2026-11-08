---
title: "vue/prefer-true-attribute-shorthand"
description: "Prefer the shorthand for a boolean attribute bound to `true`."
---

# `vue/prefer-true-attribute-shorthand`

Prefer the shorthand for a boolean attribute bound to `true`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

A native boolean disabled attribute binds the constant true value.

```vue annotate="remove:2"
<template>
<MyComponent :visible="true" />
</template>
```

## Good

The native attribute uses its boolean shorthand. False bindings and component props retain their explicit values.

```vue annotate="add:2,3,4"
<template>
<MyComponent visible />
<MyComponent :visible="false" />
<MyComponent :visible="isVisible" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-true-attribute-shorthand": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L28)
