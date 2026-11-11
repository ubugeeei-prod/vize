---
title: "type/no-for-in-array"
description: "Disallow for-in iteration over array-like values."
---

# `type/no-for-in-array`

Disallow for-in iteration over array-like values.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

for-in walks the keys of an array instead of its element values.

```vue annotate="remove:4,5"
<script setup lang="ts">
const numbers = [1, 2, 3];
function loopsKeys(): void {
  for (const key in numbers) {
    console.log(key);
  }
}
</script>
```

## Good

for-of walks the numeric elements of the array.

```vue annotate="add:4,5"
<script setup lang="ts">
const numbers = [1, 2, 3];
function loopsKeys(): void {
  for (const n of numbers) {
    console.log(n);
  }
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-for-in-array": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_for_in_array.rs#L5)
