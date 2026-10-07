---
title: "vue/require-component-registration"
---

# `vue/require-component-registration`

Require explicit import or registration for components

Default severity: `warning`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](../options.md).

List explicit component names supplied by application plugins or Musea previewSetup. PascalCase and kebab-case spellings are accepted; regular expressions are not interpreted. Options do not enable the rule. Later layers replace the list; an empty list clears inherited names.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-registration": "warn"
      },
      "ruleOptions": {
        "vue/require-component-registration": {
          "globals": [
            "MyButton",
            "MyIcon"
          ]
        }
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
<MissingWidget />
</template>
```

## Good

```vue
<template>
<MyButton />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [All rules](../all.md)
