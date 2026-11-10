---
title: "a11y/alt-text"
description: "Require alternative text for media elements."
---

# `a11y/alt-text`

Require alternative text for media elements.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The image submit control supplies only its image URL; it has no `alt` text describing the action.

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

## Good

`alt="Submit search"` gives the image control an accessible name that describes submitting the search.

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/alt-text": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L35)
