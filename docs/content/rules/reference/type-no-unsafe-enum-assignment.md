---
title: "type/no-unsafe-enum-assignment"
description: "Disallow assigning a non-enum value to an enum."
---

# `type/no-unsafe-enum-assignment`

Disallow assigning a non-enum value to an enum.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The binding is annotated as Color, but it is initialized with the number 1.

```vue annotate="remove:6"
<script setup lang="ts">
enum Color {
  Red = 0,
  Green = 1,
}
const color: Color = 1;
</script>
```

## Good

Color.Red is a member of the Color enum.

```vue annotate="add:6"
<script setup lang="ts">
enum Color {
  Red = 0,
  Green = 1,
}
const color: Color = Color.Red;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-enum-assignment": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_enum_assignment.rs#L5)
