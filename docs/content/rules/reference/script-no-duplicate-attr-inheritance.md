---
title: "script/no-duplicate-attr-inheritance"
---

# `script/no-duplicate-attr-inheritance`

Flag a component that applies its fallthrough attributes twice

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The component keeps automatic attribute inheritance and also forwards `$attrs` to its button. The same fallthrough attributes can be applied twice.

```vue
<script setup></script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## Good

`inheritAttrs: false` disables automatic inheritance, leaving the explicit button binding as the single forwarding path.

```vue
<script setup>defineOptions({ inheritAttrs: false })</script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L35) · [All rules](../all.md)
