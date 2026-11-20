---
title: "vue/prefer-pug-lang"
description: "Prefer lang=\"pug\" over the legacy jade 1.x template language."
---

# `vue/prefer-pug-lang`

Prefer lang="pug" over the legacy jade 1.x template language.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The template names jade, which Vue renders with the legacy jade 1.x package, and uses the jade-only dashed each.

```vue annotate="remove:1,3"
<template lang="jade">
ul
  - each item in [1, 2]
    li= item
</template>
```

## Good

The template names pug and writes the loop with the pug 3 each keyword.

```vue annotate="add:1,3"
<template lang="pug">
ul
  each item in [1, 2]
    li= item
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-pug-lang": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prefer_pug_lang.rs#L42)
