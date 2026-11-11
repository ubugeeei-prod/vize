---
title: "type/no-duplicate-type-constituents"
description: "Disallow duplicate union or intersection constituents."
---

# `type/no-duplicate-type-constituents`

Disallow duplicate union or intersection constituents.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

string appears twice in the same union, so the second constituent adds nothing.

```vue annotate="remove:2"
<script setup lang="ts">
type Dup = string | string;
</script>
```

## Good

The alias keeps a single string constituent.

```vue annotate="add:2"
<script setup lang="ts">
type Dup = string;
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-duplicate-type-constituents": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_duplicate_type_constituents.rs#L5)
