---
title: "vue/no-template-lang"
---

# `vue/no-template-lang`

Discourage lang attribute on template block

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
        "vue/no-template-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The template selects Pug through lang. This is an intended HTML-only convention; the current SFC path does not diagnose this catalog ID.

```vue
<template lang="pug">
p Notice
</template>
```

## Good

An ordinary HTML template omits lang and uses the paragraph directly. This illustrates the convention without claiming a current SFC finding.

```vue
<template>
<p>Notice</p>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L11) · [All rules](../all.md)
