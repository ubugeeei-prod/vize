---
title: "a11y/anchor-has-content"
description: "Require anchor elements to have accessible content."
---

# `a11y/anchor-has-content`

Require anchor elements to have accessible content.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The `/settings` link has no text or other naming content, so its destination has no accessible description.

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

## Good

The visible `Settings` text supplies content for the same destination link.

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
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
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L20)
