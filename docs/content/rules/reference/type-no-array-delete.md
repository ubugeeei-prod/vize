---
title: "type/no-array-delete"
description: "Disallow deleting elements from array-like values."
---

# `type/no-array-delete`

Disallow deleting elements from array-like values.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

delete numbers[0] removes an array element and leaves a hole instead of reindexing the array.

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
delete numbers[0];
</script>
```

## Good

splice(0, 1) removes the first element and keeps the array dense.

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.splice(0, 1);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-array-delete": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_array_delete.rs#L5)
