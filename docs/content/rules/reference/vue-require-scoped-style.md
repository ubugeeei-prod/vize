---
title: "vue/require-scoped-style"
---

# `vue/require-scoped-style`

Require scoped attribute on style tags

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The `.button` style is unscoped and can affect matching elements outside this component.

```vue
<style>
.button {
  color: red;
}
</style>
```

## Good

Adding `scoped` applies Vue's component scope to the same selector and declarations.

```vue
<style scoped>
.button {
  color: red;
}
</style>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [All rules](../all.md)
