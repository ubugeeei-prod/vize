---
title: "script/no-deprecated-data-object-declaration"
---

# `script/no-deprecated-data-object-declaration`

Disallow an object literal as the component data option (Vue 3 requires a function)

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
        "script/no-deprecated-data-object-declaration": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The Options API `data` option is an object literal, a Vue 2 form that Vue 3 no longer accepts.

```vue
<script lang="ts">
export default {
// `data` must be a function in Vue 3, not an object literal.
data: {
count: 0
}
}
</script>
```

## Good

`data()` returns a new `{ count: 0 }` object, providing the function-based data declaration required by Vue 3.

```vue
<script lang="ts">
export default {
data() {
return { count: 0 }
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) · [All rules](../all.md)
