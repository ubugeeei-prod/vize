---
title: "html/no-empty-palpable-content"
description: "Interdire les éléments vides qui attendent un contenu visible."
---

# `html/no-empty-palpable-content`

Interdire les éléments vides qui attendent un contenu visible.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le paragraphe, l’élément de liste et la cellule de tableau ont tous un contenu palpable vide.

```vue annotate="remove:2,3,4"
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

## Bon

Du texte remplit le paragraphe, une interpolation fournit le contenu de l’élément de liste, et aria-label nomme explicitement la cellule autrement vide.

```vue annotate="add:2,3,4"
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-empty-palpable-content": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles HTML](../html.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32)
