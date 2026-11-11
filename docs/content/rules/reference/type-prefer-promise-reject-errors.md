---
title: "type/prefer-promise-reject-errors"
description: "Require Promise rejection reasons to be Error-like values."
---

# `type/prefer-promise-reject-errors`

Require Promise rejection reasons to be Error-like values.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

Promise.reject receives a string, which is not an Error reason.

```vue annotate="remove:2"
<script setup lang="ts">
const rejected = Promise.reject("failure");
</script>
```

## Good

Promise.reject receives an Error for the same failure text.

```vue annotate="add:2"
<script setup lang="ts">
const rejected = Promise.reject(new Error("failure"));
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-promise-reject-errors": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_promise_reject_errors.rs#L5)
