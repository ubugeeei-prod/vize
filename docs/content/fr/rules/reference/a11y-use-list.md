---
title: "a11y/use-list"
description: "Suggérer des éléments de liste pour les textes ressemblant à des listes à puces."
---

# `a11y/use-list`

Suggérer des éléments de liste pour les textes ressemblant à des listes à puces.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Les tâches sont des paragraphes séparés précédés de tirets saisis, plutôt que des éléments de liste.

```vue annotate="remove:2,3"
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

## Bon

Une liste non ordonnée et ses éléments expriment les mêmes tâches avec une sémantique de liste.

```vue annotate="add:2,3,4,5"
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/use-list": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L38)
