---
title: "vue/no-array-index-key"
description: "Interdire l’utilisation directe de la variable d’index de v-for comme :key."
---

# `vue/no-array-index-key`

Interdire l’utilisation directe de la variable d’index de v-for comme :key.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La clé de la liste est son index actuel ; l’identité de l’élément change donc lorsque la liste est réordonnée.

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

## Bon

La clé provient de item.id, ce qui préserve l’identité de chaque élément lorsqu’il change de position.

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-array-index-key": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32)
