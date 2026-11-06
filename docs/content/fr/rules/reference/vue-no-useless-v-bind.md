---
title: "vue/no-useless-v-bind"
description: "Interdire un v-bind dont la valeur est une simple chaîne littérale."
---

# `vue/no-useless-v-bind`

Interdire un v-bind dont la valeur est une simple chaîne littérale.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La liaison foo évalue une chaîne constante entre guillemets ou une chaîne de template sans interpolation.

```vue annotate="remove:2,3"
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

## Bon

La valeur constante devient un attribut statique ; les valeurs variables et interpolées conservent leur liaison.

```vue annotate="add:2,3,4"
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-useless-v-bind": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29)
