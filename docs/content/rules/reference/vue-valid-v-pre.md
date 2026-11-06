---
title: "vue/valid-v-pre"
description: "Enforce valid `v-pre` directives."
---

# `vue/valid-v-pre`

Enforce valid `v-pre` directives.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

`v-pre` has an argument, modifier, or value, although this directive is a value-free marker that skips compilation.

```vue annotate="remove:2,3,4"
<template>
<div v-pre:arg></div>
<div v-pre.mod></div>
<div v-pre="foo"></div>
</template>
```

## Good

Bare `v-pre` leaves the subtree uncompiled, so its mustaches render as written.

```vue annotate="add:2"
<template>
<div v-pre>{{ raw }}</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-pre": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_pre.rs#L30)
