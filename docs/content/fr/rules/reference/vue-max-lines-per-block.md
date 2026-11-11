---
title: "vue/max-lines-per-block"
description: "Limite le nombre de lignes de chaque bloc SFC."
---

# `vue/max-lines-per-block`

Limite le nombre de lignes de chaque bloc SFC.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le bloc template dépasse le maximum configuré.

```vue annotate="remove:4,6,7,8,9,10,11,12,13,14,15,16,17"
<script setup lang="ts">
const label = "a";
</script>

<template>
  <ul>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
  </ul>
</template>

```

## Bon

Le bloc template reste dans le maximum configuré.

```vue annotate="add:5"
<script setup lang="ts">
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>

```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-lines-per-block": "warn"
    },
    "ruleOptions": {
      "vue/max-lines-per-block": {
        "template": 10
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/max_lines_per_block.rs#L15)
