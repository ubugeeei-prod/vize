---
title: "type/no-redundant-type-constituents"
description: "Disallow members of unions and intersections that do nothing or override type information."
---

# `type/no-redundant-type-constituents`

Disallow members of unions and intersections that do nothing or override type information.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

unknown already covers string, so the string constituent does not narrow the union.

```vue annotate="remove:2"
<script setup lang="ts">
type Redundant = unknown | string;
</script>
```

## Good

string and number are both useful constituents of the union.

```vue annotate="add:2"
<script setup lang="ts">
type Redundant = string | number;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-redundant-type-constituents": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_redundant_type_constituents.rs#L5)
