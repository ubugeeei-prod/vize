---
title: "a11y/landmark-roles"
description: "Validate landmark role placement and uniqueness."
---

# `a11y/landmark-roles`

Validate landmark role placement and uniqueness.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Two `main` elements declare duplicate main landmarks in the same template.

```vue annotate="remove:3"
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

## Good

The dashboard remains the main landmark; the settings area becomes a named navigation landmark.

```vue annotate="add:3"
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/landmark-roles": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44)
