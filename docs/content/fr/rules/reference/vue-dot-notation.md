---
title: "vue/dot-notation"
description: "Préfère la notation pointée pour les propriétés statiques."
---

# `vue/dot-notation`

Préfère la notation pointée pour les propriétés statiques.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La propriété statique `name` est accédée avec des crochets.

```vue annotate="remove:2"
<template>
  <p>{{ user["name"] }}</p>
</template>
```

## Bon

La notation pointée exprime directement la même propriété.

```vue annotate="add:2"
<template>
  <p>{{ user.name }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/dot-notation": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/dot_notation.rs#L10)
