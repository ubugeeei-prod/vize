---
title: "type/no-unnecessary-type-assertion"
description: "Disallow type assertions that do not change the expression type."
---

# `type/no-unnecessary-type-assertion`

Disallow type assertions that do not change the expression type.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

known is already number, so asserting number does not change its type.

```vue annotate="remove:3"
<script setup lang="ts">
const known: number = 1;
const asserted = known as number;
</script>
```

## Good

Using known without an assertion keeps the annotated number type.

```vue annotate="add:3"
<script setup lang="ts">
const known: number = 1;
const asserted = known;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-assertion": "warn"
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_assertion.rs#L5)
