---
title: "type/no-unsafe-call"
description: "Disallow calling values typed as any."
---

# `type/no-unsafe-call`

Disallow calling values typed as any.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

anyValue is typed as any, so calling it does not check the callee.

```vue annotate="remove:2,3"
<script setup lang="ts">
declare const anyValue: any;
anyValue();
</script>
```

## Good

plainNumber is a function, so the call has a known callee.

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
function plainNumber(): number {
  return 1;
}
plainNumber();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-call": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_call.rs#L5)
