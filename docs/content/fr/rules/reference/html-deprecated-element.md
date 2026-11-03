---
title: "html/deprecated-element"
description: "Interdire les éléments HTML obsolètes."
---

# `html/deprecated-element`

Interdire les éléments HTML obsolètes.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

L’élément `center` utilise un élément de présentation HTML obsolète.

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

## Bon

Une section et une classe de style remplacent l’élément obsolète tout en préservant le contenu.

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
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
        "html/deprecated-element": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles HTML](../html.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33)
