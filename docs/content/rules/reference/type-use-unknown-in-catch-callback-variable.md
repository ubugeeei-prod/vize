---
title: "type/use-unknown-in-catch-callback-variable"
description: "Require Promise catch callback variables to use an explicit unknown annotation."
---

# `type/use-unknown-in-catch-callback-variable`

Require Promise catch callback variables to use an explicit unknown annotation.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The catch callback names error without an explicit unknown annotation.

```vue annotate="remove:5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
getNumber().catch((error) => {
  console.log(error);
});
</script>
```

## Good

The catch callback annotates error as unknown.

```vue annotate="add:5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
getNumber().catch((error: unknown) => {
  console.log(error);
});
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/use-unknown-in-catch-callback-variable": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/use_unknown_in_catch_callback_variable.rs#L5)
