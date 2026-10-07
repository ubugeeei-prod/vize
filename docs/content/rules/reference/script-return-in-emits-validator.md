---
title: "script/return-in-emits-validator"
---

# `script/return-in-emits-validator`

Require a return value in every Options API emits validator

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
        "script/return-in-emits-validator": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

```vue
<script lang="ts">
export default {
emits: {
submit(payload) {
if (!payload.email) {
// missing return
}
}
}
}
</script>
```

## Good

```vue
<script lang="ts">
export default {
emits: {
submit(payload) {
return !!payload.email
}
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) · [All rules](../all.md)
