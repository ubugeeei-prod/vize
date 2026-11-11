---
title: "type/no-unnecessary-boolean-literal-compare"
description: "Disallow unnecessary comparisons against boolean literals."
---

# `type/no-unnecessary-boolean-literal-compare`

Disallow unnecessary comparisons against boolean literals.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

flag is already boolean, so comparing it with true adds no check.

```vue annotate="remove:3"
<script setup lang="ts">
function isFlag(flag: boolean): boolean {
  return flag === true;
}
</script>
```

## Good

Returning flag uses the boolean value directly.

```vue annotate="add:3"
<script setup lang="ts">
function isFlag(flag: boolean): boolean {
  return flag;
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
      "type/no-unnecessary-boolean-literal-compare": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_boolean_literal_compare.rs#L5)
