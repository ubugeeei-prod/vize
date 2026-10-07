---
title: "script/no-async-in-computed"
---

# `script/no-async-in-computed`

Disallow async functions in computed properties

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-async-in-computed": "error"
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
const data = computed(async () => {
const response = await fetch('/api/data')
return response.json()
})
</script>
```

## Good

```vue
<script setup lang="ts">
// Use ref + watch with cleanup for async operations
const data = ref(null)
watch(query, async (value, _oldValue, onCleanup) => {
const controller = new AbortController()
let active = true
onCleanup(() => {
active = false
controller.abort()
})
const response = await fetch(`/api/data?q=${value}`, { signal: controller.signal })
const next = await response.json()
if (active) data.value = next
})

// Or use a dedicated async state library
const { data } = useAsyncData(() => fetch('/api/data'))
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) · [All rules](../all.md)
