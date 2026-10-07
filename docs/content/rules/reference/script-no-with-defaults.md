---
title: "script/no-with-defaults"
---

# `script/no-with-defaults`

Discourage withDefaults in favor of destructuring defaults (Vue 3.5+)

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
        "script/no-with-defaults": "warn"
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
// Using withDefaults (verbose)
const props = withDefaults(defineProps<{
count?: number
name?: string
}>(), {
count: 0,
name: 'default'
})
</script>
```

## Good

```vue
<script setup lang="ts">
// Using destructuring defaults (Vue 3.5+)
const { count = 0, name = 'default' } = defineProps<{
count?: number
name?: string
}>()

// Or without destructuring if defaults not needed
const props = defineProps<{ count: number }>()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [All rules](../all.md)
