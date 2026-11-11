---
title: "type/only-throw-error"
description: "Require thrown values to be Error-like."
---

# `type/only-throw-error`

Require thrown values to be Error-like.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The thrown value is a string, which is not an Error.

```vue annotate="remove:3"
<script setup lang="ts">
function fail(): never {
  throw "failure";
}
</script>
```

## Good

new Error creates an Error value for the same failure text.

```vue annotate="add:3"
<script setup lang="ts">
function fail(): never {
  throw new Error("failure");
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
      "type/only-throw-error": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/only_throw_error.rs#L5)
