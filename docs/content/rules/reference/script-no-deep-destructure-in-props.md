---
title: "script/no-deep-destructure-in-props"
---

# `script/no-deep-destructure-in-props`

Disallow deeply nested destructuring in defineProps

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: `opinionated`, `nuxt`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

This sole inline `<script setup>` has a top-level compiler macro binding that destructures `name` through `user`, exceeding the default depth of one. Arrays nested inside object bindings also increase depth. Opinionated and Nuxt enable this depth-policy warning; other presets require explicit opt-in: it does not read the compiler's `propsDestructure` setting, and `withDefaults` can compile despite the warning.

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

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L39) · [All rules](../all.md)
