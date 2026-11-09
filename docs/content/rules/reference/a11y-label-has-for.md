---
title: "a11y/label-has-for"
description: "Require labels to have associated form controls."
---

# `a11y/label-has-for`

Require labels to have associated form controls.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

List component names that render a native form control and therefore satisfy label nesting. PascalCase and kebab-case spellings are equivalent. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.

## Bad

The separate label is neither associated through `for` nor wrapped around the input.

```vue annotate="remove:2"
<template>
<label>Name</label>
</template>
```

## Good

The nested native input is implicitly associated with its wrapping label.

```vue annotate="add:2"
<template>
<label>Name <input v-model="name" /></label>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/label-has-for": "warn"
    },
    "ruleOptions": {
      "a11y/label-has-for": {
        "controlComponents": [
          "CInputField"
        ]
      }
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L36)
