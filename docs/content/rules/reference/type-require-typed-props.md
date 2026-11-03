---
title: "type/require-typed-props"
description: "Require type definition for defineProps."
---

# `type/require-typed-props`

Require type definition for defineProps.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The array-only `defineProps(["title"])` declares `title` by name without giving it a type.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps(["title"]);
</script>
```

## Good

`defineProps<{ title: string }>()` gives `title` an explicit string type instead of a name-only runtime declaration.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-props": "warn"
      },
      "typeAware": true
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57)
