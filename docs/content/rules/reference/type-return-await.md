---
title: "type/return-await"
description: "Enforce consistent use of `return await` based on context and configuration."
---

# `type/return-await`

Enforce consistent use of `return await` based on context and configuration.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

Inside try, returning getNumber() without await loses the rejection in that catch.

```vue annotate="remove:7"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
async function returnsInTry(): Promise<number> {
  try {
    return getNumber();
  } catch {
    return 0;
  }
}
</script>
```

## Good

return await getNumber() keeps the rejection inside the same try and catch.

```vue annotate="add:7"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
async function returnsInTry(): Promise<number> {
  try {
    return await getNumber();
  } catch {
    return 0;
  }
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
      "type/return-await": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/return_await.rs#L5)
