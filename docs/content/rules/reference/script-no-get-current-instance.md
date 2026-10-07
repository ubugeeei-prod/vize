---
title: "script/no-get-current-instance"
---

# `script/no-get-current-instance`

Disallow getCurrentInstance() in Vapor mode (returns null)

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
        "script/no-get-current-instance": "error"
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
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

## Good

```vue
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [All rules](../all.md)
