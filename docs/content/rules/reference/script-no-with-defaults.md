---
title: "script/no-with-defaults"
---

# `script/no-with-defaults`

Discourage withDefaults in favor of destructuring defaults (Vue 3.5+)

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `opinionated`  
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
        "script/no-with-defaults": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

`withDefaults` wraps the typed props declaration solely to supply `count` and `name` defaults, instead of the Vue 3.5+ destructuring-default style preferred here.

```vue
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

## Good

The destructuring pattern puts `count = 0` and `name = "Ada"` beside their bindings and removes the `withDefaults` wrapper.

```vue
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [All rules](../all.md)
