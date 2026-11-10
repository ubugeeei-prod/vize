---
title: "vue/no-sparse-arrays"
description: "Interdit les tableaux creux dans les expressions du template."
---

# `vue/no-sparse-arrays`

Interdit les tableaux creux dans les expressions du template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L'absence du deuxième élément crée un tableau creux.

```vue annotate="remove:2"
<template>
  <p>{{ [1,, 3] }}</p>
</template>
```

## Bon

Chaque position du tableau a une valeur explicite.

```vue annotate="add:2"
<template>
  <p>{{ [1, 2, 3] }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-sparse-arrays": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_sparse_arrays.rs#L10)
