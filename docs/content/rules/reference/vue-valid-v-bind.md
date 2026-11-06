---
title: "vue/valid-v-bind"
description: "Enforce valid `v-bind` directives."
---

# `vue/valid-v-bind`

Enforce valid `v-bind` directives.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The bare `v-bind` has no object expression, and the empty argument form has no attribute name.

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

## Good

Provide an attribute and expression, bind an object, or use Vue 3.4+ same-name shorthand such as `:loading`.

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-bind": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30)
