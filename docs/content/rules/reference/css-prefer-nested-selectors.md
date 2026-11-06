---
title: "css/prefer-nested-selectors"
description: "Recommend using CSS nesting for descendant selectors."
---

# `css/prefer-nested-selectors`

Recommend using CSS nesting for descendant selectors.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: Plain CSS style blocks (scss, sass, less and stylus blocks are skipped).

## Bad

The `.card .title` descendant selector repeats the parent selector in a flat rule.

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

## Good

The `.title` rule is nested inside `.card`, keeping the parent-child styling relationship together.

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
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
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [CSS rules](../css.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14)
