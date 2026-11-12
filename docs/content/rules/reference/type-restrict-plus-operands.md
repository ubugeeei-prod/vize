---
title: "type/restrict-plus-operands"
description: "Require plus operands to be explicitly compatible."
---

# `type/restrict-plus-operands`

Require plus operands to be explicitly compatible.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

label is a string and numbers is an array, so the two + operands are not compatible primitives.

```vue annotate="remove:3,4"
<script setup lang="ts">
const label = "a";
const numbers = [1, 2, 3];
const mixedSum = label + numbers;
</script>
```

## Good

label and extra are both strings, so the + operands match.

```vue annotate="add:3,4"
<script setup lang="ts">
const label = "a";
const extra = "b";
const mixedSum = label + extra;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/restrict-plus-operands": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/restrict_plus_operands.rs#L5)
