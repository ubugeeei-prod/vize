---
title: "script/no-ref-as-operand"
---

# `script/no-ref-as-operand`

Require ref-bound variables to be accessed via `.value` when used as an operand

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
        "script/no-ref-as-operand": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`count + 1` uses the ref object itself as the arithmetic operand instead of the number it wraps.

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

## Good

`count.value + 1` reads the wrapped number before adding one; script arithmetic requires this explicit ref access.

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) · [All rules](../all.md)
