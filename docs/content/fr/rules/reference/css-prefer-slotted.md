---
title: "css/prefer-slotted"
description: "Recommander ::v-slotted() pour styliser le contenu des slots."
---

# `css/prefer-slotted`

Recommander ::v-slotted() pour styliser le contenu des slots.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Blocs style CSS, SCSS, Sass, Less et Stylus, tels qu’écrits (sans préprocesseur).
Dans un même bloc Stylus, utilisez soit la syntaxe à accolades, soit la syntaxe indentée ; une syntaxe mixte est lue comme du Stylus à accolades.

## Mauvais

La feuille de style scoped cible le point d’insertion `slot` plutôt que les éléments fournis au slot.

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

## Bon

`:slotted(.label)` cible l’élément label fourni au moyen du sélecteur de slot scoped.

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/prefer-slotted": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles CSS](../css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34)
