---
title: "script/no-reactive-destructure"
---

# `script/no-reactive-destructure`

Disallow destructuring reactive objects which loses reactivity

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
        "script/no-reactive-destructure": "warn"
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
const state = reactive({ count: 0, name: 'foo' })
const { count, name } = state  // loses reactivity!

// Also passing reactive directly to functions that expect refs
someFunction(state.count)  // loses reactivity
</script>
```

## Good

```vue
<script setup lang="ts">
const state = reactive({ count: 0, name: 'foo' })

// Use toRef or toRefs to maintain reactivity
const count = toRef(state, 'count')
const { count, name } = toRefs(state)

// Or use computed for derived values
const doubleCount = computed(() => state.count * 2)

// Pass refs or computed to functions
someFunction(toRef(state, 'count'))
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) · [All rules](../all.md)
