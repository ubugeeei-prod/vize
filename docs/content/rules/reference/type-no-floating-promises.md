---
title: "type/no-floating-promises"
---

# `type/no-floating-promises`

Disallow floating (unhandled) Promises

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
        "type/no-floating-promises": "warn"
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

The async `save` function returns a Promise, but the standalone `save()` call neither awaits nor returns it and does not explicitly mark intentional disposal.

```vue
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

## Good

`void save()` explicitly marks the fire-and-forget intent accepted by this rule. This is an explicit disposal marker, not a rejection handler.

```vue
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) · [All rules](../all.md)
