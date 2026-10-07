---
title: "script/no-deep-destructure-in-props"
---

# `script/no-deep-destructure-in-props`

Disallow deeply nested destructuring in defineProps

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
        "script/no-deep-destructure-in-props": "warn"
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
// Deep nested destructuring
const { user: { name, age } } = defineProps<{ user: User }>()

// Very deep nesting
const { config: { settings: { theme } } } = defineProps()
</script>
```

## Good

```vue
<script setup lang="ts">
// Simple destructuring (one level)
const { name, count = 0 } = defineProps<{ name: string; count?: number }>()

// Access nested properties in the component instead
const props = defineProps<{ user: User }>()
const userName = computed(() => props.user.name)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [All rules](../all.md)
