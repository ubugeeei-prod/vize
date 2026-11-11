---
title: "type/no-mixed-enums"
description: "Disallow mixing string and numeric enum members."
---

# `type/no-mixed-enums`

Disallow mixing string and numeric enum members.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The enum mixes a numeric member with a string member.

```vue annotate="remove:4"
<script setup lang="ts">
enum Mixed {
  A = 0,
  B = "b",
}
</script>
```

## Good

Both members are numeric, so the enum has one value domain.

```vue annotate="add:4"
<script setup lang="ts">
enum Mixed {
  A = 0,
  B = 1,
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
      "type/no-mixed-enums": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_mixed_enums.rs#L5)
