---
title: "css/no-id-selectors"
description: "Déconseiller les sélecteurs d’identifiant en CSS."
---

# `css/no-id-selectors`

Déconseiller les sélecteurs d’identifiant en CSS.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Blocs style en CSS standard (les blocs scss, sass, less et stylus sont ignorés).

## Mauvais

`#submit` lie la règle de style à un sélecteur d’identifiant.

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

## Bon

La classe `.submit` fournit un point d’accroche de style réutilisable sans sélecteur d’identifiant.

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
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
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles CSS](../css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19)
