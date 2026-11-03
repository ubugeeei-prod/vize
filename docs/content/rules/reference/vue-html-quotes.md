---
title: "vue/html-quotes"
description: "Enforce quotes style of HTML attributes."
---

# `vue/html-quotes`

Enforce quotes style of HTML attributes.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The attributes use single quotes or no quotes instead of the double-quote convention.

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

## Good

Both ordinary attributes and directive expressions use double quotes.

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
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
        "vue/html-quotes": "warn"
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53)
