---
title: "type/no-unsafe-argument"
description: "Disallow calling a function with a value of type `any`."
---

# `type/no-unsafe-argument`

Disallow calling a function with a value of type `any`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

anyValue is any, and it is passed to a parameter that expects number.

```vue annotate="remove:2,6"
<script setup lang="ts">
declare const anyValue: any;
function takesNumber(value: number): number {
  return value;
}
takesNumber(anyValue);
</script>
```

## Good

The call passes the number 1 to the number parameter.

```vue annotate="add:5"
<script setup lang="ts">
function takesNumber(value: number): number {
  return value;
}
takesNumber(1);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-argument": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_argument.rs#L5)
