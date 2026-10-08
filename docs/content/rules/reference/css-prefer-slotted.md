---
title: "css/prefer-slotted"
---

# `css/prefer-slotted`

Recommend ::v-slotted() for styling slot content

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: CSS inside SFC style blocks  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The scoped stylesheet targets the `slot` outlet rather than the elements supplied through the slot.

```vue
<style scoped>
slot { color: red; }
</style>
```

## Good

`:slotted(.label)` targets the supplied label element through the scoped slot selector.

```vue
<style scoped>
:slotted(.label) { color: red; }
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [All rules](../all.md)
