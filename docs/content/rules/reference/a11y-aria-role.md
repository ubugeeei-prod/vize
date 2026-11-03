---
title: "a11y/aria-role"
description: "Elements with ARIA roles must use a valid, non-abstract ARIA role."
---

# `a11y/aria-role`

Elements with ARIA roles must use a valid, non-abstract ARIA role.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

`datepicker` is not a recognized ARIA role for this section.

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

## Good

The section uses the recognized `dialog` role and a label describing the date selection.

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
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
        "a11y/aria-role": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21)
