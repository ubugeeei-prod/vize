---
title: "a11y/form-control-has-label"
description: "Require form controls to have associated labels."
---

# `a11y/form-control-has-label`

Require form controls to have associated labels.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The search input has no label identifying what the user should enter.

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

## Good

Wrapping the input in a label associates the visible `Search` text with the control.

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
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
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L21)
