---
title: "vue/valid-attribute-name"
---

# `vue/valid-attribute-name`

Require valid attribute names

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vue SFC templates and blocks, with script context where the rule requires it  
Options: No rule-specific options. Severity and preset selection are configurable.

Bad diagnostic: `parser/template`

Malformed attribute spelling is diagnosed by parser/template before this defensive rule sees an attribute. Bad therefore reports parser/template; it does not promise a separate vue/valid-attribute-name finding.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-attribute-name": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The quote inside `my"attr` makes the attribute name malformed. This example produces the parser's `parser/template` diagnostic rather than promising a separate rule diagnostic.

```vue
<template>
<div my"attr="value"></div>
</template>
```

## Good

`my-attr` is a well-formed attribute name, so the template parser can read the attribute and its value.

```vue
<template>
<div my-attr="value"></div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [All rules](../all.md)
