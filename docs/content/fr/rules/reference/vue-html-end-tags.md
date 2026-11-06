---
title: "vue/html-end-tags"
description: "Exige une balise de fermeture explicite pour les éléments HTML qui permettent de l'omettre."
---

# `vue/html-end-tags`

Exige une balise de fermeture explicite pour les éléments HTML qui permettent de l'omettre.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Oui | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Cette règle exige des balises de fermeture explicites là où le parseur HTML autorise leur omission, par exemple pour `li`, `p` et `td`. Les balises de fermeture obligatoires manquantes relèvent de `parser/template` ; les éléments vides n'ont pas besoin de balise de fermeture.

## Mauvais

HTML permet de fermer implicitement ce `li` à la fermeture de `ul`, mais la règle exige une balise de fermeture explicite.

```vue annotate="remove:1"
<template><ul><li>First</ul></template>
```

## Bon

Ajoutez `</li>` après le contenu de l'élément, avant la fermeture du `ul` qui le contient.

```vue annotate="add:1"
<template><ul><li>First</li></ul></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/html-end-tags": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_end_tags.rs#L30)
