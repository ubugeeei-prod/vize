---
title: "vue/no-shadow-native-events"
description: "Interdit les événements de composant qui entrent en collision avec les événements natifs."
---

# `vue/no-shadow-native-events`

Interdit les événements de composant qui entrent en collision avec les événements natifs.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le composant déclare `click`, en collision avec un événement natif.

```vue annotate="remove:2"
<script setup>
const emit = defineEmits(['click'])
</script>
<template />
```

## Bon

L'événement spécifique `activate` évite la collision.

```vue annotate="add:2"
<script setup>
const emit = defineEmits(['activate'])
</script>
<template />
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-shadow-native-events": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/events.rs#L13)
