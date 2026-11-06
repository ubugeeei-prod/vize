---
title: "musea/require-component"
description: "Require component attribute in <art> block."
---

# `musea/require-component`

Require component attribute in &lt;art&gt; block.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | No | None |

Applies to: Musea .art.vue art, variant, and style blocks.

## Bad

The art block supplies a title but does not identify the component being previewed.

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

## Good

defineArt supplies ./Button.vue as the component for the art block.

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

- [Musea and CSS rules](../musea-and-css.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11)
