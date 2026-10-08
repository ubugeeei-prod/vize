---
title: "type/no-unsafe-template-binding"
---

# `type/no-unsafe-template-binding`

Disallow template bindings that resolve to unsafe types

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: No rule-specific options. Severity and preset selection are configurable.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-unsafe-template-binding": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

## Bad

The interpolated `value` is explicitly typed as `any`, so the checker cannot give the template binding a safe concrete type.

```vue
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

## Good

Changing the annotation to `string` gives the same interpolation a concrete, checkable type without changing the rendered value.

```vue
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) · [All rules](../all.md)
