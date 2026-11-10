---
title: "vue/no-empty-pattern"
description: "Interdit les motifs de déstructuration vides dans le template."
---

# `vue/no-empty-pattern`

Interdit les motifs de déstructuration vides dans le template.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le callback déstructure un objet vide.

```vue annotate="remove:2"
<template>
  <p>{{ items.map(({}) => 1) }}</p>
</template>
```

## Bon

Le callback nomme la propriété utilisée.

```vue annotate="add:2"
<template>
  <p>{{ items.map(({ id }) => id) }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-empty-pattern": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_empty_pattern.rs#L10)
