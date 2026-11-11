---
title: "type/prefer-reduce-type-parameter"
description: "Prefer Array#reduce type parameters over default-value assertions."
---

# `type/prefer-reduce-type-parameter`

Prefer Array#reduce type parameters over default-value assertions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The initial value is asserted as number[] instead of using the reduce type parameter.

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const collected = numbers.reduce((all, n) => [...all, n], [] as number[]);
</script>
```

## Good

reduce<number[]> states the accumulator type without asserting the initial array.

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const collected = numbers.reduce<number[]>((all, n) => [...all, n], []);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-reduce-type-parameter": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_reduce_type_parameter.rs#L5)
