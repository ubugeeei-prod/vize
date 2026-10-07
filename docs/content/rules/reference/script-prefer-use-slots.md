---
title: "script/prefer-use-slots"
---

# `script/prefer-use-slots`

Recommend using useSlots() over context.slots

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
        "script/prefer-use-slots": "warn"
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
<script lang="ts">
// In Options API style
export default {
setup(props, { slots }) {
return () => h('div', slots.default?.())
}
}

// Using context.slots
const vnode = context.slots.default?.()
</script>
```

## Good

```vue
<script setup lang="ts">
// Using useSlots()
const slots = useSlots()
return () => h('div', slots.default?.())
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) · [All rules](../all.md)
