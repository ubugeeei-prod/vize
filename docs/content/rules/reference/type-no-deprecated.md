---
title: "type/no-deprecated"
description: "Disallow use of deprecated declarations."
---

# `type/no-deprecated`

Disallow use of deprecated declarations.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

legacy is marked @deprecated, and the call still uses that declaration.

```vue annotate="remove:2,3,6"
<script setup lang="ts">
/** @deprecated use current() */
function legacy(): number {
  return 1;
}
const legacyValue = legacy();
</script>
```

## Good

current has no deprecation mark, so the call uses a supported declaration.

```vue annotate="add:2,5"
<script setup lang="ts">
function current(): number {
  return 1;
}
const currentValue = current();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-deprecated": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_deprecated.rs#L5)
