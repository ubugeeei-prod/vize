---
title: "ecosystem/vue-router-prefer-named-push"
---

# `ecosystem/vue-router-prefer-named-push`

Prefer named route objects for Vue Router programmatic navigation

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `ecosystem`  
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
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

router.push receives a path string that is tied to the current URL spelling.

```vue
<script setup lang="ts">
router.push("/settings");
</script>
```

## Good

router.push receives a route object with the stable settings name.

```vue
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [All rules](../all.md)
