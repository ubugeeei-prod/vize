---
title: "vue/no-negated-condition"
description: "Interdit les ternaires commençant par une condition niée."
---

# `vue/no-negated-condition`

Interdit les ternaires commençant par une condition niée.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le ternaire commence par une condition niée.

```vue annotate="remove:2"
<template>
  <p>{{ !ready ? fallback : primary }}</p>
</template>
```

## Bon

La condition positive et l'ordre des branches expriment directement le choix.

```vue annotate="add:2"
<template>
  <p>{{ ready ? primary : fallback }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-negated-condition": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L139)
