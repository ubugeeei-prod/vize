---
title: "vue/multi-word-component-names"
description: "Exiger des noms de composants composés de plusieurs mots."
---

# `vue/multi-word-component-names`

Exiger des noms de composants composés de plusieurs mots.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Dans cet exemple limité au template, le diagnostic concerne le nom de fichier. Un composant qui déclare un nom statique est vérifié par ce nom plutôt que par son nom de fichier. L’exception exacte Item configurée n’autorise pas un autre nom composé d’un seul mot.

## Mauvais

Thing.vue est un nom composé d’un seul mot absent de la liste d’exceptions configurée.

`Thing.vue`

```vue
<template><p>Item</p></template>
```

## Bon

Item.vue est autorisé par l’exception exacte Item configurée.

`Item.vue`

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
    },
    "ruleOptions": {
      "vue/multi-word-component-names": {
        "ignores": [
          "Item"
        ]
      }
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L41)
