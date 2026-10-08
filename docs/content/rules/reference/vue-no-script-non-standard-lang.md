---
title: "vue/no-script-non-standard-lang"
---

# `vue/no-script-non-standard-lang`

Discourage non-standard script lang values

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
        "vue/no-script-non-standard-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The script uses CoffeeScript syntax under lang=coffee. The current SFC path does not emit this catalog rule for that language.

```vue
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

## Good

The script uses an ordinary TypeScript declaration with lang=ts, illustrating the intended language convention.

```vue
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [All rules](../all.md)
