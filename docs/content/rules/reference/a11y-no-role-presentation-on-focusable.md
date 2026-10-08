---
title: "a11y/no-role-presentation-on-focusable"
---

# `a11y/no-role-presentation-on-focusable`

Disallow role="presentation" or role="none" on focusable elements

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The focusable billing link requests role=presentation, which conflicts with its interactive link role; browsers must ignore that presentation request.

```vue
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

## Good

Remove the conflicting presentation request and rely on the native link role and billing destination.

```vue
<template>
  <a href="/billing">Billing</a>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [All rules](../all.md)
