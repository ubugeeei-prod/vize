---
title: "type/no-confusing-void-expression"
description: "Disallow void-typed expressions in confusing value positions."
---

# `type/no-confusing-void-expression`

Disallow void-typed expressions in confusing value positions.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

logVoid() returns void, but the initializer treats that call as a value.

```vue annotate="remove:3"
<script setup lang="ts">
function logVoid(): void {}
const value = logVoid();
</script>
```

## Good

Calling logVoid() as a statement uses the void result only for its effect.

```vue annotate="add:3"
<script setup lang="ts">
function logVoid(): void {}
logVoid();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-confusing-void-expression": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_confusing_void_expression.rs#L5)
