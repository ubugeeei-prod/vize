---
title: "type/require-typed-emits"
---

# `type/require-typed-emits`

Require type definition for defineEmits

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "type/require-typed-emits": "warn"
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

The array-only `defineEmits(["save"])` declares the event name without a typed payload contract.

```vue
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

## Good

`defineEmits<{ save: [] }>()` declares the typed `save` event with an empty payload tuple, explicitly stating that it takes no payload arguments.

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) · [All rules](../all.md)
