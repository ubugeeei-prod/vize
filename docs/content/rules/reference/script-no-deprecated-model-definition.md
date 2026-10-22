---
title: "script/no-deprecated-model-definition"
---

# `script/no-deprecated-model-definition`

Disallow the Vue 2 model option, which was removed in Vue 3

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
        "script/no-deprecated-model-definition": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The component still declares the Vue 2 `model` option with `prop` and `event`.

```vue
<script lang="ts">
export default {
  model: {
    prop: 'value',
    event: 'input'
  }
}
</script>
```

## Good

The component declares the `modelValue` prop and the `update:modelValue` event instead.

```vue
<script lang="ts">
export default {
  props: ['modelValue'],
  emits: ['update:modelValue']
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_model_definition.rs#L32) · [All rules](../all.md)
