---
title: "vue/no-deprecated-router-link-tag-prop"
description: "Disallow the `tag` prop on <router-link>."
---

# `vue/no-deprecated-router-link-tag-prop`

Disallow the `tag` prop on &lt;router-link&gt;.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

RouterLink uses the removed tag prop to request a button element.

```vue annotate="remove:2"
<template>
  <router-link to="/home" tag="button">Home</router-link>
</template>
```

## Good

The slot provides navigate to an explicitly authored button.

```vue annotate="add:2,3,4"
<template>
  <router-link to="/home" v-slot="{ navigate }">
    <button @click="navigate">Home</button>
  </router-link>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-router-link-tag-prop": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L38)
