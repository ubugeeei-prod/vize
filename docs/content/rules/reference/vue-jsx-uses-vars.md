---
title: "vue/jsx-uses-vars"
description: "Treat variables used in JSX element names as used."
---

# `vue/jsx-uses-vars`

Treat variables used in JSX element names as used.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

Bad diagnostic: `vue/no-unused-components`

This helper does not emit a `vue/jsx-uses-vars` diagnostic. It credits JSX tag bindings when `vue/no-unused-components` decides whether a component is used. Enable both rules for this example. `<MyButton.Part />` credits the root `MyButton`; namespaced tags and `this` members do not supply an imported component binding.

## Bad

`MyButton` is imported but neither the template nor the JSX reads it, so the companion `vue/no-unused-components` rule reports it.

```vue annotate="remove:3"
<script setup lang="jsx">
import MyButton from "./MyButton.vue";
const view = () => <div />;
</script>
<template><div /></template>
```

## Good

The JSX tag reads the imported `MyButton`; the helper credits that use and the companion rule no longer reports an unused component.

```vue annotate="add:3"
<script setup lang="jsx">
import MyButton from "./MyButton.vue";
const view = () => <MyButton />;
</script>
<template><div /></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/jsx-uses-vars": "error",
        "vue/no-unused-components": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/jsx_uses_vars.rs#L35)
