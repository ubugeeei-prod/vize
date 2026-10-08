---
title: "script/no-deprecated-dollar-scopedslots-api"
---

# `script/no-deprecated-dollar-scopedslots-api`

Disallow the $scopedSlots instance property removed in Vue 3 (use $slots)

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
        "script/no-deprecated-dollar-scopedslots-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`this.$scopedSlots`, `ctx.$scopedSlots`, and the bare `$scopedSlots` reference use the Vue 2 scoped-slot API removed in Vue 3.

```vue
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

## Good

Replacing `$scopedSlots` with `$slots` uses the unified slot surface. The example removes the deprecated spelling rather than establishing a setup context for the receivers.

```vue
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) · [All rules](../all.md)
