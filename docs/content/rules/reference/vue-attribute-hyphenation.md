---
title: "vue/attribute-hyphenation"
---

# `vue/attribute-hyphenation`

Enforce attribute naming style on custom components

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: Available for supported findings  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: See [typed options and defaults](../options.md).

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/attribute-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The component attribute uses the camelCase spelling firstName.

```vue
<template>
<UserCard firstName="Ada" />
</template>
```

## Good

The first-name spelling follows the configured hyphenated component-attribute convention.

```vue
<template>
<UserCard first-name="Ada" />
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [All rules](../all.md)
