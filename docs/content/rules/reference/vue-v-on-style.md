---
title: "vue/v-on-style"
description: "Enforce `v-on` directive style."
---

# `vue/v-on-style`

Enforce `v-on` directive style.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

`v-on:click` uses the long event-listener form where the rule requires shorthand.

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

## Good

`@click` keeps the same handler while using the configured shorthand.

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
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
        "vue/v-on-style": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28)
