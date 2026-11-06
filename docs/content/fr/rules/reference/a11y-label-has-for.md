---
title: "a11y/label-has-for"
description: "Exiger des contrôles de formulaire associés aux libellés."
---

# `a11y/label-has-for`

Exiger des contrôles de formulaire associés aux libellés.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le label séparé n’est ni associé au moyen de `for` ni placé autour du champ.

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

## Bon

`for="email"` correspond à l’identifiant du champ et associe explicitement les deux éléments.

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/label-has-for": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L31)
