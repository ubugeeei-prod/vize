---
title: "type/no-floating-promises"
description: "Disallow floating (unhandled) Promises."
---

# `type/no-floating-promises`

Disallow floating (unhandled) Promises.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The async `save` function returns a Promise, but the standalone `save()` call neither awaits nor returns it and does not explicitly mark intentional disposal.

```vue annotate="remove:3"
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

## Good

`void save()` explicitly marks the fire-and-forget intent accepted by this rule. This is an explicit disposal marker, not a rejection handler.

```vue annotate="add:3"
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-floating-promises": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13)
