---
title: "vue/no-child-content"
description: "Disallow child content when using v-html or v-text."
---

# `vue/no-child-content`

Disallow child content when using v-html or v-text.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

v-text replaces the paragraph content, so the authored fallback text cannot survive that directive.

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

## Good

Removing the child text leaves v-text as the single source of paragraph content.

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-child-content": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L32)
