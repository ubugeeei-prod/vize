---
title: "css/prefer-nested-selectors"
description: "Recommander l’imbrication CSS pour les sélecteurs de descendants."
---

# `css/prefer-nested-selectors`

Recommander l’imbrication CSS pour les sélecteurs de descendants.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Blocs style en CSS standard (les blocs scss, sass, less et stylus sont ignorés).

## Mauvais

Le sélecteur de descendant `.card .title` répète le sélecteur parent dans une règle à plat.

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

## Bon

La règle `.title` est imbriquée dans `.card`, ce qui rassemble la relation de style parent-enfant.

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea et CSS](../musea-and-css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14)
