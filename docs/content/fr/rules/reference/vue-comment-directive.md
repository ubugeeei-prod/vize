---
title: "vue/comment-directive"
description: "Prend en charge les commentaires de contrôle des diagnostics dans `<template>`."
---

# `vue/comment-directive`

Prend en charge les commentaires de contrôle des diagnostics dans `<template>`.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

Les commentaires de suppression inutilisés dans le template ne sont signalés que si `reportUnusedDisableDirectives: true` est activé. L'exemple supprime un commentaire inutile ; les commentaires qui suppriment réellement un diagnostic restent utiles. Les noms de règles listés ne comptent que lorsque Vize exécute ces règles.

## Mauvais

Le commentaire qui désactive toutes les règles ne supprime aucun diagnostic sur le `div` suivant ; le contrôle des directives inutilisées le signale donc.

```vue annotate="remove:2"
<template>
  <!-- eslint-disable-next-line -->
  <div />
</template>
```

## Bon

Supprimez le commentaire inutile et conservez l'élément, afin de laisser les contrôles de lint actifs sur cette ligne.

```vue
<template>
  <div />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/comment-directive": "error"
    },
    "reportUnusedDisableDirectives": true
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/comment_directive.rs#L33)
