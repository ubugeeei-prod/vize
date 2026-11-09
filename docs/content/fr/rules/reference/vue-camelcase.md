---
title: "vue/camelcase"
description: "Exige camelCase pour les identifiants des expressions du template."
---

# `vue/camelcase`

Exige camelCase pour les identifiants des expressions du template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

`user_name` contient un soulignement interne.

```vue annotate="remove:2"
<template>
  <p>{{ user_name }}</p>
</template>
```

## Bon

`userName` respecte camelCase.

```vue annotate="add:2"
<template>
  <p>{{ userName }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/camelcase": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L76)
