---
title: "a11y/no-i-for-icon"
description: "Interdire l’élément &lt;i&gt; pour les icônes."
---

# `a11y/no-i-for-icon`

Interdire l’élément &lt;i&gt; pour les icônes.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’icône est affichée avec `i`, dont la sémantique textuelle ne décrit pas une action représentée uniquement par une icône.

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

## Bon

Un span décoratif masque le glyphe de l’icône, tandis que le texte séparé `Delete item` nomme l’action du bouton.

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35)
