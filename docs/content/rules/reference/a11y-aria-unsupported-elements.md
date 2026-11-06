---
title: "a11y/aria-unsupported-elements"
description: "Disallow ARIA attributes on elements that do not support them."
---

# `a11y/aria-unsupported-elements`

Disallow ARIA attributes on elements that do not support them.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The metadata element carries `aria-hidden`, although `meta` does not support ARIA attributes.

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

## Good

Removing the ARIA attribute leaves the charset declaration intact.

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/aria-unsupported-elements": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L22)
