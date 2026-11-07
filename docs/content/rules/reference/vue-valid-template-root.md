---
title: "vue/valid-template-root"
description: "Enforce valid root content in an SFC `<template>` block."
---

# `vue/valid-template-root`

Enforce valid root content in an SFC `<template>` block.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

A plain nested `<template>` occupies the template root without a directive that gives it a rendering role.

```vue annotate="remove:1"
<template></template>
```

## Good

The `<div>` is a renderable root element. This example does not impose a universal single-root restriction on Vue 3 fragments.

```vue annotate="add:1,2,3"
<template>
  <div>content</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-template-root": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L54)
