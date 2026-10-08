---
title: "css/prefer-logical-properties"
---

# `css/prefer-logical-properties`

Recommend CSS logical properties for better i18n support

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
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`margin-left` fixes the margin to a physical side regardless of writing direction.

```vue
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

## Good

`margin-inline-start` follows the start of the inline direction instead.

```vue
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [All rules](../all.md)
