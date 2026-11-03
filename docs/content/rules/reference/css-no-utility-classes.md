---
title: "css/no-utility-classes"
description: "Warn against implementing utility classes in component styles."
---

# `css/no-utility-classes`

Warn against implementing utility classes in component styles.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: Plain CSS style blocks (scss, sass, less and stylus blocks are skipped).

## Bad

The authored selectors use utility-shaped names such as `.flex`, `.mt-4`, and `.text-center`.

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

## Good

A component-specific `.my-component` selector groups the component styling under one semantic name.

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
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
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Musea and CSS rules](../musea-and-css.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37)
