---
title: "script/no-deprecated-vue-config-keycodes"
---

# `script/no-deprecated-vue-config-keycodes`

Disallow Vue.config.keyCodes, which was removed in Vue 3

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
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-vue-config-keycodes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`Vue.config.keyCodes.enter` assigns a removed key-code alias.

```vue
<script setup lang="ts">
Vue.config.keyCodes.enter = 13
</script>
```

## Good

The key code is an ordinary constant, `const enter = 13`.

```vue
<script setup lang="ts">
const enter = 13
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_vue_config_keycodes.rs#L22) · [All rules](../all.md)
