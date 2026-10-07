---
title: "script/no-unused-emit-declarations"
---

# `script/no-unused-emit-declarations`

Flag declared events that are never emitted

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
        "script/no-unused-emit-declarations": "warn"
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
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

## Good

```vue
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [All rules](../all.md)
