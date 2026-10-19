---
title: "a11y/placeholder-label-option"
---

# `a11y/placeholder-label-option`

Require a placeholder label option in a required select

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The select is `required`, but its first option is Japan, so the form starts with a country already chosen and `required` never blocks submission.

```vue
<template>
  <select v-model="country" required>
    <option value="jp">Japan</option>
    <option value="fr">France</option>
  </select>
</template>
```

## Good

An empty first option is the placeholder label option: the select starts with no value, so `required` makes the user pick a country.

```vue
<template>
  <select v-model="country" required>
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L47) · [All rules](../all.md)
