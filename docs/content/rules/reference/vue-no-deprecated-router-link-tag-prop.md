---
title: "vue/no-deprecated-router-link-tag-prop"
---

# `vue/no-deprecated-router-link-tag-prop`

Disallow the `tag` prop on &lt;router-link&gt;

[Bad](#bad) · [Good](#good)

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
        "vue/no-deprecated-router-link-tag-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

RouterLink uses the removed tag prop to request a button element.

```vue
<template>
<router-link to="/home" tag="button">Home</router-link>
</template>
```

## Good

The slot provides navigate to an explicitly authored button.

```vue
<template>
<router-link to="/home" v-slot="{ navigate }">
<button @click="navigate">Home</button>
</router-link>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [All rules](../all.md)
