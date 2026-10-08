---
title: "script/no-potential-component-option-typo"
---

# `script/no-potential-component-option-typo`

Flag likely typos in Options API component option names

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
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The option is spelled `method`, one edit away from the recognized `methods` option; Vue would not treat it as the intended methods declaration.

```vue
<script lang="ts">
export default { method: { save() {} } };
</script>
```

## Good

Changing the key to `methods` places `save()` under the recognized component option.

```vue
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) · [All rules](../all.md)
