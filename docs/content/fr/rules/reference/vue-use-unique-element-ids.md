---
title: "vue/use-unique-element-ids"
description: "Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques."
---

# `vue/use-unique-element-ids`

Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’identifiant littéral `email` est réutilisé par chaque instance de ce composant, ce qui peut faire pointer son label vers la mauvaise instance lorsque plusieurs sont affichées.

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## Bon

`useId()` produit l’`emailId` de l’instance ; liez la même valeur au `for` du label et à l’`id` de l’input.

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/use-unique-element-ids": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52)
