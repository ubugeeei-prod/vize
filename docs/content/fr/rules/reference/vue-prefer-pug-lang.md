---
title: "vue/prefer-pug-lang"
description: "Préférer lang=\"pug\" au langage de template jade 1.x hérité."
---

# `vue/prefer-pug-lang`

Préférer lang="pug" au langage de template jade 1.x hérité.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Oui | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le template nomme jade, que Vue rend avec l’ancien paquet jade 1.x, et utilise le each précédé d’un tiret propre à jade.

```vue annotate="remove:1,3"
<template lang="jade">
ul
  - each item in [1, 2]
    li= item
</template>
```

## Bon

Le template nomme pug et écrit la boucle avec le mot-clé each de pug 3.

```vue annotate="add:1,3"
<template lang="pug">
ul
  each item in [1, 2]
    li= item
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-pug-lang": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prefer_pug_lang.rs#L42)
