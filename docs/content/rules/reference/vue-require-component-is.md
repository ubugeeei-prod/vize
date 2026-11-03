---
title: "vue/require-component-is"
description: "Require `v-bind:is` on `<component>` elements."
---

# `vue/require-component-is`

Require `v-bind:is` on `<component>` elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The dynamic `<component>` has no `is` target, so Vue cannot choose a component to render.

```vue annotate="remove:2"
<template>
  <component />
</template>
```

## Good

`:is="currentComponent"` supplies the component selection; the binding may change at runtime.

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
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
        "vue/require-component-is": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27)
