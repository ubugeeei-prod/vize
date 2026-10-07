---
title: "script/no-boolean-default"
---

# `script/no-boolean-default`

Disallow a default on a Boolean prop

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
        "script/no-boolean-default": "warn"
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
props: {
// Boolean props already default to false; an explicit default is confusing.
disabled: { type: Boolean, default: true },
checked: { type: Boolean, default: false }
}
}
</script>
```

## Good

```vue
<script lang="ts">
export default {
props: {
// No explicit default: defaults to false.
disabled: { type: Boolean },
disabled2: Boolean,
// Union type may legitimately need a default.
value: { type: [Boolean, String], default: '' },
// Non-Boolean prop.
count: { type: Number, default: 0 }
}
}
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) · [All rules](../all.md)
