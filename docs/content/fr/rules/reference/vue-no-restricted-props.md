---
title: "vue/no-restricted-props"
description: "Interdit la prop configurée."
---

# `vue/no-restricted-props`

Interdit la prop configurée.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le composant déclare la prop interdite third.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ third?: number }>();
</script>
<template>
  <p>ok</p>
</template>
```

## Bon

Le composant déclare la prop first.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ first?: number }>();
</script>
<template>
  <p>ok</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-props": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-props": [
        "third"
      ]
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_props.rs#L12)
