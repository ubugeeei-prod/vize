---
title: "type/no-base-to-string"
description: "Disallow stringifying values that fall back to Object.prototype.toString()."
---

# `type/no-base-to-string`

Disallow stringifying values that fall back to Object.prototype.toString().

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

plain is a plain object, so toString() falls back to Object.prototype.toString().

```vue annotate="remove:2,3"
<script setup lang="ts">
const plain = { a: 1 };
plain.toString();
</script>
```

## Good

label is already a string, so toString() uses the string conversion.

```vue annotate="add:2,3"
<script setup lang="ts">
const label = "a";
label.toString();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-base-to-string": "warn"
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_base_to_string.rs#L5)
