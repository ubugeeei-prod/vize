---
title: "script/return-in-computed-property"
---

# `script/return-in-computed-property`

Require a return value in every computed getter

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
        "script/return-in-computed-property": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The block-bodied computed getter evaluates `1 + 2` but never returns it, leaving the computed value undefined.

```vue
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { 1 + 2; });
</script>
```

## Good

`return 1 + 2` turns the expression into the getter’s returned value. The rule looks for a value-returning return in the getter itself, not merely an expression statement.

```vue
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { return 1 + 2; });
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) · [All rules](../all.md)
