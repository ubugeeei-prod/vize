---
title: "script/no-export-in-script-setup"
---

# `script/no-export-in-script-setup`

Disallow export statements inside &lt;script setup&gt;

[Bad](#bad) · [Good](#good)

Default severity: `error`  
Presets: _none_  
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
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`export const count` attempts to expose a module export from `<script setup>`, where runtime exports are prohibited.

```vue
<script setup lang="ts">
export const count = 1;
</script>
```

## Good

Removing `export` keeps `count` as a setup binding rather than a module export.

```vue
<script setup lang="ts">
const count = 1;
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) · [All rules](../all.md)
