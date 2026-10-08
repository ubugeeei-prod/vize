---
title: "vue/no-useless-template-attributes"
---

# `vue/no-useless-template-attributes`

Disallow useless attributes on `&lt;template&gt;` elements

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-useless-template-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The conditional template has a class, but this structural wrapper does not render a DOM element to receive it.

```vue
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

## Good

The class moves to the paragraph that actually renders while v-if stays on the structural template.

```vue
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [All rules](../all.md)
