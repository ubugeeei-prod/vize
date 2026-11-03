---
title: "script/no-deep-destructure-in-props"
description: "Disallow deeply nested destructuring in defineProps."
---

# `script/no-deep-destructure-in-props`

Disallow deeply nested destructuring in defineProps.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

This sole inline `<script setup>` has a top-level compiler macro binding that destructures `name` through `user`, exceeding the default depth of one. Arrays nested inside object bindings also increase depth. Opinionated and Nuxt enable this depth-policy warning; other presets require explicit opt-in: it does not read the compiler's `propsDestructure` setting, and `withDefaults` can compile despite the warning.

```vue annotate="remove:2"
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

## Good

The props object remains intact, and a computed getter reads `props.user.name`. The nested access stays explicit without a deeply nested binding pattern.

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

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

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L39)
