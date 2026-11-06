---
title: "vue/no-deprecated-inline-template"
description: "Disallow the deprecated `inline-template` attribute."
---

# `vue/no-deprecated-inline-template`

Disallow the deprecated `inline-template` attribute.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Card uses the deprecated inline-template attribute for its supplied content.

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

## Good

The same content is passed normally without the inline-template attribute.

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-inline-template": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20)
