---
title: "vue/no-root-v-if"
description: "Interdire v-if sur l’unique élément racine d’un template."
---

# `vue/no-root-v-if`

Interdire v-if sur l’unique élément racine d’un template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La racine du composant elle-même apparaît et disparaît sous le contrôle de v-if.

```vue annotate="remove:2"
<template>
  <div v-if="show">content</div>
</template>
```

## Bon

Un div externe stable reste la racine, tandis que le paragraphe imbriqué porte la condition de visibilité.

```vue annotate="add:2,3,4"
<template>
  <div>
    <p v-if="show">content</p>
  </div>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-root-v-if": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40)
