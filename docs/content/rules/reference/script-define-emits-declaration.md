---
title: "script/define-emits-declaration"
---

# `script/define-emits-declaration`

Enforce the type-based defineEmits&lt;{}&gt;() form over the runtime/array form

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
        "script/define-emits-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`defineEmits(["change"])` uses a runtime array declaration; this style rule prefers a type-based declaration.

```vue
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

## Good

`defineEmits<{ change: [id: number] }>()` moves the event declaration into a type argument and explicitly describes the numeric payload used by `emit("change", 1)`.

```vue
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [All rules](../all.md)
