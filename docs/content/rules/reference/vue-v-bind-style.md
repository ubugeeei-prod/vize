---
title: "vue/v-bind-style"
description: "Enforce `v-bind` directive style."
---

# `vue/v-bind-style`

Enforce `v-bind` directive style.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

`v-bind:class` uses the long form where the configured binding style requires the colon shorthand.

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

## Good

`:class` retains the same expression with the required shorthand; this rule concerns spelling rather than the value's type.

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-bind-style": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L28)
