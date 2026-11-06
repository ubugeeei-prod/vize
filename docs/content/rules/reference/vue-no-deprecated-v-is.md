---
title: "vue/no-deprecated-v-is"
description: "Disallow the deprecated `v-is` directive."
---

# `vue/no-deprecated-v-is`

Disallow the deprecated `v-is` directive.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

`v-is` was deprecated in Vue 3.1; the example still uses it to swap the rendered component.

```vue annotate="remove:2"
<template>
<tr v-is="'MyRow'"></tr>
</template>
```

## Good

The `is` attribute with the `vue:` prefix, or `<component :is>`, selects the component instead.

```vue annotate="add:2,3"
<template>
<tr is="vue:MyRow"></tr>
<component :is="MyRow" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-v-is": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_is.rs#L27)
