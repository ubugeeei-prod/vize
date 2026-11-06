---
title: "vue/no-v-html"
description: "Warn against v-html to prevent XSS vulnerabilities."
---

# `vue/no-v-html`

Warn against v-html to prevent XSS vulnerabilities.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

v-html interprets content as HTML rather than ordinary text.

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

## Good

Mustache interpolation displays content as escaped text instead of injecting HTML.

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-v-html": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51)
