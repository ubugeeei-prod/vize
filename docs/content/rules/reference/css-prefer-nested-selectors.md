---
title: "css/prefer-nested-selectors"
---

# `css/prefer-nested-selectors`

Recommend using CSS nesting for descendant selectors

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
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The `.card .title` descendant selector repeats the parent selector in a flat rule.

```vue
<style scoped>
.card .title { color: red; }
</style>
```

## Good

The `.title` rule is nested inside `.card`, keeping the parent-child styling relationship together.

```vue
<style scoped>
.card { .title { color: red; } }
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [All rules](../all.md)
