---
title: "a11y/no-redundant-roles"
description: "Disallow redundant ARIA roles."
---

# `a11y/no-redundant-roles`

Disallow redundant ARIA roles.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The native button already has the button role, so `role="button"` repeats its implicit semantics.

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

## Good

Removing the repeated role keeps the button semantics supplied by HTML.

```vue annotate="add:2"
<template>
  <button>Save</button>
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
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31)
