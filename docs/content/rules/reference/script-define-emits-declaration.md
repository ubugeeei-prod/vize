---
title: "script/define-emits-declaration"
---

# `script/define-emits-declaration`

Enforce the type-based defineEmits<{}>() form over the runtime/array form

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
        "script/define-emits-declaration": "warn"
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
// Runtime array form
const emit = defineEmits(['change', 'update'])

// Runtime object form
const emit = defineEmits({
change: (id: number) => true,
})
</script>
```

## Good

```vue
<script setup lang="ts">
// Type-based form (preferred)
const emit = defineEmits<{ change: [id: number] }>()

// Named type alias
const emit = defineEmits<Emits>()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [All rules](../all.md)
