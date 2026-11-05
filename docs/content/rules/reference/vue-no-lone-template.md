---
title: "vue/no-lone-template"
description: "Disallow unnecessary `<template>` elements."
---

# `vue/no-lone-template`

Disallow unnecessary `<template>` elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The inner template has no directive or slot role that gives it a structural purpose.

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

## Good

Removing the unnecessary wrapper leaves the paragraph directly inside div.

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
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
        "vue/no-lone-template": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32)
