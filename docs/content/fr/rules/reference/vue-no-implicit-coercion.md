---
title: "vue/no-implicit-coercion"
description: "Interdit les conversions implicites dans le template."
---

# `vue/no-implicit-coercion`

Interdit les conversions implicites dans le template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La double négation convertit implicitement la valeur.

```vue annotate="remove:2"
<template>
  <p>{{ !!value }}</p>
</template>
```

## Bon

`Boolean` rend la conversion explicite.

```vue annotate="add:2"
<template>
  <p>{{ Boolean(value) }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-implicit-coercion": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_implicit_coercion.rs#L12)
