---
title: "a11y/anchor-has-content"
description: "Exiger un contenu accessible pour les éléments de lien."
---

# `a11y/anchor-has-content`

Exiger un contenu accessible pour les éléments de lien.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le lien `/settings` n’a ni texte ni autre contenu servant à le nommer ; sa destination n’a donc aucune description accessible.

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

## Bon

Le texte visible `Settings` fournit le contenu du lien vers la même destination.

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
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
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles d’accessibilité](../accessibility.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L19)
