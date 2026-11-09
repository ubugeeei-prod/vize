---
title: "vue/no-content-outside-block"
description: "Interdit le contenu hors des blocs supérieurs du SFC."
---

# `vue/no-content-outside-block`

Interdit le contenu hors des blocs supérieurs du SFC.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le texte initial est hors de tous les blocs du SFC.

```vue annotate="remove:1,2"
unexpected
<template><div /></template>
```

## Bon

Tout le contenu se trouve dans le bloc template.

```vue annotate="add:1"
<template><div>expected</div></template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-content-outside-block": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L40)
