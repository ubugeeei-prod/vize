---
title: "type/no-misused-promises"
description: "Disallow Promises in places not designed to handle them."
---

# `type/no-misused-promises`

Disallow Promises in places not designed to handle them.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

forEach expects a void callback, but the async callback returns a Promise.

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.forEach(async (n) => {
  console.log(n);
});
</script>
```

## Good

The callback is not async, so it returns void as forEach expects.

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.forEach((n) => {
  console.log(n);
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
      "type/no-misused-promises": "warn"
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
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_misused_promises.rs#L5)
