---
title: "css/no-important"
---

# `css/no-important`

Discourage use of !important in CSS

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
        "css/no-important": "warn"
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
.button {
  color: red !important;
}
</style>
```

## Good

```vue
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L15) · [All rules](../all.md)
