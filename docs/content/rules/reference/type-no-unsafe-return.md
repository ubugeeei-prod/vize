---
title: "type/no-unsafe-return"
description: "Disallow returning any-typed values from functions."
---

# `type/no-unsafe-return`

Disallow returning any-typed values from functions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The function promises a number, but it returns anyValue, which is any.

```vue annotate="remove:2,4"
<script setup lang="ts">
declare const anyValue: any;
function returnsAny(): number {
  return anyValue;
}
</script>
```

## Good

The function returns the number 1, which matches the annotated return type.

```vue annotate="add:3"
<script setup lang="ts">
function returnsAny(): number {
  return 1;
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
      "type/no-unsafe-return": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_return.rs#L5)
