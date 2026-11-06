---
title: "css/no-display-none"
description: "Suggest using v-show instead of display: none."
---

# `css/no-display-none`

Suggest using v-show instead of display: none.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: Plain CSS style blocks (scss, sass, less and stylus blocks are skipped).

## Bad

The `.message` declaration hides the local paragraph through CSS rather than a template visibility condition.

```vue annotate="remove:2,4,5,6,7,8,9"
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

## Good

`v-show="isSaved"` makes the visibility condition explicit on the local paragraph and removes `display: none`.

```vue annotate="add:2"
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/no-display-none": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Musea and CSS rules](../musea-and-css.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27)
