---
title: "musea/require-component"
description: "Require a resolvable target component for an <art> block."
---

# `musea/require-component`

Require a resolvable target component for an &lt;art&gt; block.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Musea .art.vue art, variant, and style blocks.

## Bad

The art block supplies a title but does not identify the component being previewed.

```vue annotate="remove:1,2"
<art title="Button">
  <variant name="primary"><div /></variant>
</art>
```

## Good

The imported Button rendered by the variant identifies the art target.

```vue annotate="add:1,2,3,4,5,6"
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
      "musea/require-component": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Musea rules](../musea.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11)
