---
title: "vue/no-template-lang"
---

# `vue/no-template-lang`

Discourage lang attribute on template block

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `nuxt`, `opinionated`  
Automatic fix: Not implemented for SFC lint  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Current support: `no-sfc-finding`

This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.

## Configured ID (currently no SFC finding)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
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

Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [All rules](../all.md)
