---
title: "css/no-utility-classes"
description: "Déconseiller l’implémentation de classes utilitaires dans les styles des composants."
---

# `css/no-utility-classes`

Déconseiller l’implémentation de classes utilitaires dans les styles des composants.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | Non | Aucune |

Champ d’application : Blocs style en CSS standard (les blocs scss, sass, less et stylus sont ignorés).

## Mauvais

Les sélecteurs écrits utilisent des noms de forme utilitaire, tels que `.flex`, `.mt-4` et `.text-center`.

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

## Bon

Un sélecteur `.my-component` propre au composant rassemble ses styles sous un nom sémantique unique.

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/no-utility-classes": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Voir aussi

- [Règles Musea et CSS](../musea-and-css.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37)
