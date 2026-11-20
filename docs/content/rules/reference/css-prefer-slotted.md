---
title: "css/prefer-slotted"
description: "Recommend ::v-slotted() for styling slot content."
---

# `css/prefer-slotted`

Recommend ::v-slotted() for styling slot content.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: CSS, SCSS, Sass, Less and Stylus style blocks, as written (not preprocessed). Within one Stylus block, use either braced or indented syntax consistently; mixed syntax is read as braced Stylus.

## Bad

The scoped stylesheet targets the `slot` outlet rather than the elements supplied through the slot.

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

## Good

`:slotted(.label)` targets the supplied label element through the scoped slot selector.

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/prefer-slotted": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [CSS rules](../css.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L35)
