---
title: "type/no-implied-eval"
description: "Disallow string-based dynamic code execution APIs."
---

# `type/no-implied-eval`

Disallow string-based dynamic code execution APIs.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

setTimeout receives a string, which is executed as dynamic code.

```vue annotate="remove:2,3"
<script setup lang="ts">
function logVoid(): void {}
setTimeout("logVoid()", 10);
</script>
```

## Good

setTimeout receives a function, so no string is evaluated.

```vue annotate="add:2"
<script setup lang="ts">
setTimeout(() => {}, 10);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-implied-eval": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_implied_eval.rs#L5)
