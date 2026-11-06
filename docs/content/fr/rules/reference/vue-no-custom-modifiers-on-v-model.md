---
title: "vue/no-custom-modifiers-on-v-model"
description: "Interdit les modificateurs personnalisés sur le `v-model` d'un composant pour assurer la compatibilité avec Vue 2."
---

# `vue/no-custom-modifiers-on-v-model`

Interdit les modificateurs personnalisés sur le `v-model` d'un composant pour assurer la compatibilité avec Vue 2.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Cette règle à activer explicitement impose un contrat de compatibilité avec Vue 2 au `v-model` des composants. Seuls les modificateurs `lazy`, `number` et `trim` sont autorisés. Les composants Vue 3 peuvent implémenter des modificateurs personnalisés ; les préréglages Vue 3 n'activent donc pas cette restriction. Les éléments natifs sont exclus de ce contrôle.

## Mauvais

Le modificateur `.capitalize` du composant est personnalisé et enfreint cette restriction de compatibilité avec Vue 2.

```vue annotate="remove:1"
<template><MyField v-model.capitalize="value" /></template>
```

## Bon

Utilisez le contrat ordinaire de `v-model` du composant sans modificateur personnalisé ; implémentez explicitement la mise en majuscule si elle est nécessaire.

```vue annotate="add:1"
<template><MyField v-model="value" /></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-custom-modifiers-on-v-model": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_custom_modifiers_on_v_model.rs#L22)
