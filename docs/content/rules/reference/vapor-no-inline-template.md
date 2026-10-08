---
title: "vapor/no-inline-template"
---

# `vapor/no-inline-template`

Disallow deprecated inline-template attribute

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `nuxt`, `opinionated`  
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
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

LegacyCard uses the inline-template attribute for its child markup.

```vue
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

## Good

The markup is passed through the default slot instead of an inline template.

```vue
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) · [All rules](../all.md)
