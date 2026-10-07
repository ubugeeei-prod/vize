---
title: "vue/no-deprecated-html-element-is"
---

# `vue/no-deprecated-html-element-is`

Disallow the `is` attribute on native HTML elements

Default severity: `error`  
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
        "vue/no-deprecated-html-element-is": "error"
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
<template>
<div is="MyComponent" />
</template>
```

## Good

```vue
<template>
<component :is="MyComponent" />
<div is="vue:MyComponent" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) · [All rules](../all.md)
