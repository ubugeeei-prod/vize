---
title: "vue/no-unused-setup-bindings"
---

# `vue/no-unused-setup-bindings`

Disallow unread script setup bindings

Default severity: `warning`  
Presets: _none_  
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
        "vue/no-unused-setup-bindings": "warn"
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
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

## Good

```vue
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [All rules](../all.md)
