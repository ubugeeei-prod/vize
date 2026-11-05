---
title: "vue/no-v-text"
description: "Disallow the v-text directive; prefer mustache interpolation."
---

# `vue/no-v-text`

Disallow the v-text directive; prefer mustache interpolation.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The div's content is supplied through the v-text directive.

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

## Good

Mustache interpolation expresses the same text binding directly in the element content.

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
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
        "vue/no-v-text": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31)
