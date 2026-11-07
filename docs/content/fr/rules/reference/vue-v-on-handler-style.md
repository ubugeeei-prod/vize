---
title: "vue/v-on-handler-style"
description: "Imposer une référence de méthode pour les gestionnaires v-on."
---

# `vue/v-on-handler-style`

Imposer une référence de méthode pour les gestionnaires v-on.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Les gestionnaires utilisent des fonctions en ligne, des modifications ou plusieurs instructions au lieu de références de méthode réutilisables.

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

## Bon

Utilisez une référence de gestionnaire. Déplacez la logique en ligne dans une méthode nommée afin que le template conserve un gestionnaire stable et réutilisable.

```vue annotate="add:2,3"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-on-handler-style": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L31)
