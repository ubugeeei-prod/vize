---
title: "type/no-unsafe-assignment"
description: "Disallow assigning any-typed values to more specific targets."
---

# `type/no-unsafe-assignment`

Disallow assigning any-typed values to more specific targets.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

anyValue is any, and it is assigned to a binding annotated as number.

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
const assigned: number = anyValue;
</script>
```

## Good

The binding is initialized with the number 1.

```vue annotate="add:2"
<script setup lang="ts">
const assigned: number = 1;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-assignment": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_assignment.rs#L5)
