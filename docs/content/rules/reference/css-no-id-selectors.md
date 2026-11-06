---
title: "css/no-id-selectors"
description: "Discourage use of ID selectors in CSS."
---

# `css/no-id-selectors`

Discourage use of ID selectors in CSS.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: Plain CSS style blocks (scss, sass, less and stylus blocks are skipped).

## Bad

`#submit` ties the style rule to an ID selector.

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

## Good

The `.submit` class expresses the reusable styling hook without an ID selector.

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
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
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [CSS rules](../css.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19)
