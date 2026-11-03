---
title: "css/prefer-logical-properties"
description: "Recommend CSS logical properties for better i18n support."
---

# `css/prefer-logical-properties`

Recommend CSS logical properties for better i18n support.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: Plain CSS style blocks (scss, sass, less and stylus blocks are skipped).

## Bad

`margin-left` fixes the margin to a physical side regardless of writing direction.

```vue annotate="remove:3"
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

## Good

`margin-inline-start` follows the start of the inline direction instead.

```vue annotate="add:3"
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Musea and CSS rules](../musea-and-css.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15)
