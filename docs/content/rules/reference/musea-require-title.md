---
title: "musea/require-title"
description: "Require a resolvable display title for an <art> block."
---

# `musea/require-title`

Require a resolvable display title for an &lt;art&gt; block.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | None | No | None |

Applies to: Musea .art.vue art, variant, and style blocks.

## Bad

The art block has neither explicit metadata nor an inferable component title.

```vue annotate="remove:2"
<art>
  <variant name="primary"><div /></variant>
</art>
```

## Good

The imported Button target supplies the inferred display title.

```vue annotate="add:1,2,3,4,6"
<script setup>
import Button from "./Button.vue";
</script>

<art>
  <variant name="primary"><Button /></variant>
</art>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/require-title": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Musea rules](../musea.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L32)
