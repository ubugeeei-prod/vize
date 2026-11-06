---
title: "a11y/form-control-has-label"
description: "Exiger des libellés associés aux contrôles de formulaire."
---

# `a11y/form-control-has-label`

Exiger des libellés associés aux contrôles de formulaire.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le champ de recherche n’a pas de libellé indiquant ce que l’utilisateur doit saisir.

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

## Bon

Envelopper le champ dans un label associe le texte visible `Search` au contrôle.

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/form-control-has-label": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L21)
