---
title: "a11y/aria-props"
description: "Interdire les attributs ARIA invalides."
---

# `a11y/aria-props`

Interdire les attributs ARIA invalides.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

`aria-lable` est mal orthographié et n’est pas un attribut ARIA pris en charge.

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

## Bon

L’attribut pris en charge `aria-label` fournit le nom du bouton.

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/aria-props": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L20)
