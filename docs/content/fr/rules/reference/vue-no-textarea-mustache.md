---
title: "vue/no-textarea-mustache"
description: "Interdire l’interpolation à doubles accolades dans `<textarea>`."
---

# `vue/no-textarea-mustache`

Interdire l’interpolation à doubles accolades dans `<textarea>`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le textarea place message dans une interpolation enfant au lieu de lier sa valeur.

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

## Bon

v-model lie la valeur modifiable du textarea à message.

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
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
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26)
