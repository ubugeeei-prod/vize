---
title: "script/no-top-level-ref-in-script"
---

# `script/no-top-level-ref-in-script`

Disallow top-level ref/reactive to prevent Cross-Request State Pollution

[Bad](#bad) · [Good](#good)

Default severity: `error`  
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
        "script/no-top-level-ref-in-script": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The ordinary `<script>` initializes `count` and `user` at module scope. During SSR, these state objects can be shared across component instances and requests.

```vue
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
setup() {
return { count, user }
}
}
</script>
```

## Good

The setup ref is initialized per component instance; the ordinary script keeps only a constant, a state-producing function, and a ref created inside `setup()`. None creates reactive state at ordinary module scope.

```vue
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
return reactive({ count: 0 })
}

export default {
setup() {
// Create state inside setup
const count = ref(0)
return { count }
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) · [All rules](../all.md)
