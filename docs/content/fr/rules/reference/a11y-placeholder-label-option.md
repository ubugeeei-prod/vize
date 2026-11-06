---
title: "a11y/placeholder-label-option"
description: "Exiger une option d’invite dans un select required."
---

# `a11y/placeholder-label-option`

Exiger une option d’invite dans un select required.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le select est `required`, mais sa première option est Japan : le formulaire démarre avec un pays déjà choisi et `required` ne bloque jamais l’envoi.

```vue annotate="remove:4"
<template>
  <select v-model="country" required>
    <option value="jp">Japan</option>
    <option value="fr">France</option>
  </select>
</template>
```

## Bon

Une première option vide sert d’invite : le select démarre sans valeur, et `required` oblige à choisir un pays.

```vue annotate="add:3"
<template>
  <select v-model="country" required>
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/placeholder-label-option": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L47)
