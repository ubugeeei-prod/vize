---
title: "css/prefer-slotted"
---

# `css/prefer-slotted`

Recommend ::v-slotted() for styling slot content

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

```vue
<style scoped>
.content h2 {
  margin-block: 0;
}
</style>
```

## Good

```vue
<style scoped>
::v-slotted(h2) {
  margin-block: 0;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [All rules](../all.md)
