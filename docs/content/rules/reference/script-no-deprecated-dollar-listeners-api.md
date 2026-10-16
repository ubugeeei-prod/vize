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

The script reads the removed `$listeners` property. This rule currently diagnoses scripts only; the template line illustrates migration and is not yet enforced (Draft follow-up #9251).

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

The script uses `$attrs`. Forwarding with `v-bind="$attrs"` illustrates Vue 3 migration; template enforcement is deferred to #9251. Script warnings retain their existing broad receiver/reference coverage.

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
