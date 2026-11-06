---
title: "musea/require-title"
description: "Require title attribute in <art> block."
---

# `musea/require-title`

Require title attribute in &lt;art&gt; block.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | None |

Applies to: Musea .art.vue art, variant, and style blocks.

## Bad

The art block identifies Button.vue but supplies no title.

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

## Good

The defineArt options supply the Button title for the art block.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
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
        "musea/require-title": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Musea rules](../musea.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31)
