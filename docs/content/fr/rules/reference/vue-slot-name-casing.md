---
title: "vue/slot-name-casing"
description: "Imposer camelCase aux noms statiques des points d’insertion de slot rendus."
---

# `vue/slot-name-casing`

Imposer camelCase aux noms statiques des points d’insertion de slot rendus.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Contrôle le nom statique d’un point d’insertion de slot rendu. La syntaxe de fourniture v-slot et #name côté parent est hors du champ de cette règle.

## Mauvais

Le point d’insertion de slot rendu utilise le nom statique `my-slot`, séparé par des tirets, alors que la convention par défaut exige camelCase.

```vue annotate="remove:2"
<template>
<MyCard><slot name="my-slot" /></MyCard>
</template>
```

## Bon

Le nom statique du point d’insertion utilise camelCase. La syntaxe de fourniture `#name` côté parent est hors du champ de cette règle.

```vue annotate="add:2"
<template>
<MyCard><slot name="mySlot" /></MyCard>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/slot-name-casing": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L28)
