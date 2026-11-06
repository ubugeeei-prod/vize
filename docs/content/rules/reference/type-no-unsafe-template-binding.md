---
title: "type/no-unsafe-template-binding"
description: "Disallow template bindings that resolve to unsafe types."
---

# `type/no-unsafe-template-binding`

Disallow template bindings that resolve to unsafe types.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | None |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

The interpolated `value` is explicitly typed as `any`, so the checker cannot give the template binding a safe concrete type.

```vue annotate="remove:2"
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

## Good

Changing the annotation to `string` gives the same interpolation a concrete, checkable type without changing the rendered value.

```vue annotate="add:2"
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-template-binding": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12)
