---
title: "html/no-dupe-style-properties"
description: "Disallow duplicate properties in inline style attributes."
---

# `html/no-dupe-style-properties`

Disallow duplicate properties in inline style attributes.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

Each static style repeats one property; `margin` and `MARGIN` also count as the same property.

```vue annotate="remove:2,3"
<template>
  <div style="color: red; color: blue">text</div>
  <div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

## Good

The static style uses distinct color and background properties. Dynamic style bindings are outside this static-attribute check.

```vue annotate="add:2,3"
<template>
  <div style="color: red; background: blue">text</div>
  <div :style="{ color: a, color: b }">text</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-dupe-style-properties": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [HTML rules](../html.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L39)
