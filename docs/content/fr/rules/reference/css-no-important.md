---
title: "css/no-important"
description: "Déconseiller !important en CSS."
---

# `css/no-important`

Déconseiller !important en CSS.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Blocs style en CSS standard (les blocs scss, sass, less et stylus sont ignorés).

## Mauvais

La déclaration de couleur remplace la priorité normale de la cascade avec `!important`.

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

## Bon

La couleur provient d’une propriété personnalisée sans déclaration important.

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
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
        "css/no-important": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea et CSS](../musea-and-css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14)
