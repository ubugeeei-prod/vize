---
title: "vue/multi-word-component-names"
description: "Exiger des noms de composants composés de plusieurs mots."
---

# `vue/multi-word-component-names`

Exiger des noms de composants composés de plusieurs mots.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Le diagnostic concerne le nom de fichier. Renommez ce même composant ; modifier une balise enfant ne corrige pas le problème.

## Mauvais

Item.vue donne au composant un nom composé d’un seul mot.

`Item.vue`

```vue
<template><p>Item</p></template>
```

## Bon

TodoItem.vue donne au même template un nom de composant composé de plusieurs mots.

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/multi-word-component-names": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34)
