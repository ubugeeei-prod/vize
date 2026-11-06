---
title: "vue/valid-v-if"
description: "Exiger des directives `v-if` valides."
---

# `vue/valid-v-if`

Exiger des directives `v-if` valides.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Les conditions omettent une expression ou combinent `v-if` avec une directive else sur le même nœud.

```vue annotate="remove:2,3,4"
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

## Bon

Chaque `v-if` possède une condition non vide, telle que `ready` ou `count > 0`, sans directive else incompatible.

```vue annotate="add:2,3"
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-if": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29)
