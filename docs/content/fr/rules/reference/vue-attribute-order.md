---
title: "vue/attribute-order"
description: "Imposer un ordre cohérent des attributs."
---

# `vue/attribute-order`

Imposer un ordre cohérent des attributs.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le gestionnaire d’événement apparaît avant la directive structurelle v-if et l’attribut ordinaire id.

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

## Bon

v-if vient en premier, suivi de id puis du gestionnaire d’événement, conformément à l’ordre défini par la règle.

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/attribute-order": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36)
