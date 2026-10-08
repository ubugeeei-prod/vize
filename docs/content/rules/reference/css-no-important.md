---
title: "css/no-important"
---

# `css/no-important`

Discourage use of !important in CSS

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

The color declaration overrides normal cascade priority with `!important`.

```vue
<style scoped>
.button {
  color: red !important;
}
</style>
```

## Good

The color comes from a custom property without an important declaration.

```vue
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) · [All rules](../all.md)
