---
title: "a11y/alt-text"
description: "Exiger un texte alternatif pour les éléments multimédias."
---

# `a11y/alt-text`

Exiger un texte alternatif pour les éléments multimédias.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le contrôle de soumission sous forme d’image ne fournit que l’URL de son image ; il n’a pas de texte `alt` décrivant l’action.

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

## Bon

`alt="Submit search"` donne au contrôle image un nom accessible décrivant la soumission de la recherche.

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/alt-text": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L37)
