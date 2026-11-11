---
title: "type/no-misused-spread"
description: "Disallow spreading values into incompatible positions."
---

# `type/no-misused-spread`

Disallow spreading values into incompatible positions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

Spreading a string into an array uses a value in an incompatible position.

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const chars = [...label];
</script>
```

## Good

Spreading an array into an array keeps the element type compatible.

```vue annotate="add:2,3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const chars = [...numbers];
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-misused-spread": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_misused_spread.rs#L5)
