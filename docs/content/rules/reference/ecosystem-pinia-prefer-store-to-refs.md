---
title: "ecosystem/pinia-prefer-store-to-refs"
---

# `ecosystem/pinia-prefer-store-to-refs`

Prefer storeToRefs() when destructuring Pinia stores

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

Destructuring `name` directly from the store separates the value from its reactive store access.

```vue
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

## Good

The store remains intact and storeToRefs creates a reactive reference for name.

```vue
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) · [All rules](../all.md)
