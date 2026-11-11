---
title: "vue/max-props"
description: "Limite le nombre de props d'un composant Vue."
---

# `vue/max-props`

Limite le nombre de props d'un composant Vue.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le composant déclare trois props, au-delà du maximum configuré.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ a: number; b: number; c: number }>();
</script>
<template>
  <p>props</p>
</template>
```

## Bon

Le composant déclare une prop.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ a: number }>();
</script>
<template>
  <p>props</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-props": "warn"
    },
    "ruleOptions": {
      "vue/max-props": {
        "maxProps": 2
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/max_props.rs#L16)
