---
title: "type/await-thenable"
description: "Disallow awaiting non-thenable values."
---

# `type/await-thenable`

Disallow awaiting non-thenable values.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

plainNumber returns a number, so awaiting it waits for a value that is not thenable.

```vue annotate="remove:2,3,4,6"
<script setup lang="ts">
function plainNumber(): number {
  return 1;
}
async function run(): Promise<number> {
  return await plainNumber();
}
</script>
```

## Good

Promise.resolve(1) is thenable, so awaiting it waits for a real Promise.

```vue annotate="add:3"
<script setup lang="ts">
async function run(): Promise<number> {
  return await Promise.resolve(1);
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
      "type/await-thenable": "warn"
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/await_thenable.rs#L5)
