---
title: "html/deprecated-attr"
description: "Disallow deprecated HTML attributes."
---

# `html/deprecated-attr`

Disallow deprecated HTML attributes.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The paragraph uses the deprecated presentational `align` attribute.

```vue annotate="remove:1,2,3"
<template>
<p align="center">Notice</p>
</template>
```

## Good

The class and `text-align: center` declaration express the alignment through CSS.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/deprecated-attr": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [HTML rules](../html.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32)
