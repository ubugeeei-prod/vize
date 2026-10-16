---
title: "script/no-deprecated-dollar-listeners-api"
---

# `script/no-deprecated-dollar-listeners-api`

Disallow the $listeners instance property removed in Vue 3 (merged into $attrs)

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The component instance alias and the unbound template reference use `$listeners`, which Vue 3 removed after merging listeners into attributes.

```vue
<script>
export default {
  mounted() {
    const vm = this
    return vm.$listeners
  }
}
</script>
<template><Child v-on="$listeners" /></template>
```

## Good

The component reads `$attrs`, and `v-bind="$attrs"` forwards attributes and on-prefixed listener props. Local template bindings and unrelated receivers are outside this rule.

```vue
<script>
export default {
  mounted() {
    return this.$attrs
  }
}
</script>
<template><Child v-bind="$attrs" /></template>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L54) · [All rules](../all.md)
