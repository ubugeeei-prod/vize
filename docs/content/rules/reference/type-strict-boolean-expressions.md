---
title: "type/strict-boolean-expressions"
---

# `type/strict-boolean-expressions`

Require safe boolean expressions in script and template conditions

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below  
Options: See [typed options and defaults](../options.md).

Enable typeAware and this rule explicitly. The default disallows nullable numbers, while non-null numbers are allowed.

Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/strict-boolean-expressions": "warn"
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

```vue
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

## Good

```vue
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [All rules](../all.md)
