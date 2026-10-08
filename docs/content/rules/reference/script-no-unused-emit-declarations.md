---
title: "script/no-unused-emit-declarations"
---

# `script/no-unused-emit-declarations`

Flag declared events that are never emitted

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
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
        "script/no-unused-emit-declarations": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`defineEmits` declares both `change` and `unused`, but the captured `emit` function only emits the literal event `change`.

```vue
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

## Good

Removing `unused` makes the declared event list match the observed emission. The example uses a captured, unescaped emit binding so this local usage conclusion is available.

```vue
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [All rules](../all.md)
