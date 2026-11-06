---
title: "a11y/iframe-has-title"
description: "Require iframe elements to have a title attribute."
---

# `a11y/iframe-has-title`

Require iframe elements to have a title attribute.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The checkout frame has a source URL but no `title` describing the embedded content.

```vue annotate="remove:2"
<template>
  <iframe src="/checkout"></iframe>
</template>
```

## Good

`title="Checkout preview"` names the content of that frame.

```vue annotate="add:2"
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/iframe-has-title": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15)
