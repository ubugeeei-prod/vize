---
title: "vue/valid-v-is"
description: "Enforce valid `v-is` directives."
---

# `vue/valid-v-is`

Enforce valid `v-is` directives.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

`v-is` has an argument or modifier, lacks its value, or sits on a component instead of a native HTML element.

```vue annotate="remove:2,3,4,5"
<template>
<tr v-is:arg="'MyRow'"></tr>
<tr v-is.mod="'MyRow'"></tr>
<tr v-is></tr>
<MyComponent v-is="'MyRow'" />
</template>
```

## Good

`v-is` with a value on a native element such as `<tr>` names the component to render in its place.

```vue annotate="add:2"
<template>
<tr v-is="'MyRow'"></tr>
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
        "vue/valid-v-is": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_is.rs#L30)
