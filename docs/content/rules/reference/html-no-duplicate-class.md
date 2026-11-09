---
title: "html/no-duplicate-class"
description: "Disallow duplicate class names in a static class attribute."
---

# `html/no-duplicate-class`

Disallow duplicate class names in a static class attribute.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The static class list repeats the `btn` token.

```vue annotate="remove:2"
<template>
  <div class="btn btn primary">click</div>
</template>
```

## Good

The class list keeps one `btn` token and the distinct `primary` token.

```vue annotate="add:2"
<template>
  <div class="btn primary">click</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-duplicate-class": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [HTML rules](../html.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32)
