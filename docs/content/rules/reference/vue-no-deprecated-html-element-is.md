---
title: "vue/no-deprecated-html-element-is"
description: "Disallow the `is` attribute on native HTML elements."
---

# `vue/no-deprecated-html-element-is`

Disallow the `is` attribute on native HTML elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

A native div uses the old unprefixed is attribute to request a Vue component.

```vue annotate="remove:2"
<template>
  <div is="MyComponent" />
</template>
```

## Good

A dynamic component uses :is; the native-element spelling explicitly uses the vue: prefix.

```vue annotate="add:2,3"
<template>
  <component :is="MyComponent" />
  <div is="vue:MyComponent" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-html-element-is": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L41)
