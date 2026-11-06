---
title: "a11y/no-autofocus"
description: "Disallow the use of the autofocus attribute."
---

# `a11y/no-autofocus`

Disallow the use of the autofocus attribute.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The input requests automatic focus when it appears.

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

## Good

Removing `autofocus` avoids this automatic focus request while retaining the query input.

```vue annotate="add:2"
<template>
  <input name="query" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-autofocus": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Accessibility rules](../accessibility.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19)
