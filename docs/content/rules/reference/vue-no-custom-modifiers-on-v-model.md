---
title: "vue/no-custom-modifiers-on-v-model"
description: "Disallow custom modifiers on `v-model` used on a component."
---

# `vue/no-custom-modifiers-on-v-model`

Disallow custom modifiers on `v-model` used on a component.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

This opt-in rule enforces a Vue 2 compatibility contract on component `v-model`. Only `lazy`, `number`, and `trim` are allowed modifiers. Vue 3 components can implement custom modifiers, so Vue 3 presets do not enable this restriction. Native elements are outside this rule's check.

## Bad

The component's `.capitalize` modifier is custom and therefore violates this Vue 2 compatibility restriction.

```vue annotate="remove:1"
<template><MyField v-model.capitalize="value" /></template>
```

## Good

Use the component's ordinary `v-model` contract without a custom modifier; implement any required capitalization explicitly.

```vue annotate="add:1"
<template><MyField v-model="value" /></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-custom-modifiers-on-v-model": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_custom_modifiers_on_v_model.rs#L23)
