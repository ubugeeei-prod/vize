---
title: "vue/no-irregular-whitespace"
description: "Détecte les espaces irréguliers dans les expressions du template."
---

# `vue/no-irregular-whitespace`

Détecte les espaces irréguliers dans les expressions du template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L'expression contient une espace insécable.

```vue annotate="remove:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## Bon

L'expression utilise une espace ordinaire du code source.

```vue annotate="add:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-irregular-whitespace": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_irregular_whitespace.rs#L20)
