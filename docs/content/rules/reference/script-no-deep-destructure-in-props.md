---
title: "script/no-deep-destructure-in-props"
---

# `script/no-deep-destructure-in-props`

Disallow deeply nested destructuring in defineProps

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
        "script/no-deep-destructure-in-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The binding pattern descends through `user` to destructure `name`, exceeding the default shallow props-destructuring depth.

```vue
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

## Good

The props object remains intact, and a computed getter reads `props.user.name`. The nested access stays explicit without a deeply nested binding pattern.

```vue
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [All rules](../all.md)
