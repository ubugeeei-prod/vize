---
title: "vue/no-loss-of-precision"
description: "Interdit les nombres littéraux qui perdent en précision."
---

# `vue/no-loss-of-precision`

Interdit les nombres littéraux qui perdent en précision.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L'entier ne peut pas être représenté précisément comme number.

```vue annotate="remove:2"
<template>
  <p>{{ 9007199254740993 }}</p>
</template>
```

## Bon

Une liaison précise évite le littéral imprécis dans le template.

```vue annotate="add:2"
<template>
  <p>{{ preciseValue }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-loss-of-precision": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_loss_of_precision.rs#L12)
