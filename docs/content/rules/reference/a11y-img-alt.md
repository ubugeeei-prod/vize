---
title: "a11y/img-alt"
description: "Require alt attribute on images for accessibility."
---

# `a11y/img-alt`

Require alt attribute on images for accessibility.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The avatar image is missing its `alt` attribute.

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

## Good

`alt="User avatar"` supplies a text alternative for the avatar.

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/img-alt": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L17)
