---
title: "vue/no-deprecated-filter"
description: "Interdire la syntaxe obsolète des filtres de Vue 2 utilisant l’opérateur pipe."
---

# `vue/no-deprecated-filter`

Interdire la syntaxe obsolète des filtres de Vue 2 utilisant l’opérateur pipe.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le pipe utilise la syntaxe supprimée des filtres Vue pour appliquer capitalize.

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

## Bon

L’appel capitalize(message) applique la transformation sous la forme d’une expression ordinaire.

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-filter": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L56)
