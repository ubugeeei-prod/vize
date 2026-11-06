---
title: "musea/valid-variant"
description: "Require name attribute in <variant> blocks."
---

# `musea/valid-variant`

Require name attribute in &lt;variant&gt; blocks.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | None |

Applies to: Musea .art.vue art, variant, and style blocks.

## Bad

The variant omits the name needed to identify the preview.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

## Good

The primary name identifies that variant.

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Musea rules](../musea.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8)
