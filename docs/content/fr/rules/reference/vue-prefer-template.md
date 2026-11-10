---
title: "vue/prefer-template"
description: "Préfère les littéraux de gabarit à la concaténation."
---

# `vue/prefer-template`

Préfère les littéraux de gabarit à la concaténation.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Une chaîne littérale est concaténée avec `name`.

```vue annotate="remove:2"
<template>
  <p>{{ "Hello " + name }}</p>
</template>
```

## Bon

Le littéral de gabarit garde l'interpolation dans une seule chaîne.

```vue annotate="add:2"
<template>
  <p>{{ `Hello ${name}` }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-template": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/prefer_template.rs#L10)
