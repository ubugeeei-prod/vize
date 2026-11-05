---
title: "a11y/role-has-required-aria-props"
description: "Require ARIA roles to have required properties."
---

# `a11y/role-has-required-aria-props`

Require ARIA roles to have required properties.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The checkbox role omits `aria-checked`, which conveys the checkbox state.

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

## Good

`aria-checked="false"` supplies the state required by the checkbox role.

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
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
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L37)
