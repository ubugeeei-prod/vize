---
title: "vue/prefer-single-event-payload"
description: "Impose un seul argument de charge utile pour les événements personnalisés."
---

# `vue/prefer-single-event-payload`

Impose un seul argument de charge utile pour les événements personnalisés.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L'événement move déclare et émet deux valeurs, et un écouteur en ligne ne reçoit que x.

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits<{ move: [x: number, y: number] }>()
emit("move", 1, 2)
</script>
<template />
```

## Bon

L'événement move transmet un objet, et chaque écouteur reçoit les deux coordonnées.

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits<{ move: [point: { x: number; y: number }] }>()
emit("move", { x: 1, y: 2 })
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
      "vue/prefer-single-event-payload": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/event_payload.rs#L34)
