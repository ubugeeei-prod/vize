---
title: "type/no-unsafe-member-access"
description: "Disallow member access on values typed as any."
---

# `type/no-unsafe-member-access`

Disallow member access on values typed as any.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

anyValue is any, so reading member is an unchecked property access.

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
console.log(anyValue.member);
</script>
```

## Good

box has a known member property, so the access is checked.

```vue annotate="add:2,3"
<script setup lang="ts">
const box = { member: 1 };
console.log(box.member);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-member-access": "warn"
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_member_access.rs#L5)
