---
title: "a11y/aria-role"
description: "Exiger un rôle ARIA valide et non abstrait pour les éléments possédant un rôle ARIA."
---

# `a11y/aria-role`

Exiger un rôle ARIA valide et non abstrait pour les éléments possédant un rôle ARIA.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

`datepicker` n’est pas un rôle ARIA reconnu pour cette section.

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

## Bon

La section utilise le rôle reconnu `dialog` et un libellé décrivant la sélection de la date.

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/aria-role": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L23)
