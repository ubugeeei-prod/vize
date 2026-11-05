---
title: "a11y/heading-has-content"
description: "Require heading elements to have accessible content."
---

# `a11y/heading-has-content`

Require heading elements to have accessible content.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The `h2` contributes a heading level but has no heading content.

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

## Good

`Billing settings` supplies the content of the existing level-two heading.

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
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
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17)
