---
title: "script/prefer-use-id"
---

# `script/prefer-use-id`

Recommend using useId() for generating unique IDs (Vue 3.5+)

Default severity: `warning`  
Presets: _none_  
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
        "script/prefer-use-id": "warn"
      }
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
// Manual ID generation (not SSR-safe)
const id = `input-${Math.random()}`
const id = `field-${Date.now()}`
let counter = 0; const id = `el-${counter++}`
</script>
```

## Good

```vue
<script setup lang="ts">
// Using useId() (Vue 3.5+)
const id = useId()

// In template
<label :for="id">Name</label>
<input :id="id" />
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) · [All rules](../all.md)
