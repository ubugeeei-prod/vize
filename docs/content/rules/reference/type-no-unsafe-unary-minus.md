---
title: "type/no-unsafe-unary-minus"
description: "Disallow unary negation on non-number and non-bigint values."
---

# `type/no-unsafe-unary-minus`

Disallow unary negation on non-number and non-bigint values.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

label is a string, so the unary minus does not negate a number or bigint.

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const negated = -label;
</script>
```

## Good

count is a number, so the unary minus negates that number.

```vue annotate="add:2,3"
<script setup lang="ts">
const count = 1;
const negated = -count;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-unary-minus": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_unary_minus.rs#L5)
