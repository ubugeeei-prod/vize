---
title: "a11y/tabindex-no-positive"
description: "Interdire les valeurs positives de tabindex."
---

# `a11y/tabindex-no-positive`

Interdire les valeurs positives de tabindex.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Un tabindex positif de 3 crée un ordre de focus personnalisé avant les contrôles ordinaires.

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

## Bon

Le bouton utilise son ordre de focus natif sans tabindex positif.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/tabindex-no-positive": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16)
