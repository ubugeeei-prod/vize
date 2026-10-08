---
title: "script/valid-define-emits"
---

# `script/valid-define-emits`

Enforce valid defineEmits() usage (no type+runtime args, no local references, single call)

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
        "script/valid-define-emits": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The same `defineEmits` call supplies both a type argument and the runtime array `["save"]`, mixing two mutually exclusive declarations.

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

## Good

Removing the runtime argument leaves a single type-based event declaration for `save`.

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) · [All rules](../all.md)
