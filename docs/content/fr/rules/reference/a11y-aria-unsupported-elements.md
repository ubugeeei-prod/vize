---
title: "a11y/aria-unsupported-elements"
description: "Interdire les attributs ARIA sur les éléments qui ne les prennent pas en charge."
---

# `a11y/aria-unsupported-elements`

Interdire les attributs ARIA sur les éléments qui ne les prennent pas en charge.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’élément de métadonnées porte `aria-hidden`, alors que `meta` ne prend pas en charge les attributs ARIA.

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

## Bon

Supprimer l’attribut ARIA conserve intacte la déclaration de jeu de caractères.

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/aria-unsupported-elements": "error"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L23)
