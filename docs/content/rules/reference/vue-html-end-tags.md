---
title: "vue/html-end-tags"
description: "Require end tags on HTML elements that may omit them."
---

# `vue/html-end-tags`

Require end tags on HTML elements that may omit them.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

This rule requires explicit end tags where the HTML parser permits omission, such as `li`, `p`, and `td`. Missing mandatory end tags are owned by `parser/template`; void elements do not need an end tag.

## Bad

HTML permits this `li` to end implicitly when `ul` closes, but the rule requires its end tag to be explicit.

```vue annotate="remove:1"
<template><ul><li>First</ul></template>
```

## Good

Add `</li>` after the item's content, before the containing `ul` closes.

```vue annotate="add:1"
<template><ul><li>First</li></ul></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/html-end-tags": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_end_tags.rs#L30)
