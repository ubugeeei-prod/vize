---
title: "type/no-unnecessary-type-conversion"
description: "Disallow conversions that do not change the type or value of the expression."
---

# `type/no-unnecessary-type-conversion`

Disallow conversions that do not change the type or value of the expression.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

label is already a string, so String(label) does not change its type or value.

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const same = String(label);
</script>
```

## Good

count is a number, so String(count) converts it to a string.

```vue annotate="add:2,3"
<script setup lang="ts">
const count = 1;
const same = String(count);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-conversion": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_conversion.rs#L5)
