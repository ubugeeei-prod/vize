---
title: "script/component-options-name-casing"
---

# `script/component-options-name-casing`

Enforce PascalCase for the component `name` option

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
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The component option `name: 'my-component'` is kebab-case, whereas this rule requires a literal component name in PascalCase.

```vue
<script lang="ts">
export default {
  name: 'my-component' // kebab-case
}
</script>
```

## Good

`MyComponent` begins with an uppercase letter and contains only alphanumeric characters, satisfying the name check.

```vue
<script lang="ts">
export default {
  name: 'MyComponent'
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) · [All rules](../all.md)
