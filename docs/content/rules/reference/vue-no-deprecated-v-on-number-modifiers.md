---
title: "vue/no-deprecated-v-on-number-modifiers"
---

# `vue/no-deprecated-v-on-number-modifiers`

Disallow deprecated numeric `keyCode` modifiers on `v-on`

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
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
        "vue/no-deprecated-v-on-number-modifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The keyboard handlers identify keys by the removed numeric codes 13 and 27.

```vue
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

## Good

The handlers use the named enter and esc key modifiers.

```vue
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [All rules](../all.md)
