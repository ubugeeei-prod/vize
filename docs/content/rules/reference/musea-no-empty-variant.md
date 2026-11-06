---
title: "musea/no-empty-variant"
description: "Disallow empty <variant> blocks."
---

# `musea/no-empty-variant`

Disallow empty &lt;variant&gt; blocks.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Musea .art.vue art, variant, and style blocks.

## Bad

The named primary variant is empty, so it provides no preview content.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## Good

The variant renders a primary Button with its Save content.

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/no-empty-variant": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Musea and CSS rules](../musea-and-css.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8)
