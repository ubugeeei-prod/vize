---
title: "script/one-component-per-file"
---

# `script/one-component-per-file`

Disallow more than one component definition in a file

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
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
        "script/one-component-per-file": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The file exports `Parent` and also defines an inline `Child` component in `components`.

```vue
<script lang="ts">
export default {
  name: 'Parent',
  components: {
    Child: { name: 'Child' }
  }
}
</script>
```

## Good

The file defines only `Parent`.

```vue
<script lang="ts">
export default {
  name: 'Parent'
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/one_component_per_file.rs#L30) · [All rules](../all.md)
