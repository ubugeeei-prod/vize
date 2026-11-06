---
title: "a11y/heading-levels"
description: "Interdire de sauter des niveaux de titre."
---

# `a11y/heading-levels`

Interdire de sauter des niveaux de titre.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

La séquence des titres passe directement de `h1` à `h3`, sans niveau deux.

```vue annotate="remove:3"
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

## Bon

Remplacer le titre de facturation par `h2` conserve une hiérarchie de titres consécutive.

```vue annotate="add:3"
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/heading-levels": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34)
