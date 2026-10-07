---
title: "vue/no-deprecated-slot-scope-attribute"
---

# `vue/no-deprecated-slot-scope-attribute`

Disallow the deprecated `slot-scope` attribute

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
        "vue/no-deprecated-slot-scope-attribute": "error"
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
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

## Good

```vue
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) · [All rules](../all.md)
