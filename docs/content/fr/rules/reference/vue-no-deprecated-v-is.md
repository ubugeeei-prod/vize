---
title: "vue/no-deprecated-v-is"
description: "Interdire la directive obsolète `v-is`."
---

# `vue/no-deprecated-v-is`

Interdire la directive obsolète `v-is`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

`v-is` est obsolète depuis Vue 3.1 ; l’exemple l’utilise encore pour changer le composant affiché.

```vue annotate="remove:2"
<template>
<tr v-is="'MyRow'"></tr>
</template>
```

## Bon

L’attribut `is` avec le préfixe `vue:`, ou `<component :is>`, sélectionne le composant à la place.

```vue annotate="add:2,3"
<template>
<tr is="vue:MyRow"></tr>
<component :is="MyRow" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-is": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_is.rs#L27)
