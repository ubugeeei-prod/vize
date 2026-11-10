---
title: "vue/eqeqeq"
description: "Exige l'égalité stricte dans les expressions du template."
---

# `vue/eqeqeq`

Exige l'égalité stricte dans les expressions du template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L'égalité non stricte peut convertir les opérandes.

```vue annotate="remove:2"
<template>
  <p>{{ value == null }}</p>
</template>
```

## Bon

L'égalité stricte compare sans conversion.

```vue annotate="add:2"
<template>
  <p>{{ value === null }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/eqeqeq": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/eqeqeq.rs#L10)
