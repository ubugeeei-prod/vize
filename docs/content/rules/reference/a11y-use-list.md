---
title: "a11y/use-list"
---

# `a11y/use-list`

Suggest using list elements for bullet-like text

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
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/use-list": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The task items are separate paragraphs with typed dash markers rather than list elements.

```vue
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

## Good

An unordered list and list items express the same tasks with list semantics.

```vue
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [All rules](../all.md)
