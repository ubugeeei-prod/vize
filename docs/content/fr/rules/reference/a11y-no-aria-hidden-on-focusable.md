---
title: "a11y/no-aria-hidden-on-focusable"
description: "Interdire aria-hidden=\"true\" sur les éléments pouvant recevoir le focus."
---

# `a11y/no-aria-hidden-on-focusable`

Interdire aria-hidden="true" sur les éléments pouvant recevoir le focus.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le bouton Close pouvant recevoir le focus est masqué dans l’arbre d’accessibilité avec `aria-hidden="true"`.

```vue annotate="remove:2"
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

## Bon

Le bouton reste exposé et reçoit un libellé `Close` au lieu d’être masqué.

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
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
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19)
