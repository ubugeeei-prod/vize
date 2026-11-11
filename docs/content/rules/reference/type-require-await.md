---
title: "type/require-await"
description: "Disallow async functions that contain no await expression."
---

# `type/require-await`

Disallow async functions that contain no await expression.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

noAwait is async but its body never awaits a value.

```vue annotate="remove:2,3"
<script setup lang="ts">
async function noAwait(): Promise<number> {
  return 1;
}
</script>
```

## Good

withAwait awaits Promise.resolve(1) before returning it.

```vue annotate="add:2,3"
<script setup lang="ts">
async function withAwait(): Promise<number> {
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
      "type/require-await": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/require_await.rs#L5)
