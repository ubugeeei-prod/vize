---
title: "a11y/heading-has-content"
description: "Exiger un contenu accessible pour les éléments de titre."
---

# `a11y/heading-has-content`

Exiger un contenu accessible pour les éléments de titre.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le `h2` apporte un niveau de titre, mais ne contient aucun texte de titre.

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

## Bon

`Billing settings` fournit le contenu du titre de niveau deux existant.

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/heading-has-content": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L21)
