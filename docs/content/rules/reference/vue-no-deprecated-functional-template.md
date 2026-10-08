---
title: "vue/no-deprecated-functional-template"
---

# `vue/no-deprecated-functional-template`

Disallow the `functional` attribute on the SFC `&lt;template&gt;`

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: _none_  
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
        "vue/no-deprecated-functional-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The SFC template has the removed functional attribute and reads the old props context.

```vue
<template functional>
<div>{{ props.msg }}</div>
</template>
```

## Good

The ordinary template omits functional and reads the component binding msg directly.

```vue
<template>
<div>{{ msg }}</div>
</template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) · [All rules](../all.md)
