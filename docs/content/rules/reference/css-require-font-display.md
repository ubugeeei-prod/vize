---
title: "css/require-font-display"
description: "Require font-display in @font-face rules."
---

# `css/require-font-display`

Require font-display in @font-face rules.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: Plain CSS style blocks (scss, sass, less and stylus blocks are skipped).

## Bad

The font-face declaration defines the font source but omits its font-display policy.

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

## Good

`font-display: swap` explicitly selects the fallback-to-font display policy.

```vue annotate="add:5"
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
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
        "css/require-font-display": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [CSS rules](../css.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13)
