---
title: "script/order-in-components"
---

# `script/order-in-components`

Enforce the default order of properties in component options

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/order-in-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`data` is written before `props`, so a later group appears above an earlier one.

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 }
  },
  props: ['label']
}
</script>
```

## Good

`props` comes before `data`, which matches the default group order.

```vue
<script lang="ts">
export default {
  props: ['label'],
  data() {
    return { count: 0 }
  }
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/order_in_components.rs#L34) · [All rules](../all.md)
