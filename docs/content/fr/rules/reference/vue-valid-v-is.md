---
title: "vue/valid-v-is"
description: "Exiger des directives `v-is` valides."
---

# `vue/valid-v-is`

Exiger des directives `v-is` valides.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

`v-is` possède un argument ou un modificateur, n’a pas de valeur, ou est placé sur un composant plutôt que sur un élément HTML natif.

```vue annotate="remove:2,3,4,5"
<template>
<tr v-is:arg="'MyRow'"></tr>
<tr v-is.mod="'MyRow'"></tr>
<tr v-is></tr>
<MyComponent v-is="'MyRow'" />
</template>
```

## Bon

`v-is` avec une valeur sur un élément natif tel que `<tr>` désigne le composant à afficher à sa place.

```vue annotate="add:2"
<template>
<tr v-is="'MyRow'"></tr>
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
        "vue/valid-v-is": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_is.rs#L30)
