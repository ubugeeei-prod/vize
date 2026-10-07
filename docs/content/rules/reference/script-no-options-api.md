---
title: "script/no-options-api"
---

# `script/no-options-api`

Disallow Options API patterns in Vapor mode

Default severity: `error`  
Presets: `opinionated`  
Automatic fix: None; review the suggested change  
Applies to: Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
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
  data() {
    return { count: 0 };
  },
};
</script>
```

## Good

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [All rules](../all.md)
