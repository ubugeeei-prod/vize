---
title: "vue/no-v-text"
description: "Interdire la directive v-text ; préférer l’interpolation à doubles accolades."
---

# `vue/no-v-text`

Interdire la directive v-text ; préférer l’interpolation à doubles accolades.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le contenu du div est fourni par la directive v-text.

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

## Bon

L’interpolation à doubles accolades exprime la même liaison de texte directement dans le contenu de l’élément.

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-v-text": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31)
