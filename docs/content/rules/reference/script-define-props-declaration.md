---
title: "script/define-props-declaration"
---

# `script/define-props-declaration`

Enforce type-based defineProps<{ ... }>() over the runtime/object form

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
        "script/define-props-declaration": "warn"
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
// Runtime / object form
const props = defineProps({
kind: { type: String },
count: { type: Number, default: 0 },
})

// Runtime / array form
const props = defineProps(['kind', 'count'])
</script>
```

## Good

```vue
<script setup lang="ts">
// Type-based form
const props = defineProps<{ kind: string; count?: number }>()

// Type-based with a referenced interface
const props = defineProps<Props>()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [All rules](../all.md)
