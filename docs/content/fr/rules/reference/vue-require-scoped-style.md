---
title: "vue/require-scoped-style"
description: "Exiger l’attribut scoped sur les balises style."
---

# `vue/require-scoped-style`

Exiger l’attribut scoped sur les balises style.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Non | Aucune |

Champ d’application : Templates et blocs des SFC Vue, avec le contexte du script requis par la règle.

## Mauvais

Le style `.button` n’est pas scoped et peut affecter les éléments correspondants en dehors de ce composant.

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

## Bon

Ajouter `scoped` applique la portée du composant Vue aux mêmes sélecteur et déclarations.

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
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
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Vue](../vue.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49)
