---
title: "script/define-props-destructuring"
---

# `script/define-props-destructuring`

Enforce consistent style for defineProps destructuring in <script setup>

Default severity: `warning`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: See [typed options and defaults](../options.md).

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-destructuring": "warn"
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
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

## Good

```vue
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [All rules](../all.md)
