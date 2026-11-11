---
title: "type/no-unnecessary-type-arguments"
description: "Disallow type arguments that are equal to the default."
---

# `type/no-unnecessary-type-arguments`

Disallow type arguments that are equal to the default.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The explicit number argument is the same as the default type parameter.

```vue annotate="remove:5"
<script setup lang="ts">
function identity<T = number>(value: T): T {
  return value;
}
const fromIdentity = identity<number>(1);
</script>
```

## Good

Calling identity(1) lets the default type parameter apply.

```vue annotate="add:5"
<script setup lang="ts">
function identity<T = number>(value: T): T {
  return value;
}
const fromIdentity = identity(1);
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-arguments": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_arguments.rs#L5)
